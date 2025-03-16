use anyhow::{anyhow, Result};
use winit::platform::windows::EventLoopBuilderExtWindows;
use core::panic;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use vulkanalia::Device as vk_Device;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension, SurfaceKHR};
use vulkanalia::window as vk_window;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window as winit_Window, WindowAttributes};

use crate::core::Color;
use crate::render_engine::{Device, MeshId, RenderEngine, RenderEngineInitProps, RenderState, Vertex, VirtualKey, Window};
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
use crate::render_engine::vulkan::vulkan_structs::{BufferResources, FrameSyncObjects, ImageResources, Mesh, Pipeline, Swapchain, UniformBufferObject};

mod vulkan_resources;
mod vulkan_structs;
mod vulkan_utils;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct VulkanRenderEngine {
    init_props: RenderEngineInitProps,
    mesh_id_counter: usize,
    state_sender: SyncSender<RenderState>,
    mesh_sender: Sender<(MeshId, Vec<Vertex>, Vec<u32>)>,
    keys_down_mirror: Option<HashMap<VirtualKey, bool>>,
    keys_receiver: Receiver<HashMap<VirtualKey, bool>>,
    is_closing: Arc<AtomicBool>,
    render_thread_join_handle: Option<JoinHandle<()>>,
}

struct VulkanApplication {
    init_props: RenderEngineInitProps,
    state_receiver: Receiver<RenderState>,
    mesh_receiver: Receiver<(MeshId, Vec<Vertex>, Vec<u32>)>,
    is_minimized: bool,
    is_closing: Arc<AtomicBool>,
    keys_down: HashMap<VirtualKey, bool>,
    keys_sender: SyncSender<HashMap<VirtualKey, bool>>,
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
    descriptor_sets: Vec<vk::DescriptorSet>,
    per_frame_command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: Vec<FrameSyncObjects>,

    meshes: HashMap<MeshId, Mesh>,
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
            .with_resizable(false);
        let winit_window = event_loop.create_window(window_attribs).unwrap_or_else(|_| panic!("Failed to create winit window!"));

