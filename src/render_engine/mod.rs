use anyhow::Result;
use std::sync::Arc;
use strum_macros::{EnumCount, EnumIter};

use crate::core::{Color, RenderTextureId};
use crate::core::mesh::{RenderMeshId, Vertex};
use crate::math::{Mat3, Mat4, Vec2};

pub mod vulkan;

#[derive(Clone, Debug)]
pub struct RenderEngineInitProps {
    pub debug_enabled: bool,
    pub clear_color: Color,
    pub window_props: WindowInitProps,
}

#[derive(Clone, Debug)]
pub struct WindowInitProps {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub is_resizable: bool,
}

pub trait RenderEngine<W: Window, D: Device>: Sized {
    fn new(init_props: RenderEngineInitProps) -> Result<Self>;
    fn sync_state(&mut self, state: RenderState) -> Result<()>;
    fn get_window(&self) -> Result<&W>;
    fn get_window_mut(&mut self) -> Result<&mut W>;
    fn get_device(&self) -> Result<&D>;
    fn get_device_mut(&mut self) -> Result<&mut D>;
    fn join_render_thread(&mut self) -> Result<()>;
}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_screen_position(&self) -> Vec2;
    fn is_key_down(&self, key: VirtualKey) -> bool;
    fn is_key_pressed(&self, key: VirtualKey) -> bool;
    fn is_key_released(&self, key: VirtualKey) -> bool;
    fn is_button_down(&self, button: VirtualButton) -> bool;
    fn is_button_pressed(&self, button: VirtualButton) -> bool;
    fn is_button_released(&self, button: VirtualButton) -> bool;
    fn get_mouse_screen_position(&self) -> Option<&Vec2>;
    fn set_mouse_screen_position(&mut self, screen_pos: &Vec2) -> Result<()>; // TODO: rename this and other functions to "cursor". Also don't use f32 Vec for this, but unsigned ints
    fn set_mouse_cursor_visible(&mut self, is_visible: bool) -> Result<()>;
    fn get_ndc_to_screen_space_transform(&self) -> Mat3;
    fn is_closing(&self) -> bool;
}

pub trait Device {
    fn create_mesh(&mut self, vertices: Arc<Vec<Vertex>>, vertex_indexes: Arc<Vec<u32>>) -> Result<RenderMeshId>;
    fn create_texture(&mut self, file_path: String) -> Result<RenderTextureId>;
}

#[derive(Clone, Debug)]
pub struct RenderState {
    pub view: Mat4,
    pub proj: Mat4,
    pub entity_states: Vec<EntityRenderState>,
}

#[derive(Clone, Debug)]
pub struct EntityRenderState {
    pub world: Mat4,
    pub mesh_id: RenderMeshId,
    pub texture_id: RenderTextureId,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, Eq, Hash, PartialEq)]
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
    Enter,
    Up,
    Left,
    Down,
    Right,
    Escape,
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, Eq, Hash, PartialEq)]
pub enum VirtualButton {
    Unknown,
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, EnumCount, EnumIter, Eq, Hash, PartialEq)]
pub enum VirtualElementState {
    Pressed,
    Released,
}
