use crate::ecs::component::Component;
use crate::math::{vec2, vec3, Mat4, Quat, Vec2, Vec3, MAT_4_IDENTITY, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};

/////////////////////////////////////////////////////////////////////////////
/// Common
/////////////////////////////////////////////////////////////////////////////

// Color

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);

// Camera

pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,
    pub fov_deg: f32,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, up: Vec3, fov_deg: f32) -> Self {
        Self { pos, dir, up, fov_deg }
    }

    pub(in crate) fn to_view_mat(&self) -> Mat4 {
        // TODO
        MAT_4_IDENTITY
    }

    pub(in crate) fn to_proj_mat(&self) -> Mat4 {
        // TODO
        MAT_4_IDENTITY
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            dir: VEC_3_Z_AXIS,
            up: VEC_3_Y_AXIS,
            fov_deg: 45.0,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Components
/////////////////////////////////////////////////////////////////////////////

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

// TODO: move this and other Component impls to bindings module? thinking about how the dependency tree is organized...
impl Component for Viewport2D {}

// Transform

pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scl: Vec3,
}

impl Transform {
    pub fn new(pos: Vec3, rot: Quat, scl: Vec3) -> Self {
        Self { pos, rot, scl }
    }

    pub(in crate) fn to_world_mat(&self) -> Mat4 {
        // TODO
        MAT_4_IDENTITY
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            rot: Quat::from_axis_spin(&vec3(1.0, 0.0, 0.0), 0.0),
            scl: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl Component for Transform {}

// ColorMaterial

pub struct ColorMaterial {
    pub color: Color,
}

impl ColorMaterial {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Default for ColorMaterial {
    fn default() -> Self {
        Self {
            color: RED,
        }
    }
}

impl Component for ColorMaterial {}
