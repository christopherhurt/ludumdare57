use anyhow::{anyhow, Result};
use log::{debug, error, trace, warn};
use std::ffi::CStr;
use std::os::raw::c_void;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::KhrSurfaceExtension;

use crate::render_engine::vulkan::vulkan_structs::{QueueFamilyIndices, SwapchainSupport};

pub(in crate::render_engine::vulkan) unsafe fn get_queue_family_indices(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<QueueFamilyIndices> {
    let properties = instance.get_physical_device_queue_family_properties(physical_device);

    let graphics = properties.iter()
        .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS));

    let present = properties.iter().enumerate().map(|(i, _)| i).find(|i|
        instance.get_physical_device_surface_support_khr(physical_device, *i as u32, surface)
            .unwrap_or_else(|_| panic!("get_physical_device_surface_support_khr failed"))
    );

    if let (Some(graphics), Some(present)) = (graphics, present) {
        Ok(
            QueueFamilyIndices {
                graphics: graphics as u32,
                present: present as u32,
            }
        )
    } else {
        Err(anyhow!("Missing required queue families"))
    }
}

pub(in crate::render_engine::vulkan) unsafe fn get_swapchain_support(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<SwapchainSupport> {
    Ok(
        SwapchainSupport {
            capabilities: instance.get_physical_device_surface_capabilities_khr(physical_device, surface)?,
            formats: instance.get_physical_device_surface_formats_khr(physical_device, surface)?,
            present_modes: instance.get_physical_device_surface_present_modes_khr(physical_device, surface)?,
        }
    )
}

unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(physical_device, *f);

            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}

pub(in crate::render_engine::vulkan) unsafe fn get_depth_format(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<vk::Format> {
    let candidates = &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT];

    get_supported_format(instance, physical_device, candidates, vk::ImageTiling::OPTIMAL, vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
}

pub(in crate::render_engine::vulkan) extern "system" fn debug_callback(
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
