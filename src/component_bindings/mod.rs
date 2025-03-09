use crate::ecs::component::Component;
use crate::render_engine::MeshId;
use crate::render_engine::vulkan::VulkanRenderEngine;

// Mesh

pub struct Mesh {
    pub id: MeshId,
}

impl Mesh {
    pub fn new(id: MeshId) -> Self {
        Self { id }
    }
}

impl Component for Mesh {}

// Vulkan Component

pub struct VulkanComponent {
    pub render_engine: VulkanRenderEngine,
}

impl VulkanComponent {
    pub fn new(render_engine: VulkanRenderEngine) -> Self {
        Self { render_engine }
    }
}

impl Component for VulkanComponent {}
