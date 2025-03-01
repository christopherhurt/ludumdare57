use std::sync::Arc;

use crate::ecs::component::Component;
use crate::render_engine::MeshId;
use crate::render_engine::vulkan::VulkanRenderEngine;

// Mesh

pub struct Mesh {
    pub id: Arc<MeshId>,
}

impl Component for Mesh {}

// Vulkan Component

pub struct VulkanComponent {
    pub render_engine: VulkanRenderEngine,
}

impl Component for VulkanComponent {}
