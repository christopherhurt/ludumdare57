use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

use crate::math::Vec3;

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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(in crate::render_engine::vulkan) struct Vertex {
    pub(in crate::render_engine::vulkan) pos: Vec3,
}

impl Vertex {
    pub(in crate::render_engine::vulkan) fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub(in crate::render_engine::vulkan) fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 1] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        [pos]
    }
}
