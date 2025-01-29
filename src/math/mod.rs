/////////////////////////////////////////////////////////////////////////////
/// Vec2
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

pub const VEC_2_ZERO: Vec2 = vec2(0.0, 0.0);
pub const VEC_2_X_AXIS: Vec2 = vec2(1.0, 0.0);
pub const VEC_2_Y_AXIS: Vec2 = vec2(0.0, 1.0);

/////////////////////////////////////////////////////////////////////////////
/// Vec3
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[inline]
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

pub const VEC_3_ZERO: Vec3 = vec3(0.0, 0.0, 0.0);
pub const VEC_3_X_AXIS: Vec3 = vec3(1.0, 0.0, 0.0);
pub const VEC_3_Y_AXIS: Vec3 = vec3(0.0, 1.0, 0.0);
pub const VEC_3_Z_AXIS: Vec3 = vec3(0.0, 0.0, 1.0);
