use anyhow::{anyhow, Result};
use core::panic;
use log::{debug, error, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use strum::IntoEnumIterator;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::{Device as vk_Device, Version};
use vulkanalia::vk::{self, DebugUtilsMessengerEXT, ExtDebugUtilsExtension, KhrSurfaceExtension, SurfaceKHR};
use vulkanalia::window as vk_window;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window as winit_Window, WindowAttributes};

use crate::math::Vec3;
use crate::render_engine::{Device, MeshId, RenderEngine, RenderEngineInitProps, RenderState, VirtualKey, Window};

const VULKAN_PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_LAYER_NAME: vk::ExtensionName = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const REQUIRED_DEVICE_EXTENSION_NAMES: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const MAX_FRAMES_IN_FLIGHT: usize = 3; // TODO: rename for triple buffer approach

pub struct VulkanRenderEngine {
    render_thread_join_handle: Option<JoinHandle<()>>,
}

struct VulkanApplication {
    init_properties: RenderEngineInitProps,
    is_minimized: bool,
    is_closing: bool,
    keys_down: HashMap<VirtualKey, bool>,
    // TODO: mesh id to mesh mapping
    context: Option<VulkanContext>,
}

struct VulkanContext {
    winit_window: winit_Window,
    vk_instance: Instance,
    surface: SurfaceKHR,

    debug_messenger: Option<DebugUtilsMessengerEXT>,
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

    fn sync_state(&mut self, state: RenderState) {
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

    unsafe fn join_render_thread(mut self) -> Result<()> {
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
    }
}

impl Drop for VulkanRenderEngine {
    fn drop(&mut self) {
        todo!() // TODO
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
        let loader = unsafe { LibloadingLoader::new(LIBRARY).unwrap_or_else(|_| panic!("Failed to create loader for {}", LIBRARY)) };
        let entry = unsafe { Entry::new(loader) }.unwrap_or_else(|_| panic!("Failed to load entry point for {}", LIBRARY));

        let win_properties = &self.init_properties.window_props;
        let window_attribs = WindowAttributes::default()
            .with_title(win_properties.title.clone())
            .with_inner_size(LogicalSize::new(win_properties.width, win_properties.height))
            .with_resizable(false);
        let winit_window = event_loop.create_window(window_attribs).unwrap_or_else(|_| panic!("Failed to create winit window!"));

        let (vk_instance, debug_messenger) = unsafe { create_vk_instance(&winit_window, &entry, self.init_properties.debug_enabled) }
            .unwrap_or_else(|_| panic!("Failed to create Vulkan instance!"));

        let surface = unsafe { vk_window::create_surface(&vk_instance, &winit_window, &winit_window) }.unwrap_or_else(|_| panic!("Failed to create surface"));

        let context = VulkanContext {
            winit_window,
            vk_instance,
            surface,
            debug_messenger,
        };

        self.context = Some(context);
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

unsafe fn create_vk_instance(window: &winit_Window, entry: &Entry, debug_enabled: bool) -> Result<(Instance, Option<vk::DebugUtilsMessengerEXT>)> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"My Cool Game\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Hurt Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let flags = if cfg!(target_os = "macos") && entry.version()? >= VULKAN_PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");

        extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());

        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();
    let mut enabled_layers: Vec<*const i8> = Vec::new();

