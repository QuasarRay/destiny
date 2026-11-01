
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
            // Source Vector3d::Normalize returns unchanged for a zero-length
            // axis, whereas DVec3::normalize returns non-finite components.
            let norm = axis.length();
            if norm != 0.0 {
                axis *= 1.0 / norm;
            }
        }

        let half_angle = angle * 0.5;

        Self {
            w: half_angle.cos(),
            v: axis * half_angle.sin(),
        }
    }

    /// Original C++ `QuaternionD::LengthSq`, preserving the original owner
    /// and name. `DVec3` implements the vector-length operation below.
    #[allow(non_snake_case)]
    pub fn LengthSq(self) -> f64 {
        self.w * self.w + self.v.length_squared()
    }

    /// Previous Rust spelling retained for callers.
    pub fn length_squared(self) -> f64 {
        self.LengthSq()
    }

    /// Mathematical inverse, including for non-unit quaternions.
    ///
    /// A zero-length quaternion has no inverse.
    pub fn inverse(self) -> Self {
        let length_sq = self.LengthSq();

        Self {
            w: self.w,
            v: -self.v,
        } / length_sq
    }

    /// Original C++ `QuaternionD::RotateVector` calculation:
    /// `(*this) * QuaternionD(0.0, v) * (-*this)`.
    #[allow(non_snake_case)]
    pub fn RotateVector(self, vector: DVec3) -> DVec3 {
        (self * Self::from_scalar_vector(0.0, vector) * (-self)).v
    }

    /// Previous Rust spelling retained for callers.
    pub fn rotate_vector(self, vector: DVec3) -> DVec3 {
        self.RotateVector(vector)
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
        // Preserve Quaternion.h's exact scalar operation order; avoid
        // abstracting through Bevy dot/cross which may reorder floating math.
        Self::new(
            self.w * rhs.w - self.v.x * rhs.v.x - self.v.y * rhs.v.y - self.v.z * rhs.v.z,
            self.w * rhs.v.x + self.v.x * rhs.w + self.v.y * rhs.v.z - self.v.z * rhs.v.y,
            self.w * rhs.v.y + self.v.y * rhs.w + self.v.z * rhs.v.x - self.v.x * rhs.v.z,
            self.w * rhs.v.z + self.v.z * rhs.w + self.v.x * rhs.v.y - self.v.y * rhs.v.x,
        )
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
