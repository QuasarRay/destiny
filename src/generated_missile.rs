// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// @generated: source missile AST and scalar representation glue. Do not edit.
// Original CCP ehf. MIT source; bindings: verification/migration/missile_source_evidence.json.
// Regenerate: python tools/migrate_destiny.py generate
use destiny_original_spec::{
    OriginalMode, OriginalParkBall, OriginalParkError, Vec3, original_goto_thrust,
};
use serde::{Deserialize, Serialize};
pub fn thrust(
    ball: &mut OriginalParkBall,
    other: &OriginalParkBall,
    current_time: i64,
    friction: f64,
    dt: f64,
    tick_interval_ms: i64,
) -> Result<Vec3, OriginalParkError> {
    let Some(elapsed) = current_time
        .checked_sub(ball.effect_stamp)
        .and_then(|n| n.checked_mul(tick_interval_ms))
    else {
        return Err(OriginalParkError::OutsideDeclaredScope);
    };
    if elapsed <= 800 {
        return Ok(original_goto_thrust(ball, ball.goto, friction, dt, false));
    }
    let delta = ball.position.sub(other.position);
    let distance = delta.length();
    let intercept_time = distance / f64::from(ball.max_velocity);
    let mut target = other.position.add(other.velocity.scale(intercept_time));
    let radius = f64::from(ball.follow_range) + f64::from(ball.radius) + f64::from(other.radius);
    if distance == 0.0 {
        target = other.position.add(
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }
            .scale(radius),
        );
    } else {
        target = target.add(delta.scale(radius).scale(1.0 / distance));
    }
    ball.goto = target;
    Ok(original_goto_thrust(
        ball,
        target,
        friction,
        dt,
        ball.mode == OriginalMode::Missile,
    ))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceFloat32(#[serde(with = "float32")] pub f32);

pub mod float32 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Bits {
        source_f32_bits_v1: u32,
    }
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Stored {
        Bits(Bits),
        Numeric(f64),
    }
    pub fn serialize<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
        if value.is_finite() {
            serializer.serialize_f64(f64::from(*value))
        } else {
            Bits {
                source_f32_bits_v1: value.to_bits(),
            }
            .serialize(serializer)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
        Ok(match Stored::deserialize(deserializer)? {
            Stored::Bits(value) => f32::from_bits(value.source_f32_bits_v1),
            Stored::Numeric(value) => value as f32,
        })
    }
}
