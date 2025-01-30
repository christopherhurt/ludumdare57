use std::rc::Rc;

use crate::math::{vec2, vec3, Vec2, Vec3, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};

/////////////////////////////////////////////////////////////////////////////
/// Common
/////////////////////////////////////////////////////////////////////////////

// Color

pub mod color {
    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub const RED: Color = rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = rgb(0.0, 0.0, 1.0);
}

/////////////////////////////////////////////////////////////////////////////
/// Scene
/////////////////////////////////////////////////////////////////////////////

// Event

pub struct Event {
    pub evt_type: EventType,
}

pub enum EventType {
    Update,
}

// Signal

pub struct Signal {
    pub name: str,
}

// Camera

pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        Self { pos, dir, up }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            dir: VEC_3_Z_AXIS,
            up: VEC_3_Y_AXIS,
        }
    }
}

// Viewport2D

pub struct Viewport2D {
    pub cam: Camera,
    pub offset: Vec2,
    pub scale: Vec2,
}

impl Viewport2D {
    pub fn new(cam: Camera, offset: Vec2, scale: Vec2) -> Self {
        Self { cam, offset, scale }
    }
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self {
            cam: Camera::default(),
            offset: VEC_2_ZERO,
            scale: vec2(1.0, 1.0),
        }
    }
}

// Scene

pub struct Scene {
    pub root_nodes: Vec<Node>,
    pub viewports: Vec<Viewport2D>,
}

impl Scene {
    pub fn new(root_nodes: Vec<Node>, viewports: Vec<Viewport2D>) -> Self {
        Self { root_nodes, viewports }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            root_nodes: Vec::with_capacity(256),
            viewports: vec![Viewport2D::default()],
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Node
/////////////////////////////////////////////////////////////////////////////

// Node

pub struct Node {
    pub id: u32,
    pub tags: Box<[&'static str]>,
    pub children: Vec<Node>,

    pub transform: Option<Transform>,
    pub mesh: Option<Rc<Mesh>>,
    pub color_material: Option<Rc<ColorMaterial>>,

    pub handle_event: Option<fn(&Scene, &Event)>,
    pub handle_signal: Option<fn(&Scene, &Signal)>,
}

// Transform

pub struct Transform {
    pub pos: Vec3,
    pub rot: Vec3,
    pub scl: Vec3,
}

impl Transform {
    pub fn new(pos: Vec3, rot: Vec3, scl: Vec3) -> Self {
        Self { pos, rot, scl }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            rot: VEC_3_ZERO,
            scl: vec3(1.0, 1.0, 1.0),
        }
    }
}

// Mesh

pub struct Mesh {
    pub id: u32,
}

// ColorMaterial

pub struct ColorMaterial {
    pub color: color::Color,
}

impl Default for ColorMaterial {
    fn default() -> Self {
        Self {
            color: color::RED,
        }
    }
}
