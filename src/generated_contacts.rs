// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
#[derive(Debug, Clone, Copy)]
pub struct CollisionEvent {
    pub ball_id: i64,
    pub other_id: i64,
    pub acceleration: [f64; 3],
}
#[derive(Debug, Clone, Copy)]
pub enum CollisionNotice {
    Contact(CollisionEvent),
    HostError { ball_id: i64 },
}
#[derive(Debug)]
pub struct CollisionPass {
    pub(crate) world: bevy::ecs::world::WorldId,
    pub(crate) entity: bevy::prelude::Entity,
    pub(crate) owner: i64,
    pub(crate) generation: u64,
    pub(crate) limit: usize,
    pub(crate) index: usize,
    pub(crate) failures: usize,
}
impl CollisionPass {
    #[doc = "src/Ball.cpp:468-488; capture size once, reread mCollisions[i]/mLastC at each SendEvent, PyError then continue; typed host boundary"]
    pub fn delivered_count(&self) -> usize {
        self.index
    }
}
impl CollisionPass {
    #[doc = "src/Ball.cpp:468-488; capture size once, reread mCollisions[i]/mLastC at each SendEvent, PyError then continue; typed host boundary"]
    pub fn captured_count(&self) -> usize {
        self.limit
    }
}
impl CollisionPass {
    #[doc = "src/Ball.cpp:468-488; capture size once, reread mCollisions[i]/mLastC at each SendEvent, PyError then continue; typed host boundary"]
    pub fn host_error_count(&self) -> usize {
        self.failures
    }
}
