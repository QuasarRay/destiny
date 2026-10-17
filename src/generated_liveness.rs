// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
use avian3d::prelude::RigidBody;
use bevy::prelude::{Component, Entity, Resource, World};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivenessState {
    pub registered: BTreeSet<i64>,
    pub keep_alive: Vec<(i64, BTreeSet<i64>)>,
    pub interactives: Vec<(i64, i64)>,
}
#[derive(Resource, Default)]
pub(crate) struct Frame {
    pub cached_dead: BTreeSet<Entity>,
    pub projected: Vec<(Entity, RigidBody)>,
    pub contact_locations: crate::iterative_collision::ContactTrace,
}
pub(crate) fn project_components<T: Component + Copy>(
    world: &mut World,
    entities: &BTreeSet<Entity>,
    replacement: T,
) -> Vec<(Entity, T)> {
    let mut saved = Vec::new();
    for entity in entities {
        if let Some(value) = world.get::<T>(*entity).copied() {
            saved.push((*entity, value));
            world.entity_mut(*entity).insert(replacement);
        }
    }
    saved
}
pub(crate) fn restore_components<T: Component>(world: &mut World, saved: Vec<(Entity, T)>) {
    for (entity, value) in saved {
        if let Ok(mut target) = world.get_entity_mut(entity) {
            target.insert(value);
        }
    }
}
#[doc = "src/Ballpark.cpp:6216-6259; valid integer/set state"]
pub fn is_dead(master: bool, mode: u32, interactive_count: usize, keep_alive_count: usize) -> bool {
    (master)
        && (((mode) != (3_u32))
            && (((interactive_count) == (0_usize)) && ((keep_alive_count) == (0_usize))))
}

impl Frame {
    #[doc = "src/Ballpark.cpp:617-626; derived ordered ID projection only"]
    pub(crate) fn contact_ids(&self, id: i64) -> Vec<i64> {
        ((((self.contact_locations).iter()).find(|row| (row.0) == (id))).map(|row| (row.1).clone()))
            .unwrap_or_default()
    }
}
#[cfg(test)]
#[path = "../tests/support/liveness_tracking.rs"]
mod tests;
