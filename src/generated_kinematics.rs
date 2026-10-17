// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(from = "PoseBits", into = "PoseBits")]
pub struct Pose {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PoseBits {
    position: [u64; 3],
    velocity: [u64; 3],
}
impl From<Pose> for PoseBits {
    fn from(p: Pose) -> Self {
        Self {
            position: p.position.map(f64::to_bits),
            velocity: p.velocity.map(f64::to_bits),
        }
    }
}
impl From<PoseBits> for Pose {
    fn from(p: PoseBits) -> Self {
        Self {
            position: p.position.map(f64::from_bits),
            velocity: p.velocity.map(f64::from_bits),
        }
    }
}
impl Pose {
    #[doc = "src/Ballpark.cpp:3356-3371; new object only; existing Vec3 sub/scale matches Vector3d operators"]
    pub(crate) fn created(position: [f64; 3usize], velocity: [f64; 3usize], dt: f64) -> Self {
        let previous = (destiny_original_spec::Vec3 {
            x: position[0_usize],
            y: position[1_usize],
            z: position[2_usize],
        })
        .sub(
            (destiny_original_spec::Vec3 {
                x: velocity[0_usize],
                y: velocity[1_usize],
                z: velocity[2_usize],
            })
            .scale(dt),
        );
        Self {
            position: [previous.x, previous.y, previous.z],
            velocity,
        }
    }
}
impl Pose {
    #[doc = "src/Vector3d.h:8-18; field representation mapping"]
    pub(crate) fn position_vector(self) -> destiny_original_spec::Vec3 {
        destiny_original_spec::Vec3 {
            x: self.position[0_usize],
            y: self.position[1_usize],
            z: self.position[2_usize],
        }
    }
}
impl Pose {
    #[doc = "src/Vector3d.h:8-18; field representation mapping"]
    pub(crate) fn velocity_vector(self) -> destiny_original_spec::Vec3 {
        destiny_original_spec::Vec3 {
            x: self.velocity[0_usize],
            y: self.velocity[1_usize],
            z: self.velocity[2_usize],
        }
    }
}
impl Pose {
    #[doc = "src/Ball.cpp:1484; Vector3d.h:22,46; reciprocal multiplication then f32 Vector3 components"]
    pub(crate) fn acceleration(self, current: [f64; 3usize], dt: f64) -> [f64; 3usize] {
        let delta = (destiny_original_spec::Vec3 {
            x: current[0_usize],
            y: current[1_usize],
            z: current[2_usize],
        })
        .sub((self).velocity_vector());
        let scaled = (delta).scale((1.0_f64) / (dt));
        [
            ((scaled.x) as f32) as f64,
            ((scaled.y) as f32) as f64,
            ((scaled.z) as f32) as f64,
        ]
    }
}
