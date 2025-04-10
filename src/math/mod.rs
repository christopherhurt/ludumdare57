use anyhow::{anyhow, Result};
use std::ops;

const EQUALITY_THRESHOLD: f32 = f32::EPSILON;

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

impl Vec2 {
    #[inline]
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn normalized(&self) -> Result<Vec2> {
        let len = self.len();

        if len < EQUALITY_THRESHOLD {
            Err(anyhow!("Cannot normalize a zero length vector!"))
        } else {
            Ok(vec2(self.x / len, self.y / len))
        }
    }

    #[inline]
    pub fn to_vec3(&self, z: f32) -> Vec3 {
        vec3(self.x, self.y, z)
    }

    #[inline]
    pub fn to_vec4(&self, z: f32, w: f32) -> Vec4 {
        vec4(self.x, self.y, z, w)
    }
}

impl ops::Add for Vec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        vec2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        vec2(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ops::SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::Mul for Vec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        vec2(self.x * rhs.x, self.y * rhs.y)
    }
}

impl ops::MulAssign for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self {
        vec2(self.x * rhs, self.y * rhs)
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        vec2(self * rhs.x, self * rhs.y)
    }
}

impl ops::MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl ops::Div for Vec2 {
    type Output = Vec2;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        vec2(self.x / rhs.x, self.y / rhs.y)
    }
}

impl ops::DivAssign for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn div(self, rhs: f32) -> Self {
        vec2(self.x / rhs, self.y / rhs)
    }
}

impl ops::DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl ops::Neg for Vec2 {
    type Output = Vec2;

    #[inline]
    fn neg(self) -> Self {
        vec2(-self.x, -self.y)
    }
}

impl PartialEq<Vec2> for Vec2 {
    fn eq(&self, other: &Vec2) -> bool {
        (self.x - other.x).abs() < EQUALITY_THRESHOLD
            && (self.y - other.y).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Vec2 {}

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

impl Vec3 {
    #[inline]
    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline]
    pub fn normalized(&self) -> Result<Vec3> {
        let len = self.len();

        if len < EQUALITY_THRESHOLD {
            Err(anyhow!("Cannot normalize a zero length vector!"))
        } else {
            Ok(vec3(self.x / len, self.y / len, self.z / len))
        }
    }

    #[inline]
    pub fn dot(&self, vec: &Vec3) -> f32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    #[inline]
    pub fn cross(&self, vec: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * vec.z - self.z * vec.y,
            y: self.z * vec.x - self.x * vec.z,
            z: self.x * vec.y - self.y * vec.x,
        }
    }

    #[inline]
    pub fn angle_rads_from(&self, vec: &Vec3) -> Result<f32> {
        let self_len = self.len();
        let vec_len = vec.len();

        if self_len < EQUALITY_THRESHOLD || vec_len < EQUALITY_THRESHOLD {
            Err(anyhow!("Cannot get angle from a zero length vector!"))
        } else {
            Ok((self.dot(vec) / (self_len * vec_len)).acos())
        }
    }

    #[inline]
    pub fn rotated(&self, axis: &Vec3, spin_rads: f64) -> Result<Vec3> {
        // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
        let axis_norm = match axis.normalized() {
            Ok(a) => Ok(a),
            Err(_) => Err(anyhow!("Cannot rotate vector about a zero length axis!")),
        }?;

        let half_spin_rads = spin_rads / 2.0;
        let cos_half_spin = half_spin_rads.cos() as f32;
        let sin_half_spin = half_spin_rads.sin() as f32;
        let crossed = axis_norm.cross(self);

        Ok(*self + (2.0 * cos_half_spin * sin_half_spin * crossed) + (2.0 * sin_half_spin * sin_half_spin * axis_norm.cross(&crossed)))
    }

    #[inline]
    pub fn distance_to(&self, other: &Vec3) -> f32 {
        (*self - *other).len()
    }

