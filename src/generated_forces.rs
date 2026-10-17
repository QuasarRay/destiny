// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(from = "ForceBits", into = "ForceBits")]
pub struct Forces {
    pub last_g: [f64; 3],
    pub last_c: [f64; 3],
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ForceBits {
    last_g: [u64; 3],
    last_c: [u64; 3],
}
impl From<Forces> for ForceBits {
    fn from(value: Forces) -> Self {
        Self {
            last_g: value.last_g.map(f64::to_bits),
            last_c: value.last_c.map(f64::to_bits),
        }
    }
}
impl From<ForceBits> for Forces {
    fn from(value: ForceBits) -> Self {
        Self {
            last_g: value.last_g.map(f64::from_bits),
            last_c: value.last_c.map(f64::from_bits),
        }
    }
}
impl Forces {
    #[doc = "src/Ballpark.cpp:791-793; only after the source live/free control admission"]
    pub fn reset(&mut self) {
        self.last_g = [0.0_f64, 0.0_f64, 0.0_f64];
        self.last_c = [0.0_f64, 0.0_f64, 0.0_f64];
    }
}
impl Forces {
    #[doc = "src/Ball.h:162-163; binary64 Vector3d representation projection"]
    pub fn controller(&self) -> destiny_original_spec::Vec3 {
        destiny_original_spec::Vec3 {
            x: self.last_g[0_usize],
            y: self.last_g[1_usize],
            z: self.last_g[2_usize],
        }
    }
}
impl Forces {
    #[doc = "src/Ballpark.cpp:899"]
    pub fn set_controller(&mut self, value: destiny_original_spec::Vec3) {
        self.last_g = [value.x, value.y, value.z];
    }
}
impl Forces {
    #[doc = "src/Ball.h:162-163; binary64 Vector3d representation projection"]
    pub fn collision(&self) -> destiny_original_spec::Vec3 {
        destiny_original_spec::Vec3 {
            x: self.last_c[0_usize],
            y: self.last_c[1_usize],
            z: self.last_c[2_usize],
        }
    }
}
impl Forces {
    #[doc = "src/Ballpark.cpp:2493-2499,2995; actual source force publication"]
    pub fn set_collision(&mut self, value: destiny_original_spec::Vec3) {
        self.last_c = [value.x, value.y, value.z];
    }
}
