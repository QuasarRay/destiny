// Copyright © 2015 CCP ehf. (original Quaternion.h).
//
// Direct class correspondence: C++ QuaternionD <-> Rust QuaternionD.
// The sole external representation substitution is Vector3d -> Bevy DVec3.
// No ECS Component, Resource, System, or Relationship is introduced because
// Quaternion.h defines none. Bevy DQuat conversions below are boundary
// adapters, not additional Destiny quaternion operations.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use bevy::math::{DQuat, DVec3};

/// Source: src/Quaternion.h, typedef struct QuaternionD.
///
/// A general (not necessarily unit) quaternion. Using Bevy's DQuat as the
/// internal representation would change the original arbitrary-quaternion
/// value model; DVec3 is used only for the C++ Vector3d field.
///
/// Rust cannot overload the C++ constructors. Each of the original four
/// constructors has exactly one Rust entry point, documented below.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuaternionD {
    pub w: f64,
    pub v: DVec3,
}

impl QuaternionD {
    /// Original QuaternionD(double _w, double _x, double _y, double _z).
    #[must_use]
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, v: DVec3::new(x, y, z) }
    }

    /// Original QuaternionD(double _w, const Vector3d& _v).
    #[must_use]
    pub const fn from_scalar_vector(w: f64, v: &DVec3) -> Self {
        Self { w, v: *v }
    }

    /// Original QuaternionD(Vector3d axis, double angle).
    ///
    /// This deliberately reproduces Vector3d::LengthSq, Normalize, and the
    /// sin(angle * 0.5) * axis evaluation rather than calling DVec3::normalize.
    /// In particular, Normalize leaves a zero-length axis unchanged.
    #[must_use]
    pub fn from_axis_angle(mut axis: DVec3, angle: f64) -> Self {
        if (axis.x * axis.x + axis.y * axis.y + axis.z * axis.z) != 1.0 {
            let mut norm =
                (axis.x * axis.x + axis.y * axis.y + axis.z * axis.z).sqrt();
            if norm != 0.0 {
                norm = 1.0 / norm;
                axis.x *= norm;
                axis.y *= norm;
                axis.z *= norm;
            }
        }
        // Same operation order as the two distinct expressions in Quaternion.h.
        let w = (angle * 0.5).cos();
        let scalar = (angle * 0.5).sin();
        let v = DVec3::new(axis.x * scalar, axis.y * scalar, axis.z * scalar);
        Self { w, v }
    }

    /// Original double QuaternionD::LengthSq() const.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn LengthSq(&self) -> f64 {
        self.w * self.w + (self.v.x * self.v.x + self.v.y * self.v.y + self.v.z * self.v.z)
    }

    /// Original Vector3d QuaternionD::RotateVector(const Vector3d& v) const.
    /// C++ unary - means inverse, not componentwise quaternion negation.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn RotateVector(&self, vector: &DVec3) -> DVec3 {
        let q = *self * Self::from_scalar_vector(0.0, vector) * (-*self);
        q.v
    }
}

/// Original QuaternionD() : w(1.0), with Vector3d() initializing x/y/z=0.
impl Default for QuaternionD {
    fn default() -> Self {
        Self { w: 1.0, v: DVec3::ZERO }
    }
}

/// Original QuaternionD::operator+=(const QuaternionD&).
impl AddAssign for QuaternionD {
    fn add_assign(&mut self, q: Self) {
        self.w += q.w;
        self.v.x += q.v.x;
        self.v.y += q.v.y;
        self.v.z += q.v.z;
    }
}

/// Original QuaternionD::operator-=(const QuaternionD&).
impl SubAssign for QuaternionD {
    fn sub_assign(&mut self, q: Self) {
        self.w -= q.w;
        self.v.x -= q.v.x;
        self.v.y -= q.v.y;
        self.v.z -= q.v.z;
    }
}

/// Original QuaternionD::operator-() const: returns the INVERSE.
///
/// The original does not reject a zero quaternion, so the same IEEE-754
/// division-by-zero behavior is preserved instead of adding an error.
impl Neg for QuaternionD {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::from_scalar_vector(self.w, &(-self.v)) / self.LengthSq()
    }
}

/// Original QuaternionD::operator/(double c) const.
///
/// First computes c = 1 / c, then scales each component.
impl Div<f64> for QuaternionD {
    type Output = Self;
    fn div(self, mut c: f64) -> Self::Output {
        c = 1.0 / c;
        Self::new(self.w * c, self.v.x * c, self.v.y * c, self.v.z * c)
    }
}

/// Original QuaternionD::operator*(const QuaternionD&) const.
/// Preserve the C++ scalar expression and associativity, not a DQuat product.
impl Mul for QuaternionD {
    type Output = Self;
    fn mul(self, q: Self) -> Self::Output {
        Self::new(
            self.w * q.w - self.v.x * q.v.x - self.v.y * q.v.y - self.v.z * q.v.z,
            self.w * q.v.x + self.v.x * q.w + self.v.y * q.v.z - self.v.z * q.v.y,
            self.w * q.v.y + self.v.y * q.w + self.v.z * q.v.x - self.v.x * q.v.z,
            self.w * q.v.z + self.v.z * q.w + self.v.x * q.v.y - self.v.y * q.v.x,
        )
    }
}

/// Original QuaternionD::operator+(const QuaternionD&) const.
impl Add for QuaternionD {
    type Output = Self;
    fn add(self, q: Self) -> Self::Output {
        Self::new(self.w + q.w, self.v.x + q.v.x, self.v.y + q.v.y, self.v.z + q.v.z)
    }
}

/// Original QuaternionD::operator-(const QuaternionD&) const.
impl Sub for QuaternionD {
    type Output = Self;
    fn sub(self, q: Self) -> Self::Output {
        Self::new(self.w - q.w, self.v.x - q.v.x, self.v.y - q.v.y, self.v.z - q.v.z)
    }
}

/// Original QuaternionD::operator== and operator!=.
///
/// Implemented explicitly instead of delegating comparison to Bevy DVec3 so
/// both C++ memberwise comparison predicates have direct counterparts.
impl PartialEq for QuaternionD {
    fn eq(&self, q: &Self) -> bool {
        self.w == q.w
            && self.v.x == q.v.x
            && self.v.y == q.v.y
            && self.v.z == q.v.z
    }

    fn ne(&self, q: &Self) -> bool {
        self.w != q.w
            || self.v.x != q.v.x
            || self.v.y != q.v.y
            || self.v.z != q.v.z
    }
}

// ---------- Explicit boundary to pure Bevy quaternion APIs ----------
// These conversions have no counterparts in Quaternion.h and are not counted
// as original Destiny methods. They do not normalize or change the four
// floating-point components.
impl From<DQuat> for QuaternionD {
    fn from(q: DQuat) -> Self {
        Self::new(q.w, q.x, q.y, q.z)
    }
}
impl From<QuaternionD> for DQuat {
    fn from(q: QuaternionD) -> Self {
        DQuat::from_xyzw(q.v.x, q.v.y, q.v.z, q.w)
    }
}
