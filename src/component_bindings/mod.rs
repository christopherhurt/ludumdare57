use crate::ecs::ECSActions;
use crate::ecs::component::Component;
use crate::render_engine::MeshId;
use crate::render_engine::vulkan::VulkanRenderEngine;

// TODO: move these bindings to the render module? and just accept the fact that other modules like physics will have a hard dependency on the ecs module/components...

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
impl ECSActions for Mesh {}

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
impl ECSActions for VulkanComponent {}
