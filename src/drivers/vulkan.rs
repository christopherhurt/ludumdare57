use anyhow::Result;

use crate::core::Scene;
use crate::renderer::{Renderer, VirtualKey, Window};

/////////////////////////////////////////////////////////////////////////////
/// VulkanWindow
/////////////////////////////////////////////////////////////////////////////

pub struct VulkanWindow {
    // TODO
}

impl Window for VulkanWindow {
    fn get_width(&self) -> u32 {
        todo!() // TODO
    }

    fn get_height(&self) -> u32 {
        todo!() // TODO
    }

    fn is_key_down(&self, key: VirtualKey) -> bool {
        todo!() // TODO
    }

    fn should_close(&self) -> bool {
        todo!() // TODO
    }
}

/////////////////////////////////////////////////////////////////////////////
/// VulkanRenderer
/////////////////////////////////////////////////////////////////////////////

pub struct VulkanRenderer<W: Window> {
    // TODO
    window: W,
}

impl Renderer<VulkanWindow> for VulkanRenderer<VulkanWindow> {
    fn get_instance() -> &'static mut Self {
        todo!() // TODO
    }

    fn render_scene(&mut self, scene: &Scene) -> Result<()> {
        todo!() // TODO
    }

    fn get_window(&self) -> &VulkanWindow {
        todo!() // TODO
    }
}
