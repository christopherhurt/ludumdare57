use anyhow::{anyhow, Result};
use winit::platform::windows::EventLoopBuilderExtWindows;
use core::panic;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::{thread, usize};
use std::thread::JoinHandle;
use strum::{EnumCount, IntoEnumIterator};
use vulkanalia::Device as vk_Device;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension, SurfaceKHR};
use vulkanalia::window as vk_window;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window as winit_Window, WindowAttributes};

use crate::core::Color;
use crate::core::mesh::Vertex;
use crate::ecs::ComponentActions;
use crate::ecs::component::Component;
use crate::math::{vec2, Vec2};
use crate::render_engine::{Device, RenderMeshId, RenderEngine, RenderEngineInitProps, RenderState, VirtualButton, VirtualKey, VirtualElementState, Window};
use crate::render_engine::vulkan::vulkan_resources::{
    create_vk_instance,
    pick_physical_device,
    create_logical_device,
    create_swapchain,
    create_render_pass,
    create_descriptor_set_layout,
    create_pipeline,
    create_command_pool,
    create_depth_objects,
    create_framebuffer,
    create_uniform_buffer,
    create_descriptor_pool,
    create_descriptor_sets,
    create_command_buffer,
    create_sync_objects,
    create_vertex_buffer,
    create_index_buffer,
};
use crate::render_engine::vulkan::vulkan_structs::{BufferResources, FrameSyncObjects, ImageResources, VulkanMesh, Pipeline, Swapchain, UniformBufferObject};

mod vulkan_resources;
mod vulkan_structs;
mod vulkan_utils;

const MAX_FRAMES_IN_FLIGHT: usize = 3;
// TODO: we'll want a way to resize the uniform buffer and/or overwrite it multiple times per frame, instead of capping the limit on total uniform desciptors like this
const NUM_UNIFORM_DESCRIPTORS: usize = 4096;

pub struct VulkanRenderEngine {
    mesh_id_counter: usize,
    state_sender: SyncSender<RenderState>,
    mesh_sender: Sender<(RenderMeshId, Arc<Vec<Vertex>>, Arc<Vec<u32>>)>,
    keys_down: HashMap<VirtualKey, bool>,
    keys_pressed: HashMap<VirtualKey, bool>,
    keys_released: HashMap<VirtualKey, bool>,
    keys_receiver: Receiver<(VirtualKey, VirtualElementState)>,
    buttons_down: HashMap<VirtualButton, bool>,
    buttons_pressed: HashMap<VirtualButton, bool>,
    buttons_released: HashMap<VirtualButton, bool>,
    buttons_receiver: Receiver<(VirtualButton, VirtualElementState)>,
    mouse_pos: Option<Vec2>,
    mouse_pos_receiver: Receiver<Option<Vec2>>,
    window_extent: vk::Extent2D,
    window_extent_receiver: Receiver<vk::Extent2D>,
    is_closing: Arc<AtomicBool>,
    render_thread_join_handle: Option<JoinHandle<()>>,
}

impl Component for VulkanRenderEngine {}
impl ComponentActions for VulkanRenderEngine {}

struct VulkanApplication {
    init_props: RenderEngineInitProps,
    state_receiver: Receiver<RenderState>,
    mesh_receiver: Receiver<(RenderMeshId, Arc<Vec<Vertex>>, Arc<Vec<u32>>)>,
    is_minimized: bool,
    is_resized: bool,
    is_closing: Arc<AtomicBool>,
    keys_sender: SyncSender<(VirtualKey, VirtualElementState)>,
    buttons_sender: SyncSender<(VirtualButton, VirtualElementState)>,
    mouse_pos_sender: SyncSender<Option<Vec2>>,
    window_extent_sender: SyncSender<vk::Extent2D>,
    context: Option<VulkanContext>,
    swapchain_fences: Vec<vk::Fence>,
    frame: usize,
}

struct VulkanContext {
    clear_color: Color,

    winit_window: winit_Window,
    // This isn't needed after creation time, but it needs to retained for the lifetime of VulkanContext to prevent memory leaks
    _entry: Entry,
    vk_instance: Instance,
    surface: SurfaceKHR,
    debug_messenger: Option<DebugUtilsMessengerEXT>,

    ubo_alignment: usize,

