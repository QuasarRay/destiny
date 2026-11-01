// Copyright © 2014 CCP ehf. Original Box.cpp / Box.h.
// The ECS owns boxes as entities; Partition manages their index and lifetime.
use std::collections::BTreeMap;

use bevy::{math::DVec3, prelude::{Component, Entity, World}};
use crate::partitionable::BoxKey;
use crate::partition::Partition;
use bevy::prelude::Mut;
use pyo3::{prelude::*, types::PyAny};

type BallSortKey = (u8, i64, i64);

/// Mirrors Box.h: an ECS entity owns each spatial node; Partition indexes them.
/// Box member collections order ball and collidable identities by the original
/// C++ comparators, rather than by Bevy's Entity IDs.
#[derive(Component, Debug)]
#[allow(non_snake_case)] // Original C++ Box::mObject identity
pub struct Box {
    pub mKey: BoxKey,
    pub mLevel: usize,
    pub lo: DVec3,
    pub hi: DVec3,
    pub mParent: Option<Entity>,
    pub mChildren: BTreeMap<BoxKey, Entity>,
    pub balls: BTreeMap<BallSortKey, Entity>,
    pub mStaticCollidables: BTreeMap<(i64, i64), Entity>,
    pub mBubble: i32,
    pub mHandled: bool,
    /// Owns the C++ Box::mObject Python reference within the Box itself.
    pub mObject: Option<Py<PyAny>>,
}
impl Box {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mChildren.is_empty() && self.balls.is_empty() && self.mStaticCollidables.is_empty()
    }
    pub(crate) fn contains_collidables(&self) -> bool {
        !self.balls.is_empty() || !self.mStaticCollidables.is_empty()
    }
}


impl Box {
    /// C++ `Box::SetObject`: replaces (or clears) an owned Python reference.
    /// A missing or despawned Box is rejected; a Python None is NOT a null
    /// `PyObject*` (use `None` in Rust to clear the association).
    /// Original C++ Box::SetObject, operating on the Bevy-owned Box.
    #[allow(non_snake_case)]
    pub fn SetObject(world: &mut World, box_entity: Entity, object: Option<Py<PyAny>>) {
        if let Some(mut b) = world.get_mut::<Self>(box_entity) {
            b.mObject = object;
        }
    }

    /// Previous Rust API kept as a checked compatibility spelling.
    pub fn set_object(world: &mut World, box_entity: Entity, object: Option<Py<PyAny>>) -> bool {
        if world.get::<Self>(box_entity).is_none() {
            return false;
        }
        Self::SetObject(world, box_entity, object);
        true
    }

    #[must_use]
    pub fn object(world: &World, box_entity: Entity) -> Option<&Py<PyAny>> {
        world.get::<Self>(box_entity)?.mObject.as_ref()
    }

    /// Source-owned counterpart of C++ Box::AddChildren. An absent child
    /// is ignored; membership is keyed by the original Box::mKey comparer.
    #[allow(non_snake_case)]
    pub fn AddChildren(world: &mut World, box_entity: Entity, child: Option<Entity>) {
        let Some(child) = child else { return };
        let Some(key) = world.get::<Self>(child).map(|b| b.mKey) else { return };
        if let Some(mut b) = world.get_mut::<Self>(box_entity) {
            b.mChildren.entry(key).or_insert(child);
        }
    }

    /// C++ Box::Traverse, with the partition index retrieved from Bevy World.
    #[allow(non_snake_case)]
    pub fn Traverse(world: &mut World, entity: Entity, seed: i32) {
        if world.contains_resource::<Partition>() {
            world.resource_scope(|world, partition: Mut<'_, Partition>| {
                Self::traverse_with_partition(&partition, world, entity, seed);
            });
        }
    }

    /// Bevy's World separates ownership of Box and Partition, but the
    /// traversal algorithm itself belongs to Box as in original Box.cpp.
    pub(crate) fn traverse_with_partition(
        partition: &Partition, world: &mut World, start: Entity, seed: i32,
    ) {
        let mut todo = vec![start];
        while let Some(entity) = todo.pop() {
            let Some(b) = world.get::<Self>(entity) else { continue };
            if b.mBubble != -1 { continue; }
            let (key, level) = (b.mKey, b.mLevel as i32);
            world.get_mut::<Self>(entity).unwrap().mBubble = seed;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dy == 0 && dz == 0 { continue; }
                        if let (Some(x), Some(y), Some(z)) = (
                            key.ix.checked_add(dx), key.iy.checked_add(dy), key.iz.checked_add(dz),
                        ) {
                            if let Some(other) = partition.check_box(x, y, z, level) {
                                todo.push(other);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Original C++ Box::NearbyBubbles returns -1 if there is no marked
    /// neighbor; it is *not* a nullable pointer.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn NearbyBubbles(world: &World, entity: Entity) -> i32 {
        world.get_resource::<Partition>()
            .and_then(|p| Self::nearby_bubbles_with_partition(p, world, entity))
            .unwrap_or(-1)
    }

    pub(crate) fn nearby_bubbles_with_partition(
        partition: &Partition, world: &World, entity: Entity,
    ) -> Option<i32> {
        world.get::<Self>(entity)?;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 { continue; }
                    if let Some(neighbor) =
                        partition.check_offset_box(world, Some(entity), (x, y, z))
                    {
                        if let Some(other) = world.get::<Self>(neighbor) {
                            if other.mBubble != -1 {
                                return Some(other.mBubble);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// C++ Box::operator== compares the keys, not object identity.
    #[must_use]
    pub fn same_key(&self, other: &Self) -> bool {
        self.mKey == other.mKey
    }
}

/// Legacy Rust spelling retained temporarily for existing ECS consumers.
/// The canonical C++ correspondence is `Box`.
pub type PartitionBox = Box;
