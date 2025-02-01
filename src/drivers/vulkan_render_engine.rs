use anyhow::Result;
use std::rc::Rc;

use crate::core::{Mesh, Scene};
use crate::math::Vec3;
use crate::render_engine::{Device, Window, RenderEngine, RenderEngineProperties, VirtualKey};

pub struct VulkanRenderEngine {
    window: VulkanWindow,
    device: VulkanDevice,
}

pub struct VulkanWindow {}

pub struct VulkanDevice {
    // TODO: mesh id to mesh mapping
}

impl VulkanRenderEngine {
    pub fn new(properties: &RenderEngineProperties) -> Result<Self> {
        todo!() // TODO
    }
}

impl RenderEngine<VulkanWindow, VulkanDevice> for VulkanRenderEngine {
    fn sync_data(&mut self, scene: &Scene) -> anyhow::Result<()> {
        todo!() // TODO
    }

    fn get_window(&self) -> &VulkanWindow {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut VulkanWindow {
        &mut self.window
    }

    fn get_device(&self) -> &VulkanDevice {
        &self.device
    }

    fn get_device_mut(&mut self) -> &mut VulkanDevice {
        &mut self.device
    }
}

impl Drop for VulkanRenderEngine {
    fn drop(&mut self) {
        todo!() // TODO
    }
}

impl Window for VulkanWindow {
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

impl Device for VulkanDevice {
    fn create_mesh(&mut self, vertex_positions: Vec<Vec3>, vertex_indexes: Option<Vec<usize>>) -> Result<Rc<Mesh>> {
        todo!() // TODO
    }
}