    #[inline]
    pub fn xy(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    #[inline]
    pub fn yx(&self) -> Vec2 {
        vec2(self.y, self.x)
    }

    #[inline]
    pub fn xz(&self) -> Vec2 {
        vec2(self.x, self.z)
    }

    #[inline]
    pub fn zx(&self) -> Vec2 {
        vec2(self.z, self.x)
    }

    #[inline]
    pub fn yz(&self) -> Vec2 {
        vec2(self.y, self.z)
    }

    #[inline]
    pub fn zy(&self) -> Vec2 {
        vec2(self.z, self.y)
    }

    #[inline]
    pub fn to_vec4(&self, w: f32) -> Vec4 {
        vec4(self.x, self.y, self.z, w)
    }
}

impl ops::Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        vec3(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        vec3(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Mul for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        vec3(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl ops::MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self {
        vec3(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        vec3(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl ops::MulAssign<f32> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div for Vec3 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        vec3(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl ops::DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: f32) -> Self {
        vec3(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl ops::DivAssign<f32> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    #[inline]
    fn neg(self) -> Self {
        vec3(-self.x, -self.y, -self.z)
    }
}

impl PartialEq<Vec3> for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        (self.x - other.x).abs() < EQUALITY_THRESHOLD
            && (self.y - other.y).abs() < EQUALITY_THRESHOLD
            && (self.z - other.z).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Vec3 {}

/////////////////////////////////////////////////////////////////////////////
/// Vec4
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 { x, y, z, w }
}

pub const VEC_4_ZERO: Vec4 = vec4(0.0, 0.0, 0.0, 0.0);
pub const VEC_4_X_AXIS: Vec4 = vec4(1.0, 0.0, 0.0, 0.0);
pub const VEC_4_Y_AXIS: Vec4 = vec4(0.0, 1.0, 0.0, 0.0);
pub const VEC_4_Z_AXIS: Vec4 = vec4(0.0, 0.0, 1.0, 0.0);
pub const VEC_4_W_AXIS: Vec4 = vec4(0.0, 0.0, 0.0, 1.0);

impl Vec4 {
    #[inline]
    pub fn xy(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    #[inline]
    pub fn xyz(&self) -> Vec3 {
        vec3(self.x, self.y, self.z)
    }
}

impl PartialEq<Vec4> for Vec4 {
    fn eq(&self, other: &Vec4) -> bool {
        (self.x - other.x).abs() < EQUALITY_THRESHOLD
            && (self.y - other.y).abs() < EQUALITY_THRESHOLD
            && (self.z - other.z).abs() < EQUALITY_THRESHOLD
            && (self.w - other.w).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Vec4 {}

/////////////////////////////////////////////////////////////////////////////
/// Quaternion
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Quat {
    pub w: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

#[inline]
pub const fn quat(w: f32, i: f32, j: f32, k: f32) -> Quat {
    Quat { w, i, j, k }
}

pub const QUAT_IDENTITY: Quat = quat(1.0, 0.0, 0.0, 0.0);

impl Quat {
    #[inline]
    pub fn len(&self) -> f32 {
        (self.w * self.w + self.i * self.i + self.j * self.j + self.k * self.k).sqrt()
    }

    #[inline]
    pub fn normalized(&self) -> Quat {
        let len = self.len();

        if len < EQUALITY_THRESHOLD {
            QUAT_IDENTITY
        } else {
            quat(self.w / len, self.i / len, self.j / len, self.k / len)
        }
    }

    #[inline]
    pub fn from_axis_spin(axis: &Vec3, spin_rads: f32) -> Result<Self> {
        let sin_half = (spin_rads / 2.0).sin();
        let cos_half = (spin_rads / 2.0).cos();

        let axis_norm = match axis.normalized() {
            Ok(a) => Ok(a),
            Err(_) => Err(anyhow!("Cannot get axis spin for a zero length axis!")),
        }?;

        Ok(
            Self {
                w: cos_half,
                i: sin_half * axis_norm.x,
                j: sin_half * axis_norm.y,
                k: sin_half * axis_norm.z,
            }
        )
    }

    #[inline]
    pub fn to_rotation_matrix(&self) -> Mat4 {
        let norm = self.normalized();

        mat4(
            1.0 - 2.0 * (norm.j * norm.j + norm.k * norm.k),    2.0 * (norm.i * norm.j - norm.k * norm.w),          2.0 * (norm.i * norm.k + norm.j * norm.w),          0.0,
            2.0 * (norm.i * norm.j + norm.k * norm.w),          1.0 - 2.0 * (norm.i * norm.i + norm.k * norm.k),    2.0 * (norm.j * norm.k - norm.i * norm.w),          0.0,
            2.0 * (norm.i * norm.k - norm.j * norm.w),          2.0 * (norm.j * norm.k + norm.i * norm.w),          1.0 - 2.0 * (norm.i * norm.i + norm.j * norm.j),    0.0,
            0.0,                                                0.0,                                                0.0,                                                1.0,
        )
    }
}

impl ops::Mul for Quat {
    type Output = Quat;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // https://stackoverflow.com/questions/19956555/how-to-multiply-two-quaternions

        Self {
            w: self.w * rhs.w - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k,
            i: self.w * rhs.i + self.i * rhs.w + self.j * rhs.k - self.k * rhs.j,
            j: self.w * rhs.j - self.i * rhs.k + self.j * rhs.w + self.k * rhs.i,
            k: self.w * rhs.k + self.i * rhs.j - self.j * rhs.i + self.k * rhs.w,
        }
    }
}

impl ops::MulAssign for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        // https://stackoverflow.com/questions/19956555/how-to-multiply-two-quaternions

        self.w = self.w * rhs.w - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k;
        self.i = self.w * rhs.i + self.i * rhs.w + self.j * rhs.k - self.k * rhs.j;
        self.j = self.w * rhs.j - self.i * rhs.k + self.j * rhs.w + self.k * rhs.i;
        self.k = self.w * rhs.k + self.i * rhs.j - self.j * rhs.i + self.k * rhs.w;
    }
}

impl PartialEq<Quat> for Quat {
    fn eq(&self, other: &Quat) -> bool {
        (self.i - other.i).abs() < EQUALITY_THRESHOLD
            && (self.j - other.j).abs() < EQUALITY_THRESHOLD
            && (self.k - other.k).abs() < EQUALITY_THRESHOLD
            && (self.w - other.w).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Quat {}

/////////////////////////////////////////////////////////////////////////////
/// Mat3
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mat3 {
    // See the comment on Mat4 regarding this memory layout
    _00: f32,
    _10: f32,
    _20: f32,
    _01: f32,
    _11: f32,
    _21: f32,
    _02: f32,
    _12: f32,
    _22: f32,
}

#[inline]
pub const fn mat3(
    _00: f32, _01: f32, _02: f32,
    _10: f32, _11: f32, _12: f32,
    _20: f32, _21: f32, _22: f32,
) -> Mat3 {
    Mat3 {
        _00, _01, _02,
        _10, _11, _12,
        _20, _21, _22,
    }
}

pub const MAT_3_IDENTITY: Mat3 = mat3(
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
);

impl Mat3 {
    #[inline]
    pub fn from_columns(col_0: &Vec3, col_1: &Vec3, col_2: &Vec3) -> Self {
        mat3(
            col_0.x, col_1.x, col_2.x,
            col_0.y, col_1.y, col_2.y,
            col_0.z, col_1.z, col_2.z,
        )
    }

    #[inline]
    pub fn transposed(&self) -> Mat3 {
        mat3(
            self._00, self._10, self._20,
            self._01, self._11, self._21,
            self._02, self._12, self._22,
        )
    }

    #[inline]
    pub fn inverted(&self) -> Result<Mat3> { // TODO: cache the inversion
        // https://stackoverflow.com/questions/983999/simple-3x3-matrix-inverse-code-c

        let det = self._00 * (self._11 * self._22 - self._21 * self._12)
            - self._01 * (self._10 * self._22 - self._12 * self._20)
            + self._02 * (self._10 * self._21 - self._11 * self._20);

        if det < EQUALITY_THRESHOLD {
            return Err(anyhow!("Matrix is not invertible"));
        }

        let inv_det = 1.0 / det;

        Ok(
            Mat3 {
                _00: (self._11 * self._22 - self._21 * self._12) * inv_det,
                _01: (self._02 * self._21 - self._01 * self._22) * inv_det,
                _02: (self._01 * self._12 - self._02 * self._11) * inv_det,
                _10: (self._12 * self._20 - self._10 * self._22) * inv_det,
                _11: (self._00 * self._22 - self._02 * self._20) * inv_det,
                _12: (self._10 * self._02 - self._00 * self._12) * inv_det,
                _20: (self._10 * self._21 - self._20 * self._11) * inv_det,
                _21: (self._20 * self._01 - self._00 * self._21) * inv_det,
                _22: (self._00 * self._11 - self._10 * self._01) * inv_det,
            }
        )
    }
}

impl ops::Mul for Mat3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Mat3 {
            _00: self._00 * rhs._00 + self._01 * rhs._10 + self._02 * rhs._20,
            _01: self._00 * rhs._01 + self._01 * rhs._11 + self._02 * rhs._21,
            _02: self._00 * rhs._02 + self._01 * rhs._12 + self._02 * rhs._22,
            _10: self._10 * rhs._00 + self._11 * rhs._10 + self._12 * rhs._20,
            _11: self._10 * rhs._01 + self._11 * rhs._11 + self._12 * rhs._21,
            _12: self._10 * rhs._02 + self._11 * rhs._12 + self._12 * rhs._22,
            _20: self._20 * rhs._00 + self._21 * rhs._10 + self._22 * rhs._20,
            _21: self._20 * rhs._01 + self._21 * rhs._11 + self._22 * rhs._21,
            _22: self._20 * rhs._02 + self._21 * rhs._12 + self._22 * rhs._22,
        }
    }
}

impl ops::MulAssign for Mat3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self._00 = self._00 * rhs._00 + self._01 * rhs._10 + self._02 * rhs._20;
        self._01 = self._00 * rhs._01 + self._01 * rhs._11 + self._02 * rhs._21;
        self._02 = self._00 * rhs._02 + self._01 * rhs._12 + self._02 * rhs._22;
        self._10 = self._10 * rhs._00 + self._11 * rhs._10 + self._12 * rhs._20;
        self._11 = self._10 * rhs._01 + self._11 * rhs._11 + self._12 * rhs._21;
        self._12 = self._10 * rhs._02 + self._11 * rhs._12 + self._12 * rhs._22;
        self._20 = self._20 * rhs._00 + self._21 * rhs._10 + self._22 * rhs._20;
        self._21 = self._20 * rhs._01 + self._21 * rhs._11 + self._22 * rhs._21;
        self._22 = self._20 * rhs._02 + self._21 * rhs._12 + self._22 * rhs._22;
    }
}

impl ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self._00 * rhs.x + self._01 * rhs.y + self._02 * rhs.z,
            y: self._10 * rhs.x + self._11 * rhs.y + self._12 * rhs.z,
            z: self._20 * rhs.x + self._21 * rhs.y + self._22 * rhs.z,
        }
    }
}

impl PartialEq<Mat3> for Mat3 {
    fn eq(&self, other: &Mat3) -> bool {
        (self._00 - other._00).abs() < EQUALITY_THRESHOLD
            && (self._01 - other._01).abs() < EQUALITY_THRESHOLD
            && (self._02 - other._02).abs() < EQUALITY_THRESHOLD
            && (self._10 - other._10).abs() < EQUALITY_THRESHOLD
            && (self._11 - other._11).abs() < EQUALITY_THRESHOLD
            && (self._12 - other._12).abs() < EQUALITY_THRESHOLD
            && (self._20 - other._20).abs() < EQUALITY_THRESHOLD
            && (self._21 - other._21).abs() < EQUALITY_THRESHOLD
            && (self._22 - other._22).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Mat3 {}

/////////////////////////////////////////////////////////////////////////////
/// Mat4
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mat4 {
    // CURSED: This engine generally works with matrices in row-major order, e.g. the mat4()
    //  function below. But most shader programs will expect matrix values to be provided in
    //  column-major order. Since this struct uses a C-like memory layout, we'll just define
    //  the values in column major order such that we don't have to transpose every single
    //  matrix before serializing it and passing it as input to the shader program. In other
    //  words, THE ORDER IN WHICH THESE VALUES ARE DECLARED MATTERS.
    _00: f32,
    _10: f32,
    _20: f32,
    _30: f32,
    _01: f32,
    _11: f32,
    _21: f32,
    _31: f32,
    _02: f32,
    _12: f32,
    _22: f32,
    _32: f32,
    _03: f32,
    _13: f32,
    _23: f32,
    _33: f32,
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

impl Mat4 {
    #[inline]
    pub fn transposed(&self) -> Mat4 {
        mat4(
            self._00, self._10, self._20, self._30,
            self._01, self._11, self._21, self._31,
            self._02, self._12, self._22, self._32,
            self._03, self._13, self._23, self._33,
        )
    }

    #[inline]
    pub fn inverted(&self) -> Result<Mat4> { // TODO: cache the inversion
        // https://stackoverflow.com/questions/1148309/inverting-a-4x4-matrix

        let _2323 = self._22 * self._33 - self._23 * self._32;
        let _1323 = self._21 * self._33 - self._23 * self._31;
        let _1223 = self._21 * self._32 - self._22 * self._31;
        let _0323 = self._20 * self._33 - self._23 * self._30;
        let _0223 = self._20 * self._32 - self._22 * self._30;
        let _0123 = self._20 * self._31 - self._21 * self._30;
        let _2313 = self._12 * self._33 - self._13 * self._32;
        let _1313 = self._11 * self._33 - self._13 * self._31;
        let _1213 = self._11 * self._32 - self._12 * self._31;
        let _2312 = self._12 * self._23 - self._13 * self._22;
        let _1312 = self._11 * self._23 - self._13 * self._21;
        let _1212 = self._11 * self._22 - self._12 * self._21;
        let _0313 = self._10 * self._33 - self._13 * self._30;
        let _0213 = self._10 * self._32 - self._12 * self._30;
        let _0312 = self._10 * self._23 - self._13 * self._20;
        let _0212 = self._10 * self._22 - self._12 * self._20;
        let _0113 = self._10 * self._31 - self._11 * self._30;
        let _0112 = self._10 * self._21 - self._11 * self._20;

        let inv_det = self._00 * (self._11 * _2323 - self._12 * _1323 + self._13 * _1223)
            - self._01 * (self._10 * _2323 - self._12 * _0323 + self._13 * _0223)
            + self._02 * (self._10 * _1323 - self._11 * _0323 + self._13 * _0123)
            - self._03 * (self._10 * _1223 - self._11 * _0223 + self._12 * _0123);

        if inv_det.abs() < EQUALITY_THRESHOLD {
            return Err(anyhow!("Matrix is not invertible"));
        }

        let det = 1.0 / inv_det;

        Ok(
            Mat4 {
                _00: det *   (self._11 * _2323 - self._12 * _1323 + self._13 * _1223),
                _01: det * - (self._01 * _2323 - self._02 * _1323 + self._03 * _1223),
                _02: det *   (self._01 * _2313 - self._02 * _1313 + self._03 * _1213),
                _03: det * - (self._01 * _2312 - self._02 * _1312 + self._03 * _1212),
                _10: det * - (self._10 * _2323 - self._12 * _0323 + self._13 * _0223),
                _11: det *   (self._00 * _2323 - self._02 * _0323 + self._03 * _0223),
                _12: det * - (self._00 * _2313 - self._02 * _0313 + self._03 * _0213),
                _13: det *   (self._00 * _2312 - self._02 * _0312 + self._03 * _0212),
                _20: det *   (self._10 * _1323 - self._11 * _0323 + self._13 * _0123),
                _21: det * - (self._00 * _1323 - self._01 * _0323 + self._03 * _0123),
                _22: det *   (self._00 * _1313 - self._01 * _0313 + self._03 * _0113),
                _23: det * - (self._00 * _1312 - self._01 * _0312 + self._03 * _0112),
                _30: det * - (self._10 * _1223 - self._11 * _0223 + self._12 * _0123),
                _31: det *   (self._00 * _1223 - self._01 * _0223 + self._02 * _0123),
                _32: det * - (self._00 * _1213 - self._01 * _0213 + self._02 * _0113),
                _33: det *   (self._00 * _1212 - self._01 * _0212 + self._02 * _0112),
            }
        )
    }

    #[inline]
    pub fn to_mat3(&self) -> Mat3 {
        mat3(
            self._00, self._01, self._02,
            self._10, self._11, self._12,
            self._20, self._21, self._22,
        )
    }
}

impl ops::Mul for Mat4 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Mat4 {
            _00: self._00 * rhs._00 + self._01 * rhs._10 + self._02 * rhs._20 + self._03 * rhs._30,
            _01: self._00 * rhs._01 + self._01 * rhs._11 + self._02 * rhs._21 + self._03 * rhs._31,
            _02: self._00 * rhs._02 + self._01 * rhs._12 + self._02 * rhs._22 + self._03 * rhs._32,
            _03: self._00 * rhs._03 + self._01 * rhs._13 + self._02 * rhs._23 + self._03 * rhs._33,
            _10: self._10 * rhs._00 + self._11 * rhs._10 + self._12 * rhs._20 + self._13 * rhs._30,
            _11: self._10 * rhs._01 + self._11 * rhs._11 + self._12 * rhs._21 + self._13 * rhs._31,
            _12: self._10 * rhs._02 + self._11 * rhs._12 + self._12 * rhs._22 + self._13 * rhs._32,
            _13: self._10 * rhs._03 + self._11 * rhs._13 + self._12 * rhs._23 + self._13 * rhs._33,
            _20: self._20 * rhs._00 + self._21 * rhs._10 + self._22 * rhs._20 + self._23 * rhs._30,
            _21: self._20 * rhs._01 + self._21 * rhs._11 + self._22 * rhs._21 + self._23 * rhs._31,
            _22: self._20 * rhs._02 + self._21 * rhs._12 + self._22 * rhs._22 + self._23 * rhs._32,
            _23: self._20 * rhs._03 + self._21 * rhs._13 + self._22 * rhs._23 + self._23 * rhs._33,
            _30: self._30 * rhs._00 + self._31 * rhs._10 + self._32 * rhs._20 + self._33 * rhs._30,
            _31: self._30 * rhs._01 + self._31 * rhs._11 + self._32 * rhs._21 + self._33 * rhs._31,
            _32: self._30 * rhs._02 + self._31 * rhs._12 + self._32 * rhs._22 + self._33 * rhs._32,
            _33: self._30 * rhs._03 + self._31 * rhs._13 + self._32 * rhs._23 + self._33 * rhs._33,
        }
    }
}

impl ops::MulAssign for Mat4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self._00 = self._00 * rhs._00 + self._01 * rhs._10 + self._02 * rhs._20 + self._03 * rhs._30;
        self._01 = self._00 * rhs._01 + self._01 * rhs._11 + self._02 * rhs._21 + self._03 * rhs._31;
        self._02 = self._00 * rhs._02 + self._01 * rhs._12 + self._02 * rhs._22 + self._03 * rhs._32;
        self._03 = self._00 * rhs._03 + self._01 * rhs._13 + self._02 * rhs._23 + self._03 * rhs._33;
        self._10 = self._10 * rhs._00 + self._11 * rhs._10 + self._12 * rhs._20 + self._13 * rhs._30;
        self._11 = self._10 * rhs._01 + self._11 * rhs._11 + self._12 * rhs._21 + self._13 * rhs._31;
        self._12 = self._10 * rhs._02 + self._11 * rhs._12 + self._12 * rhs._22 + self._13 * rhs._32;
        self._13 = self._10 * rhs._03 + self._11 * rhs._13 + self._12 * rhs._23 + self._13 * rhs._33;
        self._20 = self._20 * rhs._00 + self._21 * rhs._10 + self._22 * rhs._20 + self._23 * rhs._30;
        self._21 = self._20 * rhs._01 + self._21 * rhs._11 + self._22 * rhs._21 + self._23 * rhs._31;
        self._22 = self._20 * rhs._02 + self._21 * rhs._12 + self._22 * rhs._22 + self._23 * rhs._32;
        self._23 = self._20 * rhs._03 + self._21 * rhs._13 + self._22 * rhs._23 + self._23 * rhs._33;
        self._30 = self._30 * rhs._00 + self._31 * rhs._10 + self._32 * rhs._20 + self._33 * rhs._30;
        self._31 = self._30 * rhs._01 + self._31 * rhs._11 + self._32 * rhs._21 + self._33 * rhs._31;
        self._32 = self._30 * rhs._02 + self._31 * rhs._12 + self._32 * rhs._22 + self._33 * rhs._32;
        self._33 = self._30 * rhs._03 + self._31 * rhs._13 + self._32 * rhs._23 + self._33 * rhs._33;
    }
}

impl ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self._00 * rhs.x + self._01 * rhs.y + self._02 * rhs.z + self._03 * rhs.w,
            y: self._10 * rhs.x + self._11 * rhs.y + self._12 * rhs.z + self._13 * rhs.w,
            z: self._20 * rhs.x + self._21 * rhs.y + self._22 * rhs.z + self._23 * rhs.w,
            w: self._30 * rhs.x + self._31 * rhs.y + self._32 * rhs.z + self._33 * rhs.w,
        }
    }
}

