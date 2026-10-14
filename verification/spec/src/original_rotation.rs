// Copyright (c) 2026 CCP Games
// Copyright (c) Microsoft Corporation. All rights reserved.
// MIT notices: verification/licenses/carbon-math.txt and directxmath.txt.
//! Pinned Carbon math matrix/coordinate operations and DirectXMath apr2025's
//! SSE2, non-FMA rotation branch, expressed as binary32 scalar operations.
//! Other target branches, compiler contraction/rounding mode and native
//! object/list identity correspondence remain separate obligations.

use crate::{OriginalParkBall, OriginalParkError, Vec3};

/// Two Python-list objects with three exact float members. Handles preserve
/// aliasing of PyGetRotatedVector's in-place writes and returned input list.
/// Python allocation/reference counting and arbitrary element types/lengths
/// remain external correspondence obligations, not successful Rust cases.
#[derive(Default)]
pub struct OriginalPythonVectorLists {
    lists: [Option<[f64; 3]>; 2],
}
impl OriginalPythonVectorLists {
    pub fn allocate(&mut self, values: [f64; 3]) -> Result<usize, OriginalParkError> {
        let mut i = 0;
        while i < 2 {
            if self.lists[i].is_none() {
                self.lists[i] = Some(values);
                return Ok(i);
            }
            i += 1;
        }
        Err(OriginalParkError::OutsideDeclaredScope)
    }
    #[must_use]
    pub fn values(&self, handle: usize) -> Option<[f64; 3]> {
        self.lists.get(handle).copied().flatten()
    }
    pub fn get_rotated_vector(
        &mut self,
        ball: &OriginalParkBall,
        handle: usize,
    ) -> Result<usize, OriginalParkError> {
        let values = self
            .values(handle)
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        let vector = original_rotated_vector(
            ball,
            Vec3 {
                x: values[0],
                y: values[1],
                z: values[2],
            },
        );
        // PyList_SET_ITEM(list,0..2), then Py_INCREF(list), then return list.
        let slot = self
            .lists
            .get_mut(handle)
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        *slot = Some([vector.x, vector.y, vector.z]);
        Ok(handle)
    }
}

fn directx_round(value: f32) -> f32 {
    // XMVectorRound SSE2, including its signed magic and large/NaN mask.
    let magic = f32::from_bits(8_388_608.0_f32.to_bits() | (value.to_bits() & 0x8000_0000));
    let rounded = (value + magic) - magic;
    if value.abs() <= 8_388_608.0 {
        rounded
    } else {
        value
    }
}

fn directx_sin_cos(angle: f32) -> (f32, f32) {
    let turns = directx_round(angle * 0.159154943_f32);
    let reduced = angle - turns * 6.283185307_f32;
    let signed_pi = f32::from_bits(3.141592654_f32.to_bits() | (reduced.to_bits() & 0x8000_0000));
    let (x, sign) = if reduced.abs() <= 1.570796327_f32 {
        (reduced, 1.0_f32)
    } else {
        (signed_pi - reduced, -1.0_f32)
    };
    let square = x * x;
    let mut sine = -2.3889859e-08_f32 * square + 2.7525562e-06_f32;
    sine = sine * square - 0.00019840874_f32;
    sine = sine * square + 0.0083333310_f32;
    sine = sine * square - 0.16666667_f32;
    sine = (sine * square + 1.0_f32) * x;
    let mut cosine = -2.6051615e-07_f32 * square + 2.4760495e-05_f32;
    cosine = cosine * square - 0.0013888378_f32;
    cosine = cosine * square + 0.041666638_f32;
    cosine = cosine * square - 0.5_f32;
    cosine = (cosine * square + 1.0_f32) * sign;
    (sine, cosine)
}