    physical_device: vk::PhysicalDevice,
    device: vk_Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: Swapchain,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline: Pipeline,
    single_time_command_pool: vk::CommandPool,
    per_frame_command_pools: Vec<vk::CommandPool>,
    depth_image_resources: ImageResources,
    depth_image_view: vk::ImageView,
    framebuffers: Vec<vk::Framebuffer>,
    uniform_buffers: Vec<BufferResources>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<Vec<vk::DescriptorSet>>,
    per_frame_command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: Vec<FrameSyncObjects>,

    meshes: HashMap<RenderMeshId, VulkanMesh>,
}

impl VulkanContext {
    fn new(
        init_properties: RenderEngineInitProps,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        let loader = unsafe { LibloadingLoader::new(LIBRARY).unwrap_or_else(|_| panic!("Failed to create loader for {}", LIBRARY)) };
        let entry = unsafe { Entry::new(loader) }.unwrap_or_else(|_| panic!("Failed to load entry point for {}", LIBRARY));

        let win_properties = init_properties.window_props;
        let window_attribs = WindowAttributes::default()
            .with_title(win_properties.title.clone())
            .with_inner_size(LogicalSize::new(win_properties.width, win_properties.height))
            .with_resizable(win_properties.is_resizable);
        let winit_window = event_loop.create_window(window_attribs).unwrap_or_else(|_| panic!("Failed to create winit window!"));

        unsafe {
            let (vk_instance, debug_messenger) = create_vk_instance(&winit_window, &entry, init_properties.debug_enabled)
                .unwrap_or_else(|_| panic!("Failed to create Vulkan instance!"));

            let surface = vk_window::create_surface(&vk_instance, &winit_window, &winit_window).unwrap_or_else(|_| panic!("Failed to create surface"));

            let physical_device = pick_physical_device(&vk_instance, surface).unwrap_or_else(|e| panic!("{}", e));

            let ubo_alignment = UniformBufferObject::get_offset_alignment(vk_instance.get_physical_device_properties(physical_device).limits.min_uniform_buffer_offset_alignment as usize);

            let (device, graphics_queue, present_queue) = create_logical_device(&vk_instance, surface, physical_device, init_properties.debug_enabled).unwrap_or_else(|e| panic!("{}", e));
            let swapchain = create_swapchain(MAX_FRAMES_IN_FLIGHT as u32, &winit_window, &vk_instance, surface, &device, physical_device).unwrap_or_else(|e| panic!("{}", e));
            let render_pass = create_render_pass(&vk_instance, &device, physical_device, swapchain.format).unwrap_or_else(|e| panic!("{}", e));
            let descriptor_set_layout = create_descriptor_set_layout(&device).unwrap_or_else(|e| panic!("{}", e));
            let pipeline = create_pipeline(&device, render_pass, &swapchain, descriptor_set_layout).unwrap_or_else(|e| panic!("{}", e));
            let single_time_command_pool = create_command_pool(&vk_instance, &device, surface, physical_device).unwrap_or_else(|e| panic!("{}", e));
            let per_frame_command_pools = (0..swapchain.images.len()).map(|_| create_command_pool(&vk_instance, &device, surface, physical_device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let (depth_image_resources, depth_image_view) = create_depth_objects(&vk_instance, &device, physical_device, &swapchain.extent, single_time_command_pool, graphics_queue).unwrap_or_else(|e| panic!("{}", e));
            let framebuffers = swapchain.image_views.iter().map(|i| create_framebuffer(&device, render_pass, swapchain.extent, *i, depth_image_view).unwrap_or_else(|e| panic!("{}", e))).collect();
            // TODO: split up into more than one uniform buffer per frame
            let uniform_buffers = (0..swapchain.images.len()).map(|_| create_uniform_buffer::<UniformBufferObject>(&vk_instance, &device, physical_device, NUM_UNIFORM_DESCRIPTORS, ubo_alignment).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let descriptor_pool = create_descriptor_pool(&device, swapchain.images.len() * NUM_UNIFORM_DESCRIPTORS).unwrap_or_else(|e| panic!("{}", e));
            let descriptor_sets = (0..swapchain.images.len()).map(|i| create_descriptor_sets(&device, descriptor_set_layout, descriptor_pool, &uniform_buffers[i], NUM_UNIFORM_DESCRIPTORS, size_of::<UniformBufferObject>(), ubo_alignment).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let per_frame_command_buffers = per_frame_command_pools.iter().map(|p| create_command_buffer(&device, *p).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let sync_objects = (0..MAX_FRAMES_IN_FLIGHT).map(|_| create_sync_objects(&device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();

            Self {
                clear_color: init_properties.clear_color,

                winit_window,
                _entry: entry,
                vk_instance,
                surface,
                debug_messenger,

                ubo_alignment,

                physical_device,
                device,
                graphics_queue,
                present_queue,
                swapchain,
                render_pass,
                descriptor_set_layout,
                pipeline,
                single_time_command_pool,
                per_frame_command_pools,
                depth_image_resources,
                depth_image_view,
                framebuffers,
                uniform_buffers,
                descriptor_pool,
                descriptor_sets,
                per_frame_command_buffers,
                sync_objects,

                meshes: HashMap::new(),
            }
        }
    }

    unsafe fn update_command_buffer(&mut self, image_index: usize, state: RenderState) -> Result<()> {
        let command_pool = self.per_frame_command_pools[image_index];
        self.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.per_frame_command_buffers[image_index];
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.swapchain.extent);
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [self.clear_color.r, self.clear_color.g, self.clear_color.b, self.clear_color.a],
            },
        };
        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };
        let clear_values = &[color_clear_value, depth_clear_value];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);
        self.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);

        self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);

        // TODO: do this in a way that doesn't involve a double copy of RenderState? (i.e. serializing to RenderState, then copying RenderState to a buffer)
        //  Also, we don't want to all uniforms every entity... only the per-entity uniforms
        for (i, e) in state.entity_states.iter().enumerate() {
            let mesh = self.meshes.get(&e.mesh_id).unwrap_or_else(|| panic!("No mesh exists for ID {}", e.mesh_id.0));

            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[self.descriptor_sets[image_index][i]],
                &[],
            );
            self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer.buffer], &[0]);
            self.device.cmd_bind_index_buffer(command_buffer, mesh.index_buffer.buffer, 0, vk::IndexType::UINT32);
            self.device.cmd_draw_indexed(command_buffer, mesh.index_count as u32, 1, 0, 0, 0);
        }

        self.device.cmd_end_render_pass(command_buffer);

        self.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    unsafe fn recreate_swapchain(&mut self) -> Result<()> {
        self.destroy_swapchain()?;

        self.swapchain = create_swapchain(MAX_FRAMES_IN_FLIGHT as u32, &self.winit_window, &self.vk_instance, self.surface, &self.device, self.physical_device).unwrap_or_else(|e| panic!("{}", e));
        self.render_pass = create_render_pass(&self.vk_instance, &self.device, self.physical_device, self.swapchain.format).unwrap_or_else(|e| panic!("{}", e));
        self.pipeline = create_pipeline(&self.device, self.render_pass, &self.swapchain, self.descriptor_set_layout).unwrap_or_else(|e| panic!("{}", e));
        (self.depth_image_resources, self.depth_image_view) = create_depth_objects(&self.vk_instance, &self.device, self.physical_device, &self.swapchain.extent, self.single_time_command_pool, self.graphics_queue).unwrap_or_else(|e| panic!("{}", e));
        self.framebuffers = self.swapchain.image_views.iter().map(|i| create_framebuffer(&self.device, self.render_pass, self.swapchain.extent, *i, self.depth_image_view).unwrap_or_else(|e| panic!("{}", e))).collect();
        self.uniform_buffers = (0..self.swapchain.images.len()).map(|_| create_uniform_buffer::<UniformBufferObject>(&self.vk_instance, &self.device, self.physical_device, NUM_UNIFORM_DESCRIPTORS, self.ubo_alignment).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
        self.descriptor_pool = create_descriptor_pool(&self.device, self.swapchain.images.len() * NUM_UNIFORM_DESCRIPTORS).unwrap_or_else(|e| panic!("{}", e));
        self.descriptor_sets = (0..self.swapchain.images.len()).map(|i| create_descriptor_sets(&self.device, self.descriptor_set_layout, self.descriptor_pool, &self.uniform_buffers[i], NUM_UNIFORM_DESCRIPTORS, size_of::<UniformBufferObject>(), self.ubo_alignment).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
        self.per_frame_command_buffers = self.per_frame_command_pools.iter().map(|p| create_command_buffer(&self.device, *p).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();

        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self) -> Result<()> {
        self.device.device_wait_idle()?;

        self.device.destroy_image_view(self.depth_image_view, None);
        self.device.free_memory(self.depth_image_resources.memory, None);
        self.device.destroy_image(self.depth_image_resources.image, None);

        self.device.destroy_descriptor_pool(self.descriptor_pool, None);
        self.uniform_buffers
            .iter()
            .for_each(|b| {
                self.device.destroy_buffer((*b).buffer, None);
                self.device.free_memory((*b).memory, None);
            });

        self.framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));

        self.device.destroy_pipeline(self.pipeline.pipeline, None);
        self.device.destroy_pipeline_layout(self.pipeline.layout, None);

        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain.image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.swapchain.swapchain, None);

        Ok(())
    }

    unsafe fn destroy(mut self) -> Result<()> {
        self.device.device_wait_idle()?;

        self.destroy_swapchain()?;

        self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

        self.meshes.values().for_each(|m| {
            self.device.destroy_buffer((*m).vertex_buffer.buffer, None);
            self.device.free_memory((*m).vertex_buffer.memory, None);

            self.device.destroy_buffer((*m).index_buffer.buffer, None);
            self.device.free_memory((*m).index_buffer.memory, None);
        });

        self.sync_objects
            .iter()
            .for_each(|s| {
                self.device.destroy_fence((*s).in_flight_fence, None);
                self.device.destroy_semaphore((*s).image_available_semaphore, None);
                self.device.destroy_semaphore((*s).render_finished_semaphore, None);
            });

        self.device.destroy_command_pool(self.single_time_command_pool, None);
        self.per_frame_command_pools
            .iter()
            .for_each(|p| self.device.destroy_command_pool(*p, None));

        self.device.destroy_device(None);

        self.vk_instance.destroy_surface_khr(self.surface, None);

        if let Some(d) = self.debug_messenger {
            self.vk_instance.destroy_debug_utils_messenger_ext(d, None);
        }

        self.vk_instance.destroy_instance(None);

        Ok(())
    }
}