impl PartialEq<Mat4> for Mat4 {
    fn eq(&self, other: &Mat4) -> bool {
        (self._00 - other._00).abs() < EQUALITY_THRESHOLD
            && (self._01 - other._01).abs() < EQUALITY_THRESHOLD
            && (self._02 - other._02).abs() < EQUALITY_THRESHOLD
            && (self._03 - other._03).abs() < EQUALITY_THRESHOLD
            && (self._10 - other._10).abs() < EQUALITY_THRESHOLD
            && (self._11 - other._11).abs() < EQUALITY_THRESHOLD
            && (self._12 - other._12).abs() < EQUALITY_THRESHOLD
            && (self._13 - other._13).abs() < EQUALITY_THRESHOLD
            && (self._20 - other._20).abs() < EQUALITY_THRESHOLD
            && (self._21 - other._21).abs() < EQUALITY_THRESHOLD
            && (self._22 - other._22).abs() < EQUALITY_THRESHOLD
            && (self._23 - other._23).abs() < EQUALITY_THRESHOLD
            && (self._30 - other._30).abs() < EQUALITY_THRESHOLD
            && (self._31 - other._31).abs() < EQUALITY_THRESHOLD
            && (self._32 - other._32).abs() < EQUALITY_THRESHOLD
            && (self._33 - other._33).abs() < EQUALITY_THRESHOLD
    }
}

