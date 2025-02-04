use anyhow::{anyhow, Result};
use winit::event::ElementState;
use core::panic;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use vulkanalia::Version;
use vulkanalia::vk;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

use crate::core::{Mesh, Scene};
use crate::math::Vec3;
use crate::render_engine::{Device, RenderEngine, RenderEngineInitProperties, VirtualKey, Window};

const VULKAN_PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_LAYER_NAME: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const REQUIRED_DEVICE_EXTENSION_NAMES: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct VulkanRenderEngine {
    render_thread_join_handle: Option<JoinHandle<()>>,
}

struct VulkanApplication {
    init_properties: RenderEngineInitProperties,
    is_minimized: bool,
    is_closing: bool,
    keys_down: HashMap<VirtualKey, bool>,
    // TODO: mesh id to mesh mapping
}

impl VulkanRenderEngine {
    pub fn new(init_properties: RenderEngineInitProperties) -> Result<Self> {
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
    fn sync_data(&mut self, scene: &Scene) -> anyhow::Result<()> {
        todo!() // TODO
    }

    fn get_window(&self) -> &VulkanRenderEngine {
        self
    }

    fn get_window_mut(&mut self) -> &mut VulkanRenderEngine {
        self
    }

    fn get_device(&self) -> &VulkanRenderEngine {
        self
    }

    fn get_device_mut(&mut self) -> &mut VulkanRenderEngine {
        self
    }

    fn join_render_thread(&mut self) -> Result<()> {
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
    fn create_mesh(&mut self, vertex_positions: Vec<Vec3>, vertex_indexes: Option<Vec<usize>>) -> Result<Rc<Mesh>> {
        todo!() // TODO
    }
}

impl Drop for VulkanRenderEngine {
    fn drop(&mut self) {
        todo!() // TODO
    }
}

impl VulkanApplication {
    fn new(init_properties: RenderEngineInitProperties) -> Result<Self> {
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
        // TODO: create window, graphics context here on first event
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId, // Need to use this if we ever have multiple windows
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
                // TODO: self.device.device_wait_idle().unwrap()
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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct Vertex {
    pos: Vec3,
}
