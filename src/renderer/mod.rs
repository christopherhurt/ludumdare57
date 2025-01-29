use anyhow::Result;

use crate::core::Scene;

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
