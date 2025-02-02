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

/////////////////////////////////////////////////////////////////////////////
/// Mat4
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mat4 {
    pub _00: f32,
    pub _01: f32,
    pub _02: f32,
    pub _03: f32,
    pub _10: f32,
    pub _11: f32,
    pub _12: f32,
    pub _13: f32,
    pub _20: f32,
    pub _21: f32,
    pub _22: f32,
    pub _23: f32,
    pub _30: f32,
    pub _31: f32,
    pub _32: f32,
    pub _33: f32,
}

#[inline]
pub const fn mat4(
    _00: f32, _01: f32, _02: f32, _03: f32,
    _10: f32, _11: f32, _12: f32, _13: f32,
    _20: f32, _21: f32, _22: f32, _23: f32,
    _30: f32, _31: f32, _32: f32, _33: f32,
) -> Mat4 {
    Mat4 {
        _00, _01, _02, _03,
        _10, _11, _12, _13,
        _20, _21, _22, _23,
        _30, _31, _32, _33,
    }
}

pub const MAT_4_IDENTITY: Mat4 = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
);