    if debug_enabled {
        if !available_layers.contains(&VALIDATION_LAYER_NAME) {
            return Err(anyhow!("Validation layers are enabled but not supported."));
        }

        enabled_layers.push(VALIDATION_LAYER_NAME.as_ptr());
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&enabled_layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if debug_enabled {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    let debug_messenger = if debug_enabled {
        Some(instance.create_debug_utils_messenger_ext(&debug_info, None)?)
    } else {
        None
    };

    Ok((instance, debug_messenger))
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

// TODO
// unsafe fn create_swapchain(
//     window: &winit_Window,
//     instance: &Instance,
//     device: &vk_Device,
// ) -> Result<()> {
//     let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;
//     let support = SwapchainSupport::get(instance, data, data.physical_device)?;

//     let surface_format = get_swapchain_surface_format(&support.formats);
//     let present_mode = get_swapchain_present_mode(&support.present_modes);
//     let extent = get_swapchain_extent(window, support.capabilities);

//     // Recommended to use at least one more image than the min so we shouldn't have to sometimes wait for the driver to acquire another image to render to
//     let mut image_count = support.capabilities.min_image_count + 1;
//     if support.capabilities.max_image_count != 0 // 0 means there is no maximum
//         && image_count > support.capabilities.max_image_count
//     {
//         image_count = support.capabilities.max_image_count;
//     }

//     let mut queue_family_indices = vec![];
//     let image_sharing_mode = if indices.graphics != indices.present {
//         // At least two distinct queue families are required for CONCURRENT MODE
//         queue_family_indices.push(indices.graphics);
//         queue_family_indices.push(indices.present);
//         vk::SharingMode::CONCURRENT
//     } else {
//         // Better performance than CONCURRENT mode when ownership doesn't need to be transferred between queue families,
//         // i.e. when there is only one queue family for both graphics and presenting
//         vk::SharingMode::EXCLUSIVE
//     };

//     let info = vk::SwapchainCreateInfoKHR::builder()
//         .surface(data.surface)
//         .min_image_count(image_count)
//         .image_format(surface_format.format)
//         .image_color_space(surface_format.color_space)
//         .image_extent(extent)
//         .image_array_layers(1) // Always 1 unless you're developing a stereoscopic 3D application
//         .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT) // Use a different bitmask like TRANSFER_DST when not rendering directly to the screen first, i.e. image post-processing
//         .image_sharing_mode(image_sharing_mode)
//         .queue_family_indices(&queue_family_indices)
//         .pre_transform(support.capabilities.current_transform) // Indicates we don't want any transforms applied to images, e.g. 90 degree rotation or horizontal flip
//         .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // Indicates no blending with other windows in the window system (usually don't want to do this....) https://docs.rs/vulkanalia/0.22.0/vulkanalia/vk/struct.CompositeAlphaFlagsKHR.html#associatedconstant.OPAQUE
//         .present_mode(present_mode)
//         .clipped(true) // better performance, pretty much always true unless you want to sample this for a different window in front of this one
//         .old_swapchain(vk::SwapchainKHR::null()); // Default is null anyway, only needed if current swapchain is invalid or unoptimized, and needs to be recreated from an old one (complex)

//     data.swapchain_format = surface_format.format;
//     data.swapchain_extent = extent;
//     data.swapchain = device.create_swapchain_khr(&info, None)?;
//     data.swapchain_images = device.get_swapchain_images_khr(data.swapchain)?;

//     // TODO asdf
//     data.swapchain_image_views = data
//         .swapchain_images
//         .iter()
//         .map(|i| create_image_view(device, *i, data.swapchain_format, vk::ImageAspectFlags::COLOR, 1))
//         .collect::<Result<Vec<_>, _>>()?;

//     Ok(())
// }

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

unsafe fn get_queue_family_indices(
    instance: &Instance,
    surface: SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<(usize, usize)> {
    let properties = instance.get_physical_device_queue_family_properties(physical_device);

    let graphics = properties.iter()
        .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS));

    let present = properties.iter().enumerate().map(|(i, _)| i).find(|i|
        instance.get_physical_device_surface_support_khr(physical_device, *i as u32, surface)
            .unwrap_or_else(|_| panic!("get_physical_device_surface_support_khr failed"))
    );

    if let (Some(graphics), Some(present)) = (graphics, present) {
        Ok((graphics, present))
    } else {
        Err(anyhow!("Missing required queue families"))
    }
}

unsafe fn create_image_view(
    device: &vk_Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(0)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}