impl Eq for Mat4 {}

pub fn get_world_matrix(pos: &Vec3, rot: &Quat, scl: &Vec3) -> Mat4 {
    let mut rotation_translation = rot.to_rotation_matrix();

    // Translation
    rotation_translation._03 = pos.x;
    rotation_translation._13 = pos.y;
    rotation_translation._23 = pos.z;

    let scale = get_scale_matrix(scl);

    rotation_translation * scale
}

pub fn get_scale_matrix(scl: &Vec3) -> Mat4 {
    mat4(
        scl.x,  0.0,    0.0,    0.0,
        0.0,    scl.y,  0.0,    0.0,
        0.0,    0.0,    scl.z,  0.0,
        0.0,    0.0,    0.0,    1.0,
    )
}

pub fn get_view_matrix(dir: &Vec3, up: &Vec3, pos: &Vec3) -> Result<Mat4> {
    let right = match dir.cross(&up).normalized() {
        Ok(v) => Ok(v),
        Err(_) => Err(anyhow!("Forward and up vectors must have a non-zero length!")),
    }?;
    let dir = match dir.normalized() {
        Ok(v) => Ok(v),
        Err(_) => Err(anyhow!("Forward and up vectors must have a non-zero length!")),
    }?;
    let up = match up.normalized() {
        Ok(v) => Ok(v),
        Err(_) => Err(anyhow!("Forward and up vectors must have a non-zero length!")),
    }?;

    let rotation = mat4(
        right.x,    right.y,    right.z,    0.0,
        up.x,       up.y,       up.z,       0.0,
        dir.x,      dir.y,      dir.z,      0.0,
        0.0,        0.0,        0.0,        1.0,
    );

    let translation = mat4(
        1.0, 0.0, 0.0, -pos.x,
        0.0, 1.0, 0.0, -pos.y,
        0.0, 0.0, 1.0, -pos.z,
        0.0, 0.0, 0.0, 1.0,
    );

    Ok(rotation * translation)
}

pub fn get_proj_matrix(near: f32, far: f32, fov_rads: f32, aspect_ratio: f32) -> Result<Mat4> {
    if near <= 0.0 || far <= 0.0 {
        return Err(anyhow!("Near and far values must be positive!"));
    }
    if near >= far {
        return Err(anyhow!("Near value must be less than far value!"));
    }
    if aspect_ratio <= 0.0 {
        return Err(anyhow!("Aspect ratio must be positive!"));
    }

    let tan_half_fov = (fov_rads / 2.0).tan();

    Ok(
        mat4(
            1.0 / (tan_half_fov * aspect_ratio),    0.0,                    0.0,                0.0,
            0.0,                                    -1.0 / tan_half_fov,    0.0,                0.0,
            0.0,                                    0.0,                    far / (far - near), -near * far / (far - near),
            0.0,                                    0.0,                    1.0,                0.0,
        )
    )
}
