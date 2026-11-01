
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use bevy::math::{DQuat, DVec3};

/// Double-precision quaternion compatible with Destiny's
/// original C++ QuaternionD.
///
/// Unlike Bevy's DQuat, this type also supports arbitrary
/// non-unit quaternions and preserves the C++ inverse operator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuaternionD {
    pub w: f64,
    pub v: DVec3,
}

impl QuaternionD {
    pub const IDENTITY: Self = Self {
        w: 1.0,
        v: DVec3::ZERO,
    };

    /// Equivalent to QuaternionD(w, x, y, z).
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self {
            w,
            v: DVec3::new(x, y, z),
        }
    }

    /// Equivalent to QuaternionD(w, Vector3d).
    pub const fn from_scalar_vector(w: f64, v: DVec3) -> Self {
        Self { w, v }
    }

    /// Equivalent to QuaternionD(axis, angle).
    ///
    /// The angle is in radians. The axis must be nonzero.
    pub fn from_axis_angle(mut axis: DVec3, angle: f64) -> Self {
        if axis.length_squared() != 1.0 {
            axis = axis.normalize();
        }

        let half_angle = angle * 0.5;

        Self {
            w: half_angle.cos(),
            v: axis * half_angle.sin(),
        }
    }

    /// Equivalent to LengthSq().
    pub fn length_squared(self) -> f64 {
        self.w * self.w + self.v.length_squared()
    }

    /// Mathematical inverse, including for non-unit quaternions.
    ///
    /// A zero-length quaternion has no inverse.
    pub fn inverse(self) -> Self {
        let length_sq = self.length_squared();

        Self {
            w: self.w,
            v: -self.v,
        } / length_sq
    }

    /// Equivalent to RotateVector().
    ///
    /// Uses q * (0, vector) * q.inverse(), preserving
    /// the original implementation's behavior.
    pub fn rotate_vector(self, vector: DVec3) -> DVec3 {
        let pure = Self::from_scalar_vector(0.0, vector);
        (self * pure * self.inverse()).v
    }
}

impl Default for QuaternionD {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// C++ operator +=
impl AddAssign for QuaternionD {
    fn add_assign(&mut self, rhs: Self) {
        self.w += rhs.w;
        self.v += rhs.v;
    }
}

// C++ operator -=
impl SubAssign for QuaternionD {
    fn sub_assign(&mut self, rhs: Self) {
        self.w -= rhs.w;
        self.v -= rhs.v;
    }
}

// C++ unary operator -
// IMPORTANT: This is inverse, NOT componentwise negation.
impl Neg for QuaternionD {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.inverse()
    }
}

// C++ operator / (double)
impl Div<f64> for QuaternionD {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let reciprocal = 1.0 / rhs;

        Self {
            w: self.w * reciprocal,
            v: self.v * reciprocal,
        }
    }
}

// Hamilton quaternion multiplication.
impl Mul for QuaternionD {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.v.dot(rhs.v),
            v: self.w * rhs.v
                + rhs.w * self.v
                + self.v.cross(rhs.v),
        }
    }
}

// C++ operator +
impl Add for QuaternionD {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w + rhs.w,
            v: self.v + rhs.v,
        }
    }
}

// C++ binary operator -
impl Sub for QuaternionD {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w - rhs.w,
            v: self.v - rhs.v,
        }
    }
}

// Convert from native Bevy DQuat.
impl From<DQuat> for QuaternionD {
    fn from(q: DQuat) -> Self {
        Self {
            w: q.w,
            v: q.xyz(),
        }
    }
}

// Convert into native Bevy DQuat.
// This conversion does not normalize the quaternion.
impl From<QuaternionD> for DQuat {
    fn from(q: QuaternionD) -> Self {
        DQuat::from_xyzw(q.v.x, q.v.y, q.v.z, q.w)
    }
}