        unsafe {
            let (vk_instance, debug_messenger) = create_vk_instance(&winit_window, &entry, init_properties.debug_enabled)
                .unwrap_or_else(|_| panic!("Failed to create Vulkan instance!"));

            let surface = vk_window::create_surface(&vk_instance, &winit_window, &winit_window).unwrap_or_else(|_| panic!("Failed to create surface"));

            let physical_device = pick_physical_device(&vk_instance, surface).unwrap_or_else(|e| panic!("{}", e));
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
            let uniform_buffers = (0..swapchain.images.len()).map(|_| create_uniform_buffer::<UniformBufferObject>(&vk_instance, &device, physical_device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let descriptor_pool = create_descriptor_pool(&device, swapchain.images.len()).unwrap_or_else(|e| panic!("{}", e));
            let descriptor_sets = create_descriptor_sets(&device, descriptor_set_layout, descriptor_pool, &uniform_buffers).unwrap_or_else(|e| panic!("{}", e));
            let per_frame_command_buffers = per_frame_command_pools.iter().map(|p| create_command_buffer(&device, *p).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let sync_objects = (0..MAX_FRAMES_IN_FLIGHT).map(|_| create_sync_objects(&device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();

            Self {
                clear_color: init_properties.clear_color,

                winit_window,
                _entry: entry,
                vk_instance,
                surface,
                debug_messenger,

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
        self.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.layout,
            0,
            &[self.descriptor_sets[image_index]],
            &[],
        );

        // TODO: do this in a way that doesn't involve a double copy of RenderState? (i.e. serializing to RenderState, then copying RenderState to a buffer)
        //  Also, we don't want to all uniforms every entity... only the per-entity uniforms
        for e in state.entity_states.iter() {
            let ubo = UniformBufferObject {
                world: e.world,
                view: state.view,
                proj: state.proj,
                color: e.color,
            };

            update_uniforms(&self.device, self.uniform_buffers[image_index].memory, ubo)?;

            let mesh = self.meshes.get(&e.mesh_id).unwrap_or_else(|| panic!("No mesh exists for ID {}", e.mesh_id.0));

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
        self.uniform_buffers = (0..self.swapchain.images.len()).map(|_| create_uniform_buffer::<UniformBufferObject>(&self.vk_instance, &self.device, self.physical_device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
        self.descriptor_pool = create_descriptor_pool(&self.device, self.swapchain.images.len()).unwrap_or_else(|e| panic!("{}", e));
        self.descriptor_sets = create_descriptor_sets(&self.device, self.descriptor_set_layout, self.descriptor_pool, &self.uniform_buffers).unwrap_or_else(|e| panic!("{}", e));
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
    ubo: UniformBufferObject,
) -> Result<()> {
    let memory = device.map_memory(
        uniform_memory,
        0,
        size_of::<UniformBufferObject>() as u64,
        vk::MemoryMapFlags::empty(),
    )?;

    std::ptr::copy_nonoverlapping(&ubo, memory.cast(), 1);

    device.unmap_memory(uniform_memory);

    Ok(())
}

impl RenderEngine<VulkanRenderEngine, VulkanRenderEngine> for VulkanRenderEngine {
    unsafe fn new(init_props: RenderEngineInitProps) -> Result<Self> {
        // TODO: update this so we can overwrite the buffered state(s), rather than block the sender, if the sender gets ahead of the receiver
        let (state_sender, state_receiver) = mpsc::sync_channel::<RenderState>(1);
        let (mesh_sender, mesh_receiver) = mpsc::channel();
        let (keys_sender, keys_receiver) = mpsc::sync_channel::<HashMap<VirtualKey, bool>>(1);

        let is_closing = Arc::new(AtomicBool::new(false));

        let moved_properties = init_props.clone();
        let moved_is_closing = is_closing.clone();

        let join_handle: JoinHandle<()> = thread::spawn(move || {
            // TODO: clean up Windows-specific module dependency
            let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();
            let mut application = VulkanApplication::new(moved_properties, state_receiver, mesh_receiver, keys_sender, moved_is_closing).unwrap();
            event_loop.run_app(&mut application).unwrap();
        });

        Ok(
            Self {
                init_props,
                mesh_id_counter: 0,
                state_sender,
                mesh_sender,
                keys_down_mirror: None,
                keys_receiver,
                is_closing,
                render_thread_join_handle: Some(join_handle),
            }
        )
    }

    fn sync_state(&mut self, state: RenderState) -> Result<()> {
        if let Ok(keys_down) = self.keys_receiver.try_recv() {
            self.keys_down_mirror = Some(keys_down);
        }

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

    unsafe fn join_render_thread(&mut self) -> Result<()> {
        // TODO: issue here - this doesn't actually trigger a window close, so the join below just hangs
        self.is_closing.store(true, Ordering::SeqCst);

        if let Some(join_handle) = self.render_thread_join_handle.take() {
            join_handle.join().map_err(|_| anyhow!("Failed to join render thread!"))
        } else {
            Err(anyhow!("Already joined the render thread"))
        }
    }
}

impl Window for VulkanRenderEngine {
    fn get_width(&self) -> u32 {
        // TODO: need to get this dynamically when resizing is allowed
        self.init_props.window_props.width
    }

    fn get_height(&self) -> u32 {
        // TODO: need to get this dynamically when resizing is allowed
        self.init_props.window_props.height
    }

    fn is_key_down(&self, key: VirtualKey) -> bool {
        match &self.keys_down_mirror {
            Some(keys_down) => *keys_down.get(&key).unwrap_or_else(|| panic!("Invalid key {:?}", key)),
            None => false,
        }
    }

    fn is_closing(&self) -> bool {
        self.is_closing.load(Ordering::SeqCst)
    }
}

impl Device for VulkanRenderEngine {
    unsafe fn create_mesh(&mut self, vertices: Vec<Vertex>, vertex_indexes: Vec<u32>) -> Result<MeshId> {
        if vertices.is_empty() || vertex_indexes.is_empty() {
            Err(anyhow!("Can't create a mesh with empty vertex data arrays"))
        } else if vertex_indexes.len() % 3 != 0 {
            Err(anyhow!("Can't create a mesh with an invalid number of indices"))
        } else {
            let mesh_id = MeshId(self.mesh_id_counter);

            self.mesh_id_counter += 1;

            self.mesh_sender.send((mesh_id, vertices, vertex_indexes))?;

            Ok(mesh_id)
        }
    }
}

impl VulkanApplication {
    fn new(
        init_props: RenderEngineInitProps,
        state_receiver: Receiver<RenderState>,
        mesh_receiver: Receiver<(MeshId, Vec<Vertex>, Vec<u32>)>,
        keys_sender: SyncSender<HashMap<VirtualKey, bool>>,
        is_closing: Arc<AtomicBool>,
    ) -> Result<Self> {
        let mut keys_down = HashMap::new();
        VirtualKey::iter().for_each(|vk| {
            if vk != VirtualKey::Unknown {
                keys_down.insert(vk, false);
            }
        });

        Ok(
            Self {
                init_props,
                state_receiver,
                mesh_receiver,
                is_minimized: false,
                is_closing,
                keys_down,
                keys_sender,
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
                    Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return context.recreate_swapchain(),
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
                    let vertex_buffer = create_vertex_buffer(&context.vk_instance, &context.device, context.physical_device, context.single_time_command_pool, context.graphics_queue, &vertices)?;
                    let index_buffer = create_index_buffer(&context.vk_instance, &context.device, context.physical_device, context.single_time_command_pool, context.graphics_queue, &vertex_indexes)?;

                    let mesh = Mesh {
                        mesh_id,
                        vertex_buffer,
                        index_buffer,
                        index_count: vertex_indexes.len(),
                    };

                    context.meshes.insert(mesh.mesh_id, mesh);
                }
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

                let should_recreate_swapchain = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

                if should_recreate_swapchain {
                    context.recreate_swapchain()?;
                } else if let Err(e) = result {
                    return Err(anyhow!(e));
                }

                self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

                Ok(())
            },
            None => Err(anyhow!("No Vulkan context to render")),
        }
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
        match event {
            WindowEvent::RedrawRequested if !self.is_closing.load(Ordering::SeqCst) && !self.is_minimized =>
                unsafe { self.render() }.unwrap_or_else(|e| panic!("Internal render error: {}", e)),
            WindowEvent::Resized(size) =>
                self.is_minimized = size.width == 0 || size.height == 0,
            WindowEvent::CloseRequested => {
                self.is_closing.store(true, Ordering::SeqCst);
                if let Some(c) = self.context.take() {
                    unsafe { c.destroy().unwrap(); }
                }
                event_loop.exit();
            },
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let is_down = get_is_key_down_for_state(key_event.state);

                let vk = get_vk_for_winit_physical_key(key_event.physical_key);
                if vk != VirtualKey::Unknown {
                    self.keys_down.insert(vk, is_down).unwrap_or_else(||
                        panic!("Internal error: key {:?} was not initialized in keys_down map!", vk));
                }

                let vk = get_vk_for_winit_logical_key(key_event.logical_key);
                if vk != VirtualKey::Unknown {
                    self.keys_down.insert(vk, is_down).unwrap_or_else(||
                        panic!("Internal error: key {:?} was not initialized in keys_down map!", vk));
                }

                // TODO: prob don't need to copy this for every input
                self.keys_sender.send(self.keys_down.clone()).unwrap_or_default();
            },
            _ => {},
        };
    }
}

// User Input

const fn get_is_key_down_for_state(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
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
        _ => VirtualKey::Unknown,
    }
}

fn get_vk_for_winit_logical_key(named_key: Key) -> VirtualKey {
    match named_key {
        Key::Named(NamedKey::Space) => VirtualKey::Space,
        Key::Named(NamedKey::ArrowUp) => VirtualKey::Up,
        Key::Named(NamedKey::ArrowLeft) => VirtualKey::Left,
        Key::Named(NamedKey::ArrowDown) => VirtualKey::Down,
        Key::Named(NamedKey::ArrowRight) => VirtualKey::Right,
        _ => VirtualKey::Unknown,
    }
}
