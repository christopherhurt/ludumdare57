use anyhow::{anyhow, Result};
use core::panic;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use vulkanalia::Device as vk_Device;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{DebugUtilsMessengerEXT, SurfaceKHR};
use vulkanalia::window as vk_window;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window as winit_Window, WindowAttributes};

use crate::math::Vec3;
use crate::render_engine::{Device, MeshId, RenderEngine, RenderEngineInitProps, RenderState, VirtualKey, Window};
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
};
use crate::render_engine::vulkan::vulkan_structs::{BufferResources, FrameSyncObjects, ImageResources, Mesh, Pipeline, Swapchain, UniformBufferObject};

mod vulkan_resources;
mod vulkan_structs;
mod vulkan_utils;

const MAX_FRAMES_IN_FLIGHT: usize = 3; // TODO: rename for triple buffer approach

pub struct VulkanRenderEngine {
    render_thread_join_handle: Option<JoinHandle<()>>,
}

struct VulkanApplication {
    init_properties: RenderEngineInitProps,
    is_minimized: bool,
    is_closing: bool,
    keys_down: HashMap<VirtualKey, bool>,
    context: Option<VulkanContext>,
}

struct VulkanContext {
    winit_window: winit_Window,
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
    uniform_buffer: BufferResources,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    per_frame_command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: Vec<FrameSyncObjects>,

    meshes: HashMap<MeshId, Mesh>,
}

impl VulkanContext {
    fn new(init_properties: RenderEngineInitProps, event_loop: &ActiveEventLoop) -> Self {
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
            // TODO: split up into more than one uniform buffer
            let uniform_buffer = create_uniform_buffer::<UniformBufferObject>(&vk_instance, &device, physical_device).unwrap_or_else(|e| panic!("{}", e));
            let descriptor_pool = create_descriptor_pool(&device, swapchain.images.len()).unwrap_or_else(|e| panic!("{}", e));
            let descriptor_sets = create_descriptor_sets(&device, swapchain.images.len(), descriptor_set_layout, descriptor_pool).unwrap_or_else(|e| panic!("{}", e));
            let per_frame_command_buffers = per_frame_command_pools.iter().map(|p| create_command_buffer(&device, *p).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();
            let sync_objects = (0..swapchain.images.len()).map(|_| create_sync_objects(&device).unwrap_or_else(|e| panic!("{}", e))).collect::<Vec<_>>();

            Self {
                winit_window,
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
                uniform_buffer,
                descriptor_pool,
                descriptor_sets,
                per_frame_command_buffers,
                sync_objects,

                meshes: HashMap::new(),
            }
        }
    }
}

impl VulkanRenderEngine {
    pub fn new(init_properties: RenderEngineInitProps) -> Result<Self> {
        let join_handle = thread::spawn(|| {
            let event_loop = EventLoop::new().unwrap();
            let mut application = VulkanApplication::new(init_properties).unwrap();
            event_loop.run_app(&mut application).unwrap();
        });

        Ok(
            Self {
                render_thread_join_handle: Some(join_handle),
            }
        )
    }
}

impl RenderEngine<VulkanRenderEngine, VulkanRenderEngine> for VulkanRenderEngine {
    unsafe fn new(init_props: RenderEngineInitProps) -> Self {
        todo!() // TODO
    }

    fn sync_state(&mut self, state: RenderState) -> Result<()> {
        todo!() // TODO
    }

    fn get_window(&self) -> Result<&VulkanRenderEngine> {
        todo!() // TODO
    }

    fn get_window_mut(&mut self) -> Result<&mut VulkanRenderEngine> {
        todo!() // TODO
    }

    fn get_device(&self) -> Result<&VulkanRenderEngine> {
        todo!() // TODO
    }

    fn get_device_mut(&mut self) -> Result<&mut VulkanRenderEngine> {
        todo!() // TODO
    }

    unsafe fn join_render_thread(&mut self) -> Result<()> {
        // TODO: trigger shutdown
        if let Some(join_handle) = self.render_thread_join_handle.take() {
            join_handle.join().map_err(|_| anyhow!("Failed to join render thread!"))
        } else {
            Err(anyhow!("Already joined the render thread"))
        }
    }
}

impl Window for VulkanRenderEngine {
    fn get_width(&self) -> Result<u32> {
        todo!() // TODO
    }

    fn get_height(&self) -> Result<u32> {
        todo!() // TODO
    }

    fn is_key_down(&self, key: VirtualKey) -> Result<bool> {
        todo!() // TODO
    }

    fn is_closing(&self) -> bool {
        todo!() // TODO
    }
}

impl Device for VulkanRenderEngine {
    unsafe fn create_mesh(&mut self, vertex_positions: Vec<Vec3>, vertex_indexes: Option<Vec<usize>>) -> Result<Arc<MeshId>> {
        todo!() // TODO
        // create_vertex_buffer(&instance, &device, &mut data)?;
        // create_index_buffer(&instance, &device, &mut data)?;
    }
}

impl VulkanApplication {
    fn new(init_properties: RenderEngineInitProps) -> Result<Self> {
        let mut keys_down = HashMap::new();
        VirtualKey::iter().for_each(|vk| {
            if vk != VirtualKey::Unknown {
                keys_down.insert(vk, false);
            }
        });

        Ok(
            Self {
                init_properties,
                is_minimized: false,
                is_closing: false,
                keys_down,
                context: None,
            }
        )
    }

    unsafe fn render(&mut self) -> Result<()> {
        // TODO
        Ok(())
    }
}

impl ApplicationHandler for VulkanApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.context = Some(VulkanContext::new(self.init_properties.clone(), event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested if !self.is_closing && !self.is_minimized =>
                unsafe { self.render() }.unwrap(),
            WindowEvent::Resized(size) =>
                self.is_minimized = size.width == 0 || size.height == 0,
            WindowEvent::CloseRequested => {
                self.is_closing = true;
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
