// Copyright © 2015 CCP ehf. (original C++ Partitionable)
// Rust/Bevy port: shared state and abstract behavior contract.

use std::collections::BTreeMap;

use bevy::{
    math::DVec3,
    prelude::{Component, Entity, World},
};

/// Destiny's original `ID` typedef is a signed 64-bit integer.
pub type DestinyId = i64;

/// Canonical C++ Key declaration lives with Partition, as in Partition.h.
/// `BoxKey` is a temporary compatibility alias.
pub use crate::partition::Key;
pub type BoxKey = Key;

/// Native Bevy component corresponding to the shared C++ `Partitionable` state.
///
/// Its positions use Bevy's double-precision `DVec3`; the C++ `Box*` and
/// `Ballpark*` become optional world-local ECS entity references. This does
/// not introduce a second object graph.
///
/// Note that entities do not automatically remove reciprocal box links on
/// despawn. The concrete Ball/StaticCollidable providers are responsible for
/// maintaining both sides of membership when their ports are implemented.
#[derive(Component, Debug, Clone)]
#[allow(non_snake_case)] // Canonical C++ Partitionable field names
pub struct Partitionable {
    pub mId: DestinyId,
    pub mHandled: bool,
    pub mNewPos: DVec3,
    pub mOldPos: DVec3,
    /// MSVC `long` (C++ Destiny's target ABI) is signed 32-bit.
    pub mNewBubble: i32,
    pub mOldBubble: i32,
    pub isInteractive: bool,
    /// Spatial key -> Bevy entity. Deterministic key order, not entity order.
    mBoxes: BTreeMap<BoxKey, Entity>,
    pub mMyBox: Option<Entity>,
    /// Reserved association with the owning Ballpark entity once ported.
    pub mPark: Option<Entity>,
    pub mEffectStamp: i32,
    pub isMoribund: bool,
}

impl Default for Partitionable {
    fn default() -> Self {
        Self {
            mId: 0,
            mHandled: false,
            mNewPos: DVec3::ZERO,
            mOldPos: DVec3::ZERO,
            mNewBubble: -1,
            mOldBubble: -1,
            isInteractive: false,
            mBoxes: BTreeMap::new(),
            mMyBox: None,
            mPark: None,
            mEffectStamp: 0,
            isMoribund: false,
        }
    }
}

impl Partitionable {
    /// Iterator ordered exactly as the C++ `BoxSortComparer` orders keys.
    pub fn boxes(&self) -> impl Iterator<Item = (BoxKey, Entity)> + '_ {
        self.mBoxes.iter().map(|(key, entity)| (*key, *entity))
    }

    /// Record an already-established box relationship on this side only.
    ///
    /// Equivalent spatial keys are equivalent set entries in C++:
    /// repeated insertion does not replace the original entity.
    /// The concrete behavior provider must update the Box's reciprocal set.
    pub fn track_box(&mut self, key: BoxKey, box_entity: Entity) -> bool {
        use std::collections::btree_map::Entry;
        match self.mBoxes.entry(key) {
            Entry::Vacant(v) => {
                v.insert(box_entity);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// Remove this side of an already-removed box relationship.
    pub fn untrack_box(&mut self, key: &BoxKey) -> Option<Entity> {
        self.mBoxes.remove(key)
    }

    /// Common *local-only* part of C++ `ClearBoxes()`.
    ///
    /// Unlike `DeleteFromBoxes()`, this does not update the Box's own
    /// membership set. Ball::ClearBoxes additionally clears Ball-specific
    /// active-box caches; those are not part of Partitionable.
    pub fn forget_box_references(&mut self) {
        self.mBoxes.clear();
        self.mMyBox = None;
    }
}

/// Explicit contract corresponding to Partitionable's seven pure virtual methods.
///
/// Implement this for the concrete Ball and StaticCollidable partition policies,
/// not for `Partitionable` itself: the original abstract class has NO
/// implementations of these methods. Passing Bevy's World and object Entity
/// makes reciprocal box/ballpark updates possible without unsafe C++ pointers.
/// `Option<Entity>` preserves nullable C++ Box pointers.
pub trait PartitionableBehavior {
    fn clear_boxes(world: &mut World, object: Entity);
    fn delete_from_boxes(world: &mut World, object: Entity);
    fn insert_in_box(world: &mut World, object: Entity, box_entity: Option<Entity>);
    fn get_bounding_radius(world: &World, object: Entity) -> f32;
    fn get_inflated_radius(world: &World, object: Entity, dt: f64) -> f64;
    fn set_box_active(world: &mut World, object: Entity, box_id: i32, active: bool);
    fn insert_in_boxes(
        world: &mut World,
        object: Entity,
        first: Option<Entity>,
        top: Option<Entity>,
        new_bubble_id: i32,
    );
}
