use anyhow::Result;

use crate::core::Scene;

// TODO: just rewrite for Vulkan renderer, window, and device
// TODO: rename renderer to something like render engine? and have it return refs to window and device?
// TODO: maybe keep the traits for organization, but no need to reference them in main.rs for now
// TODO: I think keep this like a singleton for now? or at least have some way to prevent multiple initializations

pub trait Renderer<W: Window> {
    fn get_instance() -> &'static mut Self;
    fn render_scene(&mut self, scene: &Scene) -> Result<()>;
    fn get_window(&self) -> &W;
}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn is_key_down(&self, key: VirtualKey) -> bool;
    fn should_close(&self) -> bool;
}

pub enum VirtualKey {
    W,
    A,
    S,
    D,
    Up,
    Left,
    Down,
    Right,
}
