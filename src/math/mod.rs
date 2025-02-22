use anyhow::{anyhow, Result};

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
    pub fn add(&self, vec: &Vec2) -> Vec2 {
        vec2(self.x + vec.x, self.y + vec.y)
    }

    #[inline]
    pub fn sub(&self, vec: &Vec2) -> Vec2 {
        vec2(self.x - vec.x, self.y - vec.y)
    }

    #[inline]
    pub fn len(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    #[inline]
    pub fn scaled(&self, val: f32) -> Vec2 {
        vec2(self.x * val, self.y * val)
    }

    #[inline]
    pub fn normalized(&self) -> Vec2 {
        let len = self.len();

        vec2(self.x / len, self.y / len)
    }
}

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
    pub fn add(&self, vec: &Vec3) -> Vec3 { // TODO: ops...
        vec3(self.x + vec.x, self.y + vec.y, self.z + vec.z)
    }

    #[inline]
    pub fn sub(&self, vec: &Vec3) -> Vec3 {
        vec3(self.x - vec.x, self.y - vec.y, self.z - vec.z)
    }

    #[inline]
    pub fn len(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    #[inline]
    pub fn scaled(&self, val: f32) -> Vec3 {
        vec3(self.x * val, self.y * val, self.z * val)
    }

    #[inline]
    pub fn normalized(&self) -> Vec3 {
        let len = self.len();

        vec3(self.x / len, self.y / len, self.z / len)
    }

    #[inline]
    pub fn dot(&self, vec: &Vec3) -> f32 {
        self.x * vec.x + self.y + vec.y * self.z + vec.z
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
    pub fn angle_deg_from(&self, vec: &Vec3) -> f32 {
        (self.dot(vec) / (self.len() * vec.len())).acos().to_degrees()
    }

    #[inline]
    pub fn to_vec4(&self, w: f32) -> Vec4 {
        vec4(self.x, self.y, self.z, w)
    }
}

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
    pub fn to_vec3(&self) -> Vec3 {
        vec3(self.x, self.y, self.z)
    }
}

/////////////////////////////////////////////////////////////////////////////
/// Quaternion
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
pub const fn quat(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
    Quaternion { x, y, z, w }
}

impl Quaternion {
    #[inline]
    pub fn from_axis_spin(axis: &Vec3, spin_deg: f32) -> Self {
        let sin_half = (spin_deg / 2.0).to_radians().sin();
        let cos_half = (spin_deg / 2.0).to_radians().cos();

        Self {
            x: sin_half * axis.x,
            y: sin_half * axis.y,
            z: sin_half * axis.z,
            w: cos_half,
        }
    }

    // TODO rotate by
}

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

impl Mat4 {
    #[inline]
    pub fn mul(&self, mat: &Mat4) -> Mat4 {
        Mat4 {
            _00: self._00 * mat._00 + self._01 * mat._10 + self._02 * mat._20 + self._03 * mat._30,
            _01: self._00 * mat._01 + self._01 * mat._11 + self._02 * mat._21 + self._03 * mat._31,
            _02: self._00 * mat._02 + self._01 * mat._12 + self._02 * mat._22 + self._03 * mat._32,
            _03: self._00 * mat._03 + self._01 * mat._13 + self._02 * mat._23 + self._03 * mat._33,
            _10: self._10 * mat._00 + self._11 * mat._10 + self._12 * mat._20 + self._13 * mat._30,
            _11: self._10 * mat._01 + self._11 * mat._11 + self._12 * mat._21 + self._13 * mat._31,
            _12: self._10 * mat._02 + self._11 * mat._12 + self._12 * mat._22 + self._13 * mat._32,
            _13: self._10 * mat._03 + self._11 * mat._13 + self._12 * mat._23 + self._13 * mat._33,
            _20: self._20 * mat._00 + self._21 * mat._10 + self._22 * mat._20 + self._23 * mat._30,
            _21: self._20 * mat._01 + self._21 * mat._11 + self._22 * mat._21 + self._23 * mat._31,
            _22: self._20 * mat._02 + self._21 * mat._12 + self._22 * mat._22 + self._23 * mat._32,
            _23: self._20 * mat._03 + self._21 * mat._13 + self._22 * mat._23 + self._23 * mat._33,
            _30: self._30 * mat._00 + self._31 * mat._10 + self._32 * mat._20 + self._33 * mat._30,
            _31: self._30 * mat._01 + self._31 * mat._11 + self._32 * mat._21 + self._33 * mat._31,
            _32: self._30 * mat._02 + self._31 * mat._12 + self._32 * mat._22 + self._33 * mat._32,
            _33: self._30 * mat._03 + self._31 * mat._13 + self._32 * mat._23 + self._33 * mat._33,
        }
    }

    #[inline]
    pub fn mul_vec(&self, vec: &Vec4) -> Vec4 {
        Vec4 {
            x: self._00 * vec.x + self._01 * vec.y + self._02 * vec.z + self._03 * vec.w,
            y: self._10 * vec.x + self._11 * vec.y + self._12 * vec.z + self._13 * vec.w,
            z: self._20 * vec.x + self._21 * vec.y + self._22 * vec.z + self._23 * vec.w,
            w: self._30 * vec.x + self._31 * vec.y + self._32 * vec.z + self._33 * vec.w,
        }
    }

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
    pub fn inverted(&self) -> Result<Mat4> {
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

        if inv_det == 0.0 {
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
}
