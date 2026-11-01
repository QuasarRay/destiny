// Copyright © 2015 CCP ehf. (original C++ Partitionable)
// Rust/Bevy port: shared state and abstract behavior contract.

use std::collections::BTreeMap;

use bevy::{
    math::DVec3,
    prelude::{Component, Entity, World},
};

/// Exact original `typedef int64_t ID;` in Partitionable.h.
pub type ID = i64;

/// Pre-existing Rust compatibility alias; the canonical name is `ID`.
pub type DestinyId = ID;

/// Canonical C++ Key declaration lives with Partition, as in Partition.h.
/// `BoxKey` is a temporary compatibility alias.
pub use crate::partition::Key;
pub type BoxKey = Key;

/// Canonical C++ `SortedSetOfBoxes` (SortedSets.h). This represents the
/// same ordered set membership using key-indexed Bevy entity identities.
/// The original std::set<Box*, BoxSortComparer> cannot be used directly
/// without C++ pointers; BTreeMap uses the corresponding Box::mKey order.
pub type SortedSetOfBoxes = BTreeMap<BoxKey, Entity>;

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
    pub mBoxes: SortedSetOfBoxes,
    pub mMyBox: Option<Entity>,
    /// Reserved association with the owning Ballpark entity once ported.
    pub mPark: Option<Entity>,
    pub mEffectStamp: i32,
    pub isMoribund: bool,
}

impl Default for Partitionable {
    fn default() -> Self {
        Self::new()
    }
}

impl Partitionable {
    /// Original C++ `Partitionable::Partitionable()` constructor, including
    /// inherited Vector3d zero construction and empty SortedSetOfBoxes.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mId: 0,
            mHandled: false,
            mNewPos: DVec3::ZERO,
            mOldPos: DVec3::ZERO,
            mNewBubble: -1,
            mOldBubble: -1,
            isInteractive: false,
            mBoxes: SortedSetOfBoxes::new(),
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

/// The seven C++ *pure virtual* methods have no bodies in Partitionable.cpp.
/// Rust traits are the language-required abstract dispatch boundary. These
/// exact names preserve the original abstract method identities and owners.
/// Each concrete derived implementation must satisfy all seven operations.
///
/// The explicit World + Entity parameters replace the original implicit C++
/// `this` and nullable pointers. Unlike the original class, Rust cannot
/// attach abstract virtual methods directly to a concrete ECS component.
#[allow(non_snake_case)]
pub trait PartitionableBehavior {
    /// `virtual void ClearBoxes() = 0;`
    fn ClearBoxes(world: &mut World, object: Entity);
    /// `virtual void DeleteFromBoxes() = 0;`
    fn DeleteFromBoxes(world: &mut World, object: Entity);
    /// `virtual void InsertInBox(Box* box) = 0;`
    fn InsertInBox(world: &mut World, object: Entity, box_entity: Option<Entity>);
    /// `virtual float GetBoundingRadius() = 0;`
    fn GetBoundingRadius(world: &World, object: Entity) -> f32;
    /// `virtual double GetInflatedRadius(double dt) = 0;`
    fn GetInflatedRadius(world: &World, object: Entity, dt: f64) -> f64;
    /// `virtual void SetBoxActive(int boxId, bool isActive) = 0;`
    fn SetBoxActive(world: &mut World, object: Entity, box_id: i32, isActive: bool);
    /// `virtual void InsertInBoxes(Box*, Box*, long) = 0;`
    fn InsertInBoxes(
        world: &mut World, object: Entity, box1: Option<Entity>,
        top: Option<Entity>, newBubbleId: i32,
    );
}

// Pure-virtual method entry points keep their original owner, Partitionable,
// while Rust's trait implements the virtual dispatch policy. No fake Ball
// methods are provided while the concrete Ball class remains incomplete.
// T supplies the dynamic concrete policy that is normally selected by a C++
// vtable; these methods do not implement any original pure-virtual bodies.
#[allow(non_snake_case)]
impl Partitionable {
    pub fn ClearBoxes<T: PartitionableBehavior>(world: &mut World, object: Entity) {
        T::ClearBoxes(world, object);
    }
    pub fn DeleteFromBoxes<T: PartitionableBehavior>(world: &mut World, object: Entity) {
        T::DeleteFromBoxes(world, object);
    }
    pub fn InsertInBox<T: PartitionableBehavior>(
        world: &mut World, object: Entity, box_entity: Option<Entity>,
    ) {
        T::InsertInBox(world, object, box_entity);
    }
    #[must_use]
    pub fn GetBoundingRadius<T: PartitionableBehavior>(world: &World, object: Entity) -> f32 {
        T::GetBoundingRadius(world, object)
    }
    #[must_use]
    pub fn GetInflatedRadius<T: PartitionableBehavior>(
        world: &World, object: Entity, dt: f64,
    ) -> f64 {
        T::GetInflatedRadius(world, object, dt)
    }
    pub fn SetBoxActive<T: PartitionableBehavior>(
        world: &mut World, object: Entity, box_id: i32, isActive: bool,
    ) {
        T::SetBoxActive(world, object, box_id, isActive);
    }
    pub fn InsertInBoxes<T: PartitionableBehavior>(
        world: &mut World, object: Entity, box1: Option<Entity>,
        top: Option<Entity>, newBubbleId: i32,
    ) {
        T::InsertInBoxes(world, object, box1, top, newBubbleId);
    }
}