/// Carbon RotationMatrix(RotationQuaternion(float(yaw),float(pitch),float(roll)))
/// followed by TransformCoord. The argument and every matrix operation narrow
/// to binary32, and each quaternion lane follows DirectX's permute/multiply/add
/// ordering. This is the declared Linux SSE2 branch without FMA or SVML.
#[must_use]
pub fn original_rotated_vector(ball: &OriginalParkBall, coordinate: Vec3) -> Vec3 {
    let (sp, cp) = directx_sin_cos((ball.pitch as f32) * 0.5_f32);
    let (sy, cy) = directx_sin_cos((ball.yaw as f32) * 0.5_f32);
    let (sr, cr) = directx_sin_cos((ball.roll as f32) * 0.5_f32);
    let qx = (cp * sy) * sr + (sp * cy) * cr;
    let qy = ((-sp) * cy) * sr + (cp * sy) * cr;
    let qz = ((-sp) * sy) * cr + (cp * cy) * sr;
    let qw = (sp * sy) * sr + (cp * cy) * cr;
    let m11 = 1.0_f32 - 2.0_f32 * (qy * qy + qz * qz);
    let m12 = 2.0_f32 * (qx * qy + qz * qw);
    let m13 = 2.0_f32 * (qx * qz - qy * qw);
    let m21 = 2.0_f32 * (qx * qy - qz * qw);
    let m22 = 1.0_f32 - 2.0_f32 * (qx * qx + qz * qz);
    let m23 = 2.0_f32 * (qy * qz + qx * qw);
    let m31 = 2.0_f32 * (qx * qz + qy * qw);
    let m32 = 2.0_f32 * (qy * qz - qx * qw);
    let m33 = 1.0_f32 - 2.0_f32 * (qx * qx + qy * qy);
    let x = coordinate.x as f32;
    let y = coordinate.y as f32;
    let z = coordinate.z as f32;
    // Keep even the homogeneous zero products: nonfinite coordinates affect
    // the original norm, and replacing it unconditionally by one is unsound.
    let norm = ((0.0_f32 * x + 0.0_f32 * y) + 0.0_f32 * z) + 1.0_f32;
    if norm != 0.0_f32 {
        Vec3 {
            x: f64::from((((x * m11 + y * m21) + z * m31) + 0.0_f32) / norm),
            y: f64::from((((x * m12 + y * m22) + z * m32) + 0.0_f32) / norm),
            z: f64::from((((x * m13 + y * m23) + z * m33) + 0.0_f32) / norm),
        }
    } else {
        Vec3::ZERO
    }
}

pub fn original_formation_thrust(
    ball: &mut OriginalParkBall,
    other: &OriginalParkBall,
) -> Result<Vec3, OriginalParkError> {
    let points = match other.formation_id {
        0 => [
            Vec3 {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 100.0,
                z: 0.0,
            },
            Vec3 {
                x: -100.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: -100.0,
                z: 0.0,
            },
        ],
        1 => [
            Vec3 {
                x: 100.0,
                y: 0.0,
                z: -50.0,
            },
            Vec3 {
                x: 50.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: -100.0,
                y: 0.0,
                z: -50.0,
            },
            Vec3 {
                x: -50.0,
                y: 0.0,
                z: 0.0,
            },
        ],
        2 => [Vec3::ZERO; 4],
        _ => return Ok(Vec3::ZERO),
    };
    let slot = ball.effect_stamp as i8;
    let count = if other.formation_id == 2 { 1 } else { 4 };
    if i32::from(slot) >= count {
        return Ok(Vec3::ZERO);
    }
    // Negative signed-char indexing is undefined in the original C++ vector;
    // it is an explicit caller obligation, never successful omission.
    if slot < 0 {
        return Err(OriginalParkError::OutsideDeclaredScope);
    }
    let Some(offset) = points.get(slot as usize).copied() else {
        return Err(OriginalParkError::OutsideDeclaredScope);
    };
    let mut length = offset.length();
    if length == 0.0 {
        length = 1.0;
    }
    let vector = offset
        .scale(f64::from(ball.radius) + f64::from(other.radius) + length)
        .scale(1.0 / length);
    let target = other.position.add(original_rotated_vector(other, vector));
    ball.goto = target;
    Ok(crate::original_goto_thrust(
        ball,
        target,
        1_000_000.0,
        1.0,
        false,
    ))
}
