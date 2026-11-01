// Copyright © 2015 CCP ehf.
// Source: StaticCollidable.h, StaticCollidable.cpp, SortedSets.h
//
// The original class inherits Partitionable; Bevy represents the original
// base subobject as a second component on the SAME Entity. This is an
// explicit C++ inheritance -> ECS boundary, not a duplicated ballpark.
use bevy::{math::DVec3, prelude::{Component, Entity, Mut, World}};
use crate::{partition::Partition, partitionable::Partitionable};

/// Canonical C++ StaticCollidable::mParentBallId and constructor default.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct StaticCollidable {
    pub mParentBallId: i64,
}

/// Original pure virtual collision geometry contract; concrete implementations
/// (Capsule, OrientedBox, etc.) have not been ported. Bevy Entity is the
/// boundary translation of C++ Ball*.
#[allow(non_snake_case)]
pub trait StaticCollidableGeometry {
    fn GetCenter(&self) -> DVec3;
    fn CollideWithBall(&self, world: &mut World, ball: Entity);
    fn CheckCollision(
        &self, p0: &DVec3, p1: &DVec3, radius: f32,
        normal: &mut DVec3, timeOfImpact: &mut f64,
    ) -> bool;
    fn GetBoundingRadius(&self) -> f32;
}

#[allow(non_snake_case)]
impl StaticCollidable {
    /// Original StaticCollidable::ClearBoxes clears links owned by the object.
    /// It DOES NOT remove the object from each box.
    pub fn ClearBoxes(world: &mut World, object: Entity) {
        if let Some(mut state) = world.get_mut::<Partitionable>(object) {
            state.forget_box_references();
        }
    }

    /// Remove from every Box using a snapshot; removal may free empty boxes.
    pub fn DeleteFromBoxes(world: &mut World, object: Entity) {
        let boxes: Vec<Entity> = world
            .get::<Partitionable>(object)
            .map(|state| state.boxes().map(|(_, box_entity)| box_entity).collect())
            .unwrap_or_default();
        if world.contains_resource::<Partition>() {
            for box_entity in boxes {
                world.resource_scope(|world, mut partition: Mut<'_, Partition>| {
                    partition.remove_static(world, box_entity, object);
                });
            }
        }
        Self::ClearBoxes(world, object);
    }

    /// C++ nullable Box* is an Option<Entity>, with both links updated via
    /// the existing authoritative Partition index.
    pub fn InsertInBox(world: &mut World, object: Entity, box_entity: Option<Entity>) {
        let Some(box_entity) = box_entity else { return };
        if world.contains_resource::<Partition>() {
            world.resource_scope(|world, partition: Mut<'_, Partition>| {
                partition.insert_static(world, box_entity, object);
            });
        }
    }

    /// Original C++ StaticCollidable::SetBoxActive has an empty body.
    pub fn SetBoxActive(_box_id: i32, _active: bool) {}

    /// C++ ignores dt and returns the dynamically-dispatched bounding radius.
    /// The geometry provider is explicit until concrete collider classes exist.
    pub fn GetInflatedRadius<G: StaticCollidableGeometry>(
        geometry: &G, _dt: f64,
    ) -> f64 {
        f64::from(geometry.GetBoundingRadius())
    }
}

/// Original StaticCollidableSortComparer from SortedSets.h and its implementation
/// in StaticCollidable.cpp. Equal parent IDs are ordered by object ID.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticCollidableSortComparer;
impl StaticCollidableSortComparer {
    pub fn less(&self, world: &World, lhs: Entity, rhs: Entity) -> Option<bool> {
        let a = world.get::<StaticCollidable>(lhs)?;
        let b = world.get::<StaticCollidable>(rhs)?;
        let a_state = world.get::<Partitionable>(lhs)?;
        let b_state = world.get::<Partitionable>(rhs)?;
        Some((a.mParentBallId, a_state.mId) < (b.mParentBallId, b_state.mId))
    }
}
