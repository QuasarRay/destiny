// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// @generated: source-bound angular AST transplantation. Do not edit.
// Copyright CCP ehf.; MIT. Source and rule hashes: verification/migration/angular_source_evidence.json.
// Regenerate: python tools/migrate_destiny.py generate
use destiny_original_spec::{OriginalMode, OriginalParkBall, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "AngleBits", into = "AngleBits")]
pub struct Angles {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub old_yaw: f64,
    pub old_pitch: f64,
    pub old_roll: f64,
    pub roll_speed: f64,
    pub old_roll_speed: f64,
    pub yaw_delta: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AngleBits {
    yaw: u64,
    pitch: u64,
    roll: u64,
    old_yaw: u64,
    old_pitch: u64,
    old_roll: u64,
    roll_speed: u64,
    old_roll_speed: u64,
    yaw_delta: u64,
}
impl From<Angles> for AngleBits {
    fn from(value: Angles) -> Self {
        Self {
            yaw: value.yaw.to_bits(),
            pitch: value.pitch.to_bits(),
            roll: value.roll.to_bits(),
            old_yaw: value.old_yaw.to_bits(),
            old_pitch: value.old_pitch.to_bits(),
            old_roll: value.old_roll.to_bits(),
            roll_speed: value.roll_speed.to_bits(),
            old_roll_speed: value.old_roll_speed.to_bits(),
            yaw_delta: value.yaw_delta.to_bits(),
        }
    }
}
impl From<AngleBits> for Angles {
    fn from(value: AngleBits) -> Self {
        Self {
            yaw: f64::from_bits(value.yaw),
            pitch: f64::from_bits(value.pitch),
            roll: f64::from_bits(value.roll),
            old_yaw: f64::from_bits(value.old_yaw),
            old_pitch: f64::from_bits(value.old_pitch),
            old_roll: f64::from_bits(value.old_roll),
            roll_speed: f64::from_bits(value.roll_speed),
            old_roll_speed: f64::from_bits(value.old_roll_speed),
            yaw_delta: f64::from_bits(value.yaw_delta),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollParameters {
    pub roll_acceleration: f64,
    pub roll_decay: f64,
    pub roll_speed_acceleration: f64,
    pub roll_speed_decay: f64,
}
impl Default for RollParameters {
    fn default() -> Self {
        Self {
            roll_acceleration: 1.0,
            roll_decay: -0.5,
            roll_speed_acceleration: -3.0,
            roll_speed_decay: -0.9,
        }
    }
}

pub struct Input {
    pub mass: f64,
    pub velocity: Vec3,
    pub position: Vec3,
    pub goto: Vec3,
    pub mode: OriginalMode,
    pub effect_stamp: i64,
    pub has_collisions: bool,
}
pub struct Environment {
    pub current_time: i64,
    pub friction: f64,
    pub dt: f64,
    pub roll_acceleration: f64,
    pub roll_decay: f64,
    pub roll_speed_acceleration: f64,
    pub roll_speed_decay: f64,
}

pub fn update(state: &mut Angles, input: &Input, environment: &Environment, snap: bool) -> bool {
    let mut scope_error = false;
    state.old_yaw = state.yaw;
    state.old_pitch = state.pitch;
    state.old_roll = state.roll;
    state.old_roll_speed = state.roll_speed;
    let mut mass_modifier = input.mass / environment.friction;
    mass_modifier = if mass_modifier != 0.0 {
        100.0 / mass_modifier
    } else {
        100.0
    };
    let velocity2 = input.velocity.norm_squared();
    let mut direction = Vec3::ZERO;
    if velocity2 == 0.0 {
        direction = input.goto.sub(input.position);
    } else {
        if input.mode == OriginalMode::Warp {
            if input.effect_stamp < 0 {
                direction = if !input.has_collisions {
                    input.goto.sub(input.position)
                } else {
                    input.velocity
                };
            } else if mass_modifier < 1.0 && environment.current_time != input.effect_stamp {
                if let Some(ticks) = environment.current_time.checked_sub(input.effect_stamp) {
                    let factor = (ticks as f64) * 2.0;
                    let inner = if factor < 15.0 { factor } else { 15.0 };
                    let modified = mass_modifier * inner;
                    mass_modifier = if modified < 1.0 { modified } else { 1.0 };
                } else {
                    scope_error = true;
                }
            }
        } else {
            direction = input.velocity;
        }
        if input.mode != OriginalMode::Missile && !snap {
            let offset = input.goto.sub(input.position).normalize();
            direction = direction.add(offset.scale(velocity2.sqrt() * 0.3));
        }
    }
    state.yaw = (direction.x).atan2(direction.z);
    let r = direction.length();
    state.pitch = if r > 0.0 {
        let ratio = direction.y / r;
        let lower = if -1.0 < ratio { ratio } else { -1.0 };
        -(if lower < 1.0 { lower } else { 1.0 }).asin()
    } else {
        0.0
    };
    modulo_2pi(&mut state.old_yaw, &mut state.yaw);
    modulo_2pi(&mut state.old_pitch, &mut state.pitch);
    if snap {
        state.roll = 0.0;
        state.old_roll = 0.0;
        state.roll_speed = 0.0;
        state.old_roll_speed = 0.0;
        state.yaw_delta = 0.0;
        return scope_error;
    }
    let reciprocal = 1.0 / (1.0 + mass_modifier * 0.5);
    state.yaw = (state.old_yaw + mass_modifier * 0.5 * state.yaw) * reciprocal;
    state.pitch = (state.old_pitch + mass_modifier * 0.5 * state.pitch) * reciprocal;
    state.yaw_delta = (state.yaw - state.old_yaw).sin().asin();
    state.roll_speed += state.yaw_delta * environment.roll_speed_acceleration;
    state.roll_speed += state.old_roll_speed * environment.roll_speed_decay * environment.dt;
    let half_pi = std::f64::consts::FRAC_PI_2;
    let upper_speed = if state.roll_speed < half_pi {
        state.roll_speed
    } else {
        half_pi
    };
    state.roll_speed = if -half_pi < upper_speed {
        upper_speed
    } else {
        -half_pi
    };
    state.roll += state.roll_speed * environment.roll_acceleration * environment.dt;
    state.roll += state.old_roll * environment.roll_decay * environment.dt;
    let upper_roll = if state.roll < half_pi {
        state.roll
    } else {
        half_pi
    };
    state.roll = if -half_pi < upper_roll {
        upper_roll
    } else {
        -half_pi
    };
    scope_error
}
fn modulo_2pi(old: &mut f64, new: &mut f64) {
    let period = std::f64::consts::TAU;
    *new %= period;
    *old %= period;
    if (*new - *old).abs() < (*new + period - *old).abs() {
        if (*new - *old).abs() >= (*new - period - *old).abs() {
            *new -= period;
        }
    } else if (*new + period - *old).abs() < (*new - period - *old).abs() {
        *new += period;
    } else {
        *new -= period;
    }
}
pub fn formation_thrust<F: Fn(Vec3) -> Vec3>(
    ball: &mut OriginalParkBall,
    other: &OriginalParkBall,
    offset: Vec3,
    friction: f64,
    dt: f64,
    rotate: F,
) -> Vec3 {
    let mut length = offset.length();
    if length == 0.0 {
        length = 1.0;
    }
    let vector = offset
        .scale(f64::from(ball.radius) + f64::from(other.radius) + length)
        .scale(1.0 / length);
    let target = other.position.add(rotate(vector));
    ball.goto = target;
    destiny_original_spec::original_goto_thrust(ball, target, friction, dt, false)
}

pub fn rotate(state: &Angles, coordinate: Vec3) -> Vec3 {
    destiny_original_spec::original_rotated_vector(
        &OriginalParkBall {
            yaw: state.yaw,
            pitch: state.pitch,
            roll: state.roll,
            ..OriginalParkBall::default()
        },
        coordinate,
    )
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod tests {
    use super::*;

    #[test]
    fn original_cpp_angular_transitions_match_exact_bits() {
        let report: serde_json::Value = serde_json::from_str(include_str!(
            "../verification/migration/angular_oracle_samples.json"
        ))
        .unwrap();
        let rows = report["rows"].as_array().unwrap();
        assert!(!rows.is_empty());
        for (index, row) in rows.iter().enumerate() {
            let encoded = row["state_bits"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        serde_json::json!(
                            u64::from_str_radix(value.as_str().unwrap(), 16).unwrap()
                        ),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let mut state: Angles =
                serde_json::from_value(serde_json::Value::Object(encoded)).unwrap();
            let values: Vec<f64> = serde_json::from_value(row["input"].clone()).unwrap();
            let mode = match row["mode"].as_u64().unwrap() {
                0 => OriginalMode::Goto,
                1 => OriginalMode::Follow,
                2 => OriginalMode::Stop,
                3 => OriginalMode::Warp,
                4 => OriginalMode::Orbit,
                5 => OriginalMode::Missile,
                6 => OriginalMode::Mushroom,
                7 => OriginalMode::Boid,
                8 => OriginalMode::Troll,
                9 => OriginalMode::Miniball,
                10 => OriginalMode::Field,
                11 => OriginalMode::Rigid,
                12 => OriginalMode::Formation,
                _ => panic!("unknown source mode"),
            };
            let input = Input {
                mass: values[0],
                velocity: Vec3 {
                    x: values[1],
                    y: values[2],
                    z: values[3],
                },
                position: Vec3 {
                    x: values[4],
                    y: values[5],
                    z: values[6],
                },
                goto: Vec3 {
                    x: values[7],
                    y: values[8],
                    z: values[9],
                },
                mode,
                effect_stamp: row["effect_stamp"].as_i64().unwrap(),
                has_collisions: row["has_collisions"].as_bool().unwrap(),
            };
            let environment = Environment {
                current_time: row["current_time"].as_i64().unwrap(),
                friction: values[10],
                dt: values[11],
                roll_acceleration: values[12],
                roll_decay: values[13],
                roll_speed_acceleration: values[14],
                roll_speed_decay: values[15],
            };
            assert!(!update(
                &mut state,
                &input,
                &environment,
                row["snap"].as_bool().unwrap()
            ));
            let actual = serde_json::to_value(state).unwrap();
            for (name, expected) in row["expected_bits"].as_object().unwrap() {
                assert_eq!(
                    actual[name].as_u64().unwrap(),
                    u64::from_str_radix(expected.as_str().unwrap(), 16).unwrap(),
                    "case {index} field {name}"
                );
            }
        }
    }

    #[test]
    fn angular_snapshot_preserves_nonfinite_payloads_and_signed_zero() {
        let words = [
            0x8000000000000000_u64,
            0x7ff0000000000000_u64,
            0xfff0000000000000_u64,
            0x7ff8000000001234_u64,
            0x7ff0000000004321_u64,
            0xfff8000000008765_u64,
            0x1_u64,
            0x10000000000000_u64,
            0x7fefffffffffffff_u64,
        ];
        let mut encoded = serde_json::to_value(Angles::default()).unwrap();
        for (slot, word) in encoded.as_object_mut().unwrap().values_mut().zip(words) {
            *slot = serde_json::json!(word);
        }
        let state: Angles = serde_json::from_value(encoded.clone()).unwrap();
        let restored: Angles =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
    }
}
