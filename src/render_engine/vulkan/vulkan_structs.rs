use vulkanalia::vk;

use crate::core::Color;
use crate::math::Mat4;
use crate::render_engine::RenderMeshId;

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct BufferResources {
    pub(in crate::render_engine::vulkan) buffer: vk::Buffer,
    pub(in crate::render_engine::vulkan) memory: vk::DeviceMemory,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct FrameSyncObjects {
    pub(in crate::render_engine::vulkan) image_available_semaphore: vk::Semaphore,
    pub(in crate::render_engine::vulkan) render_finished_semaphore: vk::Semaphore,
    pub(in crate::render_engine::vulkan) in_flight_fence: vk::Fence,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct ImageResources {
    pub(in crate::render_engine::vulkan) image: vk::Image,
    pub(in crate::render_engine::vulkan) memory: vk::DeviceMemory,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct VulkanMesh {
    pub(in crate::render_engine::vulkan) mesh_id: RenderMeshId,
    pub(in crate::render_engine::vulkan) vertex_buffer: BufferResources,
    pub(in crate::render_engine::vulkan) index_buffer: BufferResources,
    pub(in crate::render_engine::vulkan) index_count: usize,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct Pipeline {
    pub(in crate::render_engine::vulkan) pipeline: vk::Pipeline,
    pub(in crate::render_engine::vulkan) layout: vk::PipelineLayout,
}

#[derive(Copy, Clone, Debug)]
pub(in crate::render_engine::vulkan) struct QueueFamilyIndices {
    pub(in crate::render_engine::vulkan) graphics: u32,
    pub(in crate::render_engine::vulkan) present: u32,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct Swapchain {
    pub(in crate::render_engine::vulkan) format: vk::Format,
    pub(in crate::render_engine::vulkan) extent: vk::Extent2D,
    pub(in crate::render_engine::vulkan) swapchain: vk::SwapchainKHR,
    pub(in crate::render_engine::vulkan) images: Vec<vk::Image>,
    pub(in crate::render_engine::vulkan) image_views: Vec<vk::ImageView>,
}

#[derive(Clone, Debug)]
pub(in crate::render_engine::vulkan) struct SwapchainSupport {
    pub(in crate::render_engine::vulkan) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(in crate::render_engine::vulkan) formats: Vec<vk::SurfaceFormatKHR>,
    pub(in crate::render_engine::vulkan) present_modes: Vec<vk::PresentModeKHR>,
}

// TODO: split this into per-frame and per-entity uniform objects, and others?
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(in crate::render_engine::vulkan) struct UniformBufferObject {
    pub(in crate::render_engine::vulkan) world: Mat4,
    pub(in crate::render_engine::vulkan) view: Mat4,
    pub(in crate::render_engine::vulkan) proj: Mat4,
    pub(in crate::render_engine::vulkan) color: Color,
}

impl UniformBufferObject {
    pub(in crate::render_engine::vulkan) fn get_offset_alignment(min_offset_alignment: usize) -> usize {
        let ubo_size = size_of::<UniformBufferObject>();

        if ubo_size % min_offset_alignment == 0 {
            ubo_size
        } else {
            ubo_size + (min_offset_alignment - ubo_size % min_offset_alignment)
        }
    }
}