unsafe fn update_uniforms(
    device: &vk_Device,
    uniform_memory: vk::DeviceMemory,
    ubos: &Vec<UniformBufferObject>,
    ubo_alignment: usize,
) -> Result<()> {
    if !ubos.is_empty() {
        let memory = device.map_memory(
            uniform_memory,
            0,
            (ubo_alignment * ubos.len()) as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        for i in 0..ubos.len() {
            std::ptr::copy_nonoverlapping(ubos.as_ptr().add(i), (memory.cast::<u8>().add(i * ubo_alignment)).cast(), 1);
        }

        device.unmap_memory(uniform_memory);

        Ok(())
    } else {
        Ok(())
    }
}

impl RenderEngine<VulkanRenderEngine, VulkanRenderEngine> for VulkanRenderEngine {
    fn new(init_props: RenderEngineInitProps) -> Result<Self> {
        // TODO: update this so we can overwrite the buffered state(s), rather than block the sender, if the sender gets ahead of the receiver
        let (state_sender, state_receiver) = mpsc::sync_channel::<RenderState>(1);
        let (mesh_sender, mesh_receiver) = mpsc::channel();
        let (keys_sender, keys_receiver) = mpsc::sync_channel::<(VirtualKey, VirtualElementState)>(256);
        let (buttons_sender, buttons_receiver) = mpsc::sync_channel::<(VirtualButton, VirtualElementState)>(256);
        let (mouse_pos_sender, mouse_pos_receiver) = mpsc::sync_channel::<Option<Vec2>>(256);
        let (window_extent_sender, window_extent_receiver) = mpsc::sync_channel::<vk::Extent2D>(256);

        let is_closing = Arc::new(AtomicBool::new(false));

        let moved_properties = init_props.clone();
        let moved_is_closing = is_closing.clone();

        let join_handle: JoinHandle<()> = thread::spawn(move || {
            // TODO: clean up Windows-specific module dependency
            let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();
            let mut application = VulkanApplication::new(moved_properties, state_receiver, mesh_receiver, keys_sender, buttons_sender, mouse_pos_sender, window_extent_sender, moved_is_closing).unwrap();
            event_loop.run_app(&mut application).unwrap();
        });

        let width = init_props.window_props.width;
        let height = init_props.window_props.height;

        Ok(
            Self {
                mesh_id_counter: 0,
                state_sender,
                mesh_sender,
                keys_down: create_empty_vk_map(),
                keys_pressed: create_empty_vk_map(),
                keys_released: create_empty_vk_map(),
                keys_receiver,
                buttons_down: create_empty_vb_map(),
                buttons_pressed: create_empty_vb_map(),
                buttons_released: create_empty_vb_map(),
                buttons_receiver,
                mouse_pos: None,
                mouse_pos_receiver,
                window_extent: vk::Extent2D { width, height },
                window_extent_receiver,
                is_closing,
                render_thread_join_handle: Some(join_handle),
            }
        )
    }

    fn sync_state(&mut self, state: RenderState) -> Result<()> {
        // Keys
        let mut new_keys_down = self.keys_down.clone();

        while let Ok((vk, vk_state)) = self.keys_receiver.try_recv() {
            match vk_state {
                VirtualElementState::Pressed => new_keys_down.insert(vk, true),
                VirtualElementState::Released => new_keys_down.insert(vk, false),
            };
        }

        VirtualKey::iter().for_each(|vk| {
            if vk != VirtualKey::Unknown {
                let was_down = *self.keys_down.get(&vk).unwrap_or_else(|| panic!("Invalid key {:?}", &vk));
                let is_down = *new_keys_down.get(&vk).unwrap_or_else(|| panic!("Invalid key {:?}", &vk));

                self.keys_pressed.insert(vk, !was_down && is_down);
                self.keys_released.insert(vk, was_down && !is_down);
            }
        });

        self.keys_down = new_keys_down;

        // Buttons
        let mut new_buttons_down = self.buttons_down.clone();

        while let Ok((vb, vb_state)) = self.buttons_receiver.try_recv() {
            match vb_state {
                VirtualElementState::Pressed => new_buttons_down.insert(vb, true),
                VirtualElementState::Released => new_buttons_down.insert(vb, false),
            };
        }

        VirtualButton::iter().for_each(|vb| {
            if vb != VirtualButton::Unknown {
                let was_down = *self.buttons_down.get(&vb).unwrap_or_else(|| panic!("Invalid button {:?}", &vb));
                let is_down = *new_buttons_down.get(&vb).unwrap_or_else(|| panic!("Invalid button {:?}", &vb));

                self.buttons_pressed.insert(vb, !was_down && is_down);
                self.buttons_released.insert(vb, was_down && !is_down);
            }
        });

        self.buttons_down = new_buttons_down;

        // Mouse position
        while let Ok(mouse_pos) = self.mouse_pos_receiver.try_recv() {
            self.mouse_pos = mouse_pos;
        }

        // Window extent
        while let Ok(window_extent) = self.window_extent_receiver.try_recv() {
            self.window_extent = window_extent;
        }

        // Render state
        Ok(self.state_sender.try_send(state)?)
    }

    fn get_window(&self) -> Result<&VulkanRenderEngine> {
        Ok(self)
    }

    fn get_window_mut(&mut self) -> Result<&mut VulkanRenderEngine> {
        Ok(self)
    }

    fn get_device(&self) -> Result<&VulkanRenderEngine> {
        Ok(self)
    }

    fn get_device_mut(&mut self) -> Result<&mut VulkanRenderEngine> {
        Ok(self)
    }

    fn join_render_thread(&mut self) -> Result<()> {
        self.is_closing.store(true, Ordering::SeqCst);

        if let Some(join_handle) = self.render_thread_join_handle.take() {
            join_handle.join().map_err(|_| anyhow!("Failed to join render thread!"))
        } else {
            Err(anyhow!("Already joined the render thread"))
        }
    }
}

fn create_empty_vk_map() -> HashMap<VirtualKey, bool> {
    let mut vk_map = HashMap::with_capacity(VirtualKey::COUNT);

    VirtualKey::iter().for_each(|vk| {
        if vk != VirtualKey::Unknown {
            vk_map.insert(vk, false);
        }
    });

    vk_map
}

fn create_empty_vb_map() -> HashMap<VirtualButton, bool> {
    let mut vb_map = HashMap::with_capacity(VirtualButton::COUNT);

    VirtualButton::iter().for_each(|vb| {
        if vb != VirtualButton::Unknown {
            vb_map.insert(vb, false);
        }
    });

    vb_map
}

impl Window for VulkanRenderEngine {
    fn get_width(&self) -> u32 {
        self.window_extent.width
    }

    fn get_height(&self) -> u32 {
        self.window_extent.height
    }

    fn is_key_down(&self, key: VirtualKey) -> bool {
        *self.keys_down.get(&key).unwrap_or_else(|| panic!("Invalid key {:?}", key))
    }

    fn is_key_pressed(&self, key: VirtualKey) -> bool {
        *self.keys_pressed.get(&key).unwrap_or_else(|| panic!("Invalid key {:?}", key))
    }

    fn is_key_released(&self, key: VirtualKey) -> bool {
        *self.keys_released.get(&key).unwrap_or_else(|| panic!("Invalid key {:?}", key))
    }

    fn is_button_down(&self, button: VirtualButton) -> bool {
        *self.buttons_down.get(&button).unwrap_or_else(|| panic!("Invalid button {:?}", button))
    }

    fn is_button_pressed(&self, button: VirtualButton) -> bool {
        *self.buttons_pressed.get(&button).unwrap_or_else(|| panic!("Invalid button {:?}", button))
    }

    fn is_button_released(&self, button: VirtualButton) -> bool {
        *self.buttons_released.get(&button).unwrap_or_else(|| panic!("Invalid button {:?}", button))
    }

    fn get_mouse_screen_position(&self) -> Option<&Vec2> {
        self.mouse_pos.as_ref()
    }

    fn is_closing(&self) -> bool {
        self.is_closing.load(Ordering::SeqCst)
    }
}

impl Device for VulkanRenderEngine {
    fn create_mesh(&mut self, vertices: Arc<Vec<Vertex>>, vertex_indexes: Arc<Vec<u32>>) -> Result<RenderMeshId> {
        if vertices.is_empty() || vertex_indexes.is_empty() {
            Err(anyhow!("Can't create a mesh with empty vertex data arrays"))
        } else if vertex_indexes.len() % 3 != 0 {
            Err(anyhow!("Can't create a mesh with an invalid number of indices"))
        } else {
            let mesh_id = RenderMeshId(self.mesh_id_counter);

            self.mesh_id_counter += 1;

            self.mesh_sender.send((mesh_id, vertices.clone(), vertex_indexes.clone()))?;

            Ok(mesh_id)
        }
    }
}

impl VulkanApplication {
    fn new(
        init_props: RenderEngineInitProps,
        state_receiver: Receiver<RenderState>,
        mesh_receiver: Receiver<(RenderMeshId, Arc<Vec<Vertex>>, Arc<Vec<u32>>)>,
        keys_sender: SyncSender<(VirtualKey, VirtualElementState)>,
        buttons_sender: SyncSender<(VirtualButton, VirtualElementState)>,
        mouse_pos_sender: SyncSender<Option<Vec2>>,
        window_extent_sender: SyncSender<vk::Extent2D>,
        is_closing: Arc<AtomicBool>,
    ) -> Result<Self> {
        Ok(
            Self {
                init_props,
                state_receiver,
                mesh_receiver,
                is_minimized: false,
                is_resized: false,
                is_closing,
                keys_sender,
                buttons_sender,
                mouse_pos_sender,
                window_extent_sender,
                context: None,
                swapchain_fences: (0..MAX_FRAMES_IN_FLIGHT).map(|_| vk::Fence::null()).collect(),
                frame: 0,
            }
        )
    }

    unsafe fn render(&mut self) -> Result<()> {
        match &mut self.context {
            Some(context) => {
                context.device.wait_for_fences(
                    &[context.sync_objects[self.frame].in_flight_fence],
                    true,
                    u64::MAX,
                )?;

                let result = context
                    .device
                    .acquire_next_image_khr(
                        context.swapchain.swapchain,
                        u64::MAX,
                        context.sync_objects[self.frame].image_available_semaphore,
                        vk::Fence::null(),
                    );

                let image_index = match result {
                    Ok((image_index, _)) => image_index as usize,
                    Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(),
                    Err(e) => return Err(anyhow!(e)),
                };

                if !self.swapchain_fences[image_index].is_null() {
                    context.device.wait_for_fences(
                        &[self.swapchain_fences[image_index]],
                        true,
                        u64::MAX,
                    )?;
                }

                self.swapchain_fences[image_index] = context.sync_objects[self.frame].in_flight_fence;

                // TODO: Is there a possible race condition here where a RenderState could be received referencing a mesh not yet received??
                //  Also, it prob does not even make sense to do the expensive pieces like buffer creation on the render thread...
                let render_state = self.state_receiver.recv()?;
                while let Ok((mesh_id, vertices, vertex_indexes)) = self.mesh_receiver.try_recv() {
                    let index_count = vertex_indexes.len();

                    let vertex_buffer = create_vertex_buffer(&context.vk_instance, &context.device, context.physical_device, context.single_time_command_pool, context.graphics_queue, vertices)?;
                    let index_buffer = create_index_buffer(&context.vk_instance, &context.device, context.physical_device, context.single_time_command_pool, context.graphics_queue, vertex_indexes)?;

                    let mesh = VulkanMesh {
                        mesh_id,
                        vertex_buffer,
                        index_buffer,
                        index_count,
                    };

                    context.meshes.insert(mesh.mesh_id, mesh);
                }

                let ubos = render_state.entity_states.iter().map(|e| {
                    UniformBufferObject {
                        world: e.world,
                        view: render_state.view,
                        proj: render_state.proj,
                        color: e.color,
                    }
                }).collect();
                update_uniforms(&context.device, context.uniform_buffers[image_index].memory, &ubos, context.ubo_alignment)?;

                context.update_command_buffer(image_index, render_state)?;

                let wait_semaphores = &[context.sync_objects[self.frame].image_available_semaphore];
                let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                let command_buffers = &[context.per_frame_command_buffers[image_index]];
                let signal_semaphores = &[context.sync_objects[self.frame].render_finished_semaphore];
                let submit_info = vk::SubmitInfo::builder()
                    .wait_semaphores(wait_semaphores)
                    .wait_dst_stage_mask(wait_stages)
                    .command_buffers(command_buffers)
                    .signal_semaphores(signal_semaphores);

                context.device.reset_fences(&[context.sync_objects[self.frame].in_flight_fence])?;

                context.device.queue_submit(
                    context.graphics_queue,
                    &[submit_info],
                    context.sync_objects[self.frame].in_flight_fence,
                )?;

                let swapchains = &[context.swapchain.swapchain];
                let image_indices = &[image_index as u32];
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(signal_semaphores)
                    .swapchains(swapchains)
                    .image_indices(image_indices);

                let result = context.device.queue_present_khr(context.present_queue, &present_info);

                let should_recreate_swapchain =
                    result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
                    || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR)
                    || self.is_resized;

                if should_recreate_swapchain {
                    self.is_resized = false;

                    self.recreate_swapchain()?;
                } else if let Err(e) = result {
                    return Err(anyhow!(e));
                }

                self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

                Ok(())
            },
            None => Err(anyhow!("No Vulkan context to render")),
        }
    }

    unsafe fn recreate_swapchain(&mut self) -> Result<()> {
        if let Some(context) = self.context.as_mut() {
            context.recreate_swapchain()?;

            self.window_extent_sender.send(context.swapchain.extent)?;

            Ok(())
        } else {
            Ok(())
        }
    }

    fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
        self.is_closing.store(true, Ordering::SeqCst);
        if let Some(c) = self.context.take() {
            unsafe { c.destroy().unwrap(); }
        }
        event_loop.exit();
    }
}

