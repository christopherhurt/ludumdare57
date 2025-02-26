use anyhow::Result;
use std::sync::Arc;
use strum_macros::EnumIter;

use crate::math::Vec3;

pub mod vulkan;

pub struct MeshId(usize);

pub struct RenderEngineInitProps {
    pub debug_enabled: bool,
    pub window_props: WindowInitProps,
}

pub struct WindowInitProps {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub trait RenderEngine<W: Window, D: Device> {
    unsafe fn new(init_props: RenderEngineInitProps) -> Self;
    fn sync_state(&mut self, state: RenderState);
    fn get_window(&self) -> &W;
    fn get_window_mut(&mut self) -> &mut W;
    fn get_device(&self) -> &D;
    fn get_device_mut(&mut self) -> &mut D;
    unsafe fn join_render_thread(&mut self) -> Result<()>;
}

pub trait Window {
    fn get_width(&self) -> Result<u32>;
    fn get_height(&self) -> Result<u32>;
    fn is_key_down(&self, key: VirtualKey) -> Result<bool>;
    fn is_closing(&self) -> bool;
}

pub trait Device {
    unsafe fn create_mesh(&mut self, vertex_positions: Vec<Vec3>, vertex_indexes: Option<Vec<usize>>) -> Result<Arc<MeshId>>;
}

pub struct RenderState {
    // TODO: add contents
}

#[derive(Debug, Clone, Copy, EnumIter, Eq, Hash, PartialEq)]
pub enum VirtualKey {
    Unknown,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
    Up,
    Left,
    Down,
    Right,
}
