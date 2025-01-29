use crate::ecs::{Component, ECS, Entity, SystemId};
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
    pub ecs: ECS,
    pub viewports: Vec<Viewport2D>,
}

impl Scene {
    pub fn new(ecs: ECS, viewports: Vec<Viewport2D>) -> Self {
        Self { ecs, viewports }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            ecs: ECS::default(),
            viewports: vec![Viewport2D::default()],
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Components
/////////////////////////////////////////////////////////////////////////////

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

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            rot: VEC_3_ZERO,
            scl: vec3(1.0, 1.0, 1.0),
        }
    }
}

// Behavior

pub struct Behavior<F: Fn(&Scene, Entity)> {
    pub on_update: F,
}

impl<F: Fn(&Scene, Entity) + 'static> Component for Behavior<F> {}

// ColorMaterial

pub struct ColorMaterial {
    pub color: color::Color,
}

impl Component for ColorMaterial {}

impl Default for ColorMaterial {
    fn default() -> Self {
        Self {
            color: color::RED,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Systems
/////////////////////////////////////////////////////////////////////////////

pub enum System {
    Behavior,
    Render,
}

impl System {
    pub fn get_id(&self) -> SystemId {
        match self {
            System::Behavior => 0,
            System::Render => 1,
        }
    }
}