impl ApplicationHandler for VulkanApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_none() {
            let new_context = VulkanContext::new(self.init_props.clone(), event_loop);
            self.context = Some(new_context);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(context) = self.context.as_ref() {
            context.winit_window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.is_closing.load(Ordering::SeqCst) {
            self.shutdown(event_loop);
        } else {
            match event {
                WindowEvent::RedrawRequested if !self.is_minimized =>
                    unsafe { self.render() }.unwrap_or_else(|e| panic!("Internal render error: {}", e)),
                WindowEvent::Resized(size) => {
                    self.is_minimized = size.width == 0 || size.height == 0;
                    self.is_resized = true;
                },
                WindowEvent::CloseRequested => self.shutdown(event_loop),
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    let vk_state = get_virtual_state_for_winit_state(key_event.state);

                    let vk = get_vk_for_winit_physical_key(key_event.physical_key);
                    if vk != VirtualKey::Unknown {
                        self.keys_sender.send((vk, vk_state)).unwrap_or_else(|_| panic!("Failed to send physical key"));
                    }

                    let vk = get_vk_for_winit_logical_key(key_event.logical_key);
                    if vk != VirtualKey::Unknown {
                        self.keys_sender.send((vk, vk_state)).unwrap_or_else(|_| panic!("Failed to send logical key"));
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    let vb_state = get_virtual_state_for_winit_state(state);

                    let vb = get_vb_for_winit_mouse_button(button);
                    if vb != VirtualButton::Unknown {
                        self.buttons_sender.send((vb, vb_state)).unwrap_or_else(|_| panic!("Failed to send mouse button"));
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos_sender.send(Some(vec2(position.x as f32, position.y as f32)))
                        .unwrap_or_else(|_| panic!("Failed to send mouse position"));
                },
                WindowEvent::CursorLeft { .. } => {
                    self.mouse_pos_sender.send(None).unwrap_or_else(|_| panic!("Failed to send mouse position"));
                },
                _ => {},
            };
        }
    }
}

// User Input

const fn get_virtual_state_for_winit_state(state: ElementState) -> VirtualElementState {
    match state {
        ElementState::Pressed => VirtualElementState::Pressed,
        ElementState::Released => VirtualElementState::Released,
    }
}

const fn get_vk_for_winit_physical_key(key_code: PhysicalKey) -> VirtualKey {
    match key_code {
        PhysicalKey::Code(KeyCode::KeyA) => VirtualKey::A,
        PhysicalKey::Code(KeyCode::KeyB) => VirtualKey::B,
        PhysicalKey::Code(KeyCode::KeyC) => VirtualKey::C,
        PhysicalKey::Code(KeyCode::KeyD) => VirtualKey::D,
        PhysicalKey::Code(KeyCode::KeyE) => VirtualKey::E,
        PhysicalKey::Code(KeyCode::KeyF) => VirtualKey::F,
        PhysicalKey::Code(KeyCode::KeyG) => VirtualKey::G,
        PhysicalKey::Code(KeyCode::KeyH) => VirtualKey::H,
        PhysicalKey::Code(KeyCode::KeyI) => VirtualKey::I,
        PhysicalKey::Code(KeyCode::KeyJ) => VirtualKey::J,
        PhysicalKey::Code(KeyCode::KeyK) => VirtualKey::K,
        PhysicalKey::Code(KeyCode::KeyL) => VirtualKey::L,
        PhysicalKey::Code(KeyCode::KeyM) => VirtualKey::M,
        PhysicalKey::Code(KeyCode::KeyN) => VirtualKey::N,
        PhysicalKey::Code(KeyCode::KeyO) => VirtualKey::O,
        PhysicalKey::Code(KeyCode::KeyP) => VirtualKey::P,
        PhysicalKey::Code(KeyCode::KeyQ) => VirtualKey::Q,
        PhysicalKey::Code(KeyCode::KeyR) => VirtualKey::R,
        PhysicalKey::Code(KeyCode::KeyS) => VirtualKey::S,
        PhysicalKey::Code(KeyCode::KeyT) => VirtualKey::T,
        PhysicalKey::Code(KeyCode::KeyU) => VirtualKey::U,
        PhysicalKey::Code(KeyCode::KeyV) => VirtualKey::V,
        PhysicalKey::Code(KeyCode::KeyW) => VirtualKey::W,
        PhysicalKey::Code(KeyCode::KeyX) => VirtualKey::X,
        PhysicalKey::Code(KeyCode::KeyY) => VirtualKey::Y,
        PhysicalKey::Code(KeyCode::KeyZ) => VirtualKey::Z,
        PhysicalKey::Code(KeyCode::Escape) => VirtualKey::Escape,
        _ => VirtualKey::Unknown,
    }
}

fn get_vk_for_winit_logical_key(named_key: Key) -> VirtualKey {
    match named_key {
        Key::Named(NamedKey::Space) => VirtualKey::Space,
        Key::Named(NamedKey::Enter) => VirtualKey::Enter,
        Key::Named(NamedKey::ArrowUp) => VirtualKey::Up,
        Key::Named(NamedKey::ArrowLeft) => VirtualKey::Left,
        Key::Named(NamedKey::ArrowDown) => VirtualKey::Down,
        Key::Named(NamedKey::ArrowRight) => VirtualKey::Right,
        Key::Named(NamedKey::Escape) => VirtualKey::Escape,
        _ => VirtualKey::Unknown,
    }
}

fn get_vb_for_winit_mouse_button(button: MouseButton) -> VirtualButton {
    match button {
        MouseButton::Left => VirtualButton::Left,
        MouseButton::Right => VirtualButton::Right,
        MouseButton::Middle => VirtualButton::Middle,
        _ => VirtualButton::Unknown,
    }
}
