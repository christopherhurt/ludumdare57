use std::sync::Arc;

use crate::ecs::component::Component;
use crate::render_engine::MeshId;
use crate::render_engine::vulkan::VulkanRenderEngine;

// Mesh

pub struct Mesh {
    pub id: Arc<MeshId>,
}

impl Mesh {
    pub fn new(id: Arc<MeshId>) -> Self {
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
