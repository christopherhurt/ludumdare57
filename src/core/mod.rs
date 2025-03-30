use anyhow::Result;
use std::time::{Duration, SystemTime};

use crate::ecs::ComponentActions;
use crate::ecs::component::Component;
use crate::math::{get_scale_matrix, get_view_matrix, get_world_matrix, vec2, vec3, Mat4, Quat, Vec2, Vec3, QUAT_IDENTITY, VEC_2_ZERO, VEC_3_Y_AXIS, VEC_3_ZERO, VEC_3_Z_AXIS};

pub mod mesh;

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

impl PartialEq<Color> for Color {
    fn eq(&self, other: &Color) -> bool {
        (self.r - other.r).abs() < f32::EPSILON
            && (self.g - other.g).abs() < f32::EPSILON
            && (self.b - other.b).abs() < f32::EPSILON
            && (self.a - other.a).abs() < f32::EPSILON
    }
}

impl Eq for Color {}

pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
pub const ORANGE: Color = Color::rgb(1.0, 0.35, 0.0);
pub const PURPLE: Color = Color::rgb(0.5, 0.0, 0.5);
pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
pub const BROWN: Color = Color::rgb(0.6, 0.3, 0.0);
pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);

// Camera

pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,
    pub fov_rads: f32,
}

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, up: Vec3, fov_rads: f32) -> Self {
        Self { pos, dir, up, fov_rads }
    }

    pub(in crate) fn to_view_mat(&self) -> Result<Mat4> {
        get_view_matrix(&self.dir, &self.up, &self.pos)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            dir: VEC_3_Z_AXIS,
            up: VEC_3_Y_AXIS,
            fov_rads: 45.0_f32.to_radians(),
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

impl Component for Viewport2D {}
impl ComponentActions for Viewport2D {}

// Transform

#[derive(Clone, Debug)]
pub struct Transform {
    pos: Vec3,
    rot: Quat,
    scl: Vec3,

    pos_changed: bool,
    rot_changed: bool,
    scl_changed: bool,

    world_mat: Option<Mat4>,
    rot_mat: Option<Mat4>,
    scl_mat: Option<Mat4>,
}

impl Transform {
    pub fn new(pos: Vec3, rot: Quat, scl: Vec3) -> Self {
        Self {
            pos,
            rot,
            scl,

            pos_changed: true,
            rot_changed: true,
            scl_changed: true,

            world_mat: None,
            rot_mat: None,
            scl_mat: None,
        }
    }

    pub fn to_world_mat(&mut self) -> &Mat4 {
        self.world_mat.get_or_insert_with(|| get_world_matrix(&self.pos, &self.rot, &self.scl))
    }

    pub fn to_rot_mat(&mut self) -> &Mat4 {
        self.rot_mat.get_or_insert_with(|| self.rot.to_rotation_matrix())
    }

    pub fn to_scl_mat(&mut self) -> &Mat4 {
        self.scl_mat.get_or_insert_with(|| get_scale_matrix(&self.scl))
    }

    pub fn is_pos_changed_since_last_frame(&self) -> bool {
        self.pos_changed
    }

    pub fn is_rot_changed_since_last_frame(&self) -> bool {
        self.rot_changed
    }

    pub fn is_scl_changed_since_last_frame(&self) -> bool {
        self.scl_changed
    }

    pub(in crate) fn reset_changed_flags(&mut self) {
        self.pos_changed = false;
        self.rot_changed = false;
        self.scl_changed = false;
    }

    pub fn get_pos(&self) -> &Vec3 {
        &self.pos
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;

        self.pos_changed = true;

        self.world_mat = None;
    }

    pub fn get_rot(&self) -> &Quat {
        &self.rot
    }

    pub fn set_rot(&mut self, rot: Quat) {
        self.rot = rot;

        self.rot_changed = true;

        self.world_mat = None;
        self.rot_mat = None;
    }

    pub fn get_scl(&self) -> &Vec3 {
        &self.scl
    }

    pub fn set_scl(&mut self, scl: Vec3) {
        self.scl = scl;

        self.scl_changed = true;

        self.world_mat = None;
        self.scl_mat = None;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: VEC_3_ZERO,
            rot: QUAT_IDENTITY,
            scl: IDENTITY_SCALE_VEC,

            pos_changed: true,
            rot_changed: true,
            scl_changed: true,

            world_mat: None,
            rot_mat: None,
            scl_mat: None,
        }
    }
}

impl Component for Transform {}
impl ComponentActions for Transform {}

pub const IDENTITY_SCALE_VEC: Vec3 = vec3(1.0, 1.0, 1.0);

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
impl ComponentActions for ColorMaterial {}

// TimeDelta

pub struct TimeDelta {
    pub(in crate) is_started: bool,
    pub(in crate) timestamp: SystemTime,
    pub since_last_frame: Duration,
}

impl Default for TimeDelta {
    fn default() -> Self {
        Self {
            is_started: false,
            timestamp: SystemTime::now(),
            since_last_frame: Duration::from_secs(0),
        }
    }
}

impl Component for TimeDelta {}
impl ComponentActions for TimeDelta {}

// Timer

pub struct Timer {
    pub start_value: f32,
    pub end_value: f32,
    pub initial_duration: Duration,
    pub current_value: f32,
    pub remaining_duration: Option<Duration>,
}

impl Timer {
    pub fn new(start_value: f32, end_value: f32, initial_duration: Duration) -> Self {
        Self {
            start_value,
            end_value,
            initial_duration,
            current_value: get_current_value(start_value, end_value, initial_duration),
            remaining_duration: get_remaining_duration(start_value, end_value, initial_duration),
        }
    }

    pub fn for_initial_duration(initial_duration: Duration) -> Self {
        Timer::new(0.0, 1.0, initial_duration)
    }

    pub fn reset(&mut self) {
        self.current_value = get_current_value(self.start_value, self.end_value, self.initial_duration);
        self.remaining_duration = get_remaining_duration(self.start_value, self.end_value, self.initial_duration);
    }

    pub(in crate) fn update(&mut self, time_delta: &Duration) {
        if let Some(d) = self.remaining_duration {
            self.remaining_duration = d.checked_sub(*time_delta);

            if let Some(d) = self.remaining_duration {
                let ratio = d.as_secs_f32() / self.initial_duration.as_secs_f32();

                self.current_value = self.start_value + (self.end_value - self.start_value) * (1.0 - ratio);
            } else {
                self.current_value = self.end_value;
            }
        }
    }
}

fn get_current_value(start_value: f32, end_value: f32, initial_duration: Duration) -> f32 {
    if initial_duration.is_zero() {
        end_value
    } else {
        start_value
    }
}

fn get_remaining_duration(start_value: f32, end_value: f32, initial_duration: Duration) -> Option<Duration> {
    if initial_duration.is_zero() || start_value == end_value {
        None
    } else {
        Some(initial_duration)
    }
}

impl Component for Timer {}
impl ComponentActions for Timer {}
