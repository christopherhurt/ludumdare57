use anyhow::Result;
use strum_macros::EnumIter;

use crate::core::Color;
use crate::ecs::ComponentActions;
use crate::ecs::component::Component;
use crate::math::{Mat4, Vec3};

pub mod vulkan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub(in crate::render_engine) usize);

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
}

pub trait RenderEngine<W: Window, D: Device>: Sized {
    unsafe fn new(init_props: RenderEngineInitProps) -> Result<Self>;
    fn sync_state(&mut self, state: RenderState) -> Result<()>;
    fn get_window(&self) -> Result<&W>;
    fn get_window_mut(&mut self) -> Result<&mut W>;
    fn get_device(&self) -> Result<&D>;
    fn get_device_mut(&mut self) -> Result<&mut D>;
    unsafe fn join_render_thread(&mut self) -> Result<()>;
}

pub trait Window {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn is_key_down(&self, key: VirtualKey) -> bool;
    fn is_closing(&self) -> bool;
}

pub trait Device {
    unsafe fn create_mesh(&mut self, vertices: Vec<Vertex>, vertex_indexes: Vec<u32>) -> Result<MeshId>;
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
    pub mesh_id: MeshId,
    pub color: Color,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub pos: Vec3,
    pub norm: Vec3,
}

pub struct Mesh {
    pub id: MeshId,
}

impl Mesh {
    pub fn new(id: MeshId) -> Self {
        Self { id }
    }
}

impl Component for Mesh {}
impl ComponentActions for Mesh {}

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
    Enter,
    Up,
    Left,
    Down,
    Right,
}
