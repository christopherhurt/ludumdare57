use anyhow::Result;
use std::rc::Rc;

use crate::core::{Mesh, Scene};
use crate::math::Vec3;

pub struct RenderEngineProperties {
    pub debug_enabled: bool,
    pub window_properties: WindowProperties,
}

pub struct WindowProperties {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub trait RenderEngine<W: Window, D: Device>: Drop {
    fn sync_data(&mut self, scene: &Scene) -> Result<()>;
    fn get_window(&self) -> &W;
    fn get_window_mut(&mut self) -> &mut W;
    fn get_device(&self) -> &D;
    fn get_device_mut(&mut self) -> &mut D;
}

pub trait Window {
    fn get_width(&self) -> Result<u32>;
    fn get_height(&self) -> Result<u32>;
    fn is_key_down(&self, key: VirtualKey) -> Result<bool>;
    fn is_closing(&self) -> bool;
}

pub trait Device {
    fn create_mesh(&mut self, vertex_positions: Vec<Vec3>, vertex_indexes: Option<Vec<usize>>) -> Result<Rc<Mesh>>;
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
