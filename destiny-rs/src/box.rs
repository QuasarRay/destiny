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

    /// Original Box::SetParent: lazily obtain the coarser Box, set mParent,
    /// and register this Box in its parent's child collection. C++'s
    /// Partition* is supplied explicitly as the Bevy World resource.
    #[allow(non_snake_case)]
    pub fn SetParent(partition: &mut Partition, world: &mut World, child: Entity) {
        let Some((level, key)) =
            world.get::<Self>(child).map(|bx| (bx.mLevel, bx.mKey))
        else { return };
        if level == 0 { return; }
        let parent = partition.get_box(
            world, key.ix >> 2, key.iy >> 2, key.iz >> 2,
            (level - 1) as i32,
        );
        if let Some(parent) = parent {
            if let Some(mut bx) = world.get_mut::<Self>(child) {
                bx.mParent = Some(parent);
            }
            Self::AddChildren(world, parent, Some(child));
        }
    }

    /// Used by Partition::prune_empty when the child has already despawned.
    /// The original C++ destructor also disconnects its parent.
    pub(crate) fn unlink_child(
        world: &mut World, parent: Entity, child: Entity, key: BoxKey,
    ) -> bool {
        let Some(mut bx) = world.get_mut::<Self>(parent) else { return false };
        if bx.mChildren.get(&key) == Some(&child) {
            bx.mChildren.remove(&key);
            true
        } else {
            false
        }
    }

    /// Original Box::RemoveChildren: erase a child, then release the parent
    /// when it no longer holds any collidables or other children.
    #[allow(non_snake_case)]
    pub fn RemoveChildren(
        partition: &mut Partition, world: &mut World, parent: Entity, child: Entity,
    ) {
        let Some(key) = world.get::<Self>(child).map(|b| b.mKey) else { return };
        Self::unlink_child(world, parent, child, key);
        Self::ReleaseBox(partition, world, parent);
    }

    /// Original Box::AddBall: insert a Ball into the key-ordered Box set.
    /// Its reciprocal Partitionable::mBoxes link is maintained by the Ball
    /// adapter at the Bevy ECS boundary, not by this Box operation.
    #[allow(non_snake_case)]
    pub fn AddBall(world: &mut World, box_entity: Entity, ball: Option<Entity>) -> bool {
        let Some(ball) = ball else { return false };
        let Some(order) = crate::ball::BallSortComparer::key(world, ball) else { return false };
        let Some(mut bx) = world.get_mut::<Self>(box_entity) else { return false };
        use std::collections::btree_map::Entry;
        match bx.balls.entry(order) {
            Entry::Vacant(entry) => { entry.insert(ball); true }
            Entry::Occupied(_) => false,
        }
    }

    /// Original Box::AddStaticCollidable, with C++ comparator key
    /// (mParentBallId, mId) and no replacement on equivalent keys.
    #[allow(non_snake_case)]
    pub fn AddStaticCollidable(
        world: &mut World, box_entity: Entity, object: Option<Entity>,
    ) -> bool {
        let Some(object) = object else { return false };
        let Some(base) = world.get::<crate::partitionable::Partitionable>(object) else { return false };
        let Some(info) = world.get::<crate::static_collidable::StaticCollidable>(object) else { return false };
        let order = (info.mParentBallId, base.mId);
        let Some(mut bx) = world.get_mut::<Self>(box_entity) else { return false };
        use std::collections::btree_map::Entry;
        match bx.mStaticCollidables.entry(order) {
            Entry::Vacant(entry) => { entry.insert(object); true }
            Entry::Occupied(_) => false,
        }
    }

    /// Original Box::RemoveBall. Bevy tracks both sides of membership and
    /// reclaims an empty Box through Partition's world-local allocator.
    #[allow(non_snake_case)]
    pub fn RemoveBall(
        partition: &mut Partition, world: &mut World, box_entity: Entity, ball: Entity,
    ) {
        let Some(mut bx) = world.get_mut::<Self>(box_entity) else { return };
        let before = bx.balls.len();
        bx.balls.retain(|_, member| *member != ball);
        let removed = bx.balls.len() != before;
        let key = bx.mKey;
        if removed {
            if let Some(mut state) = world.get_mut::<crate::partitionable::Partitionable>(ball) {
                state.untrack_box(&key);
            }
        }
        Self::ReleaseBox(partition, world, box_entity);
    }

    /// Original Box::RemoveStaticCollidable, with the same reciprocal cleanup.
    #[allow(non_snake_case)]
    pub fn RemoveStaticCollidable(
        partition: &mut Partition, world: &mut World, box_entity: Entity, object: Entity,
    ) {
        let Some(mut bx) = world.get_mut::<Self>(box_entity) else { return };
        let before = bx.mStaticCollidables.len();
        bx.mStaticCollidables.retain(|_, member| *member != object);
        let removed = bx.mStaticCollidables.len() != before;
        let key = bx.mKey;
        if removed {
            if let Some(mut state) = world.get_mut::<crate::partitionable::Partitionable>(object) {
                state.untrack_box(&key);
            }
        }
        Self::ReleaseBox(partition, world, box_entity);
    }

    /// Original private Box::ReleaseBox, using Bevy's entity allocator and
    /// Partition's spatial index instead of the original C++ BoxAllocator.
    #[allow(non_snake_case)]
    pub(crate) fn ReleaseBox(
        partition: &mut Partition, world: &mut World, box_entity: Entity,
    ) -> bool {
        partition.release_box(world, box_entity)
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


/// Original BoxSortComparer, declared in SortedSets.h and defined in Box.cpp.
/// The order depends on Box::mKey, not the Bevy Entity identifier.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoxSortComparer;
impl BoxSortComparer {
    pub fn less(&self, world: &World, lhs: Entity, rhs: Entity) -> Option<bool> {
        let a = world.get::<Box>(lhs)?;
        let b = world.get::<Box>(rhs)?;
        Some(a.mKey < b.mKey)
    }
}

/// Legacy Rust spelling retained temporarily for existing ECS consumers.
/// The canonical C++ correspondence is `Box`.
pub type PartitionBox = Box;
