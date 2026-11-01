// Copyright © 2014 CCP ehf. Original Box.cpp / Box.h.
// The ECS owns boxes as entities; Partition manages their index and lifetime.
use std::collections::BTreeMap;

use bevy::{math::DVec3, prelude::{Component, Entity, World}};
use destiny_partitionable::BoxKey;
use pyo3::{prelude::*, types::PyAny};

type BallSortKey = (u8, i64, i64);

/// Mirrors Box.h: an ECS entity owns each spatial node; Partition indexes them.
/// Box member collections order ball and collidable identities by the original
/// C++ comparators, rather than by Bevy's Entity IDs.
#[derive(Component, Debug, Clone)]
pub struct PartitionBox {
    pub key: BoxKey,
    pub level: usize,
    pub lo: DVec3,
    pub hi: DVec3,
    pub parent: Option<Entity>,
    pub children: BTreeMap<BoxKey, Entity>,
    pub balls: BTreeMap<BallSortKey, Entity>,
    pub statics: BTreeMap<(i64, i64), Entity>,
    pub bubble: i32,
    pub handled: bool,
}
impl PartitionBox {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty() && self.balls.is_empty() && self.statics.is_empty()
    }
    pub(crate) fn contains_collidables(&self) -> bool {
        !self.balls.is_empty() || !self.statics.is_empty()
    }
}


/// The optional C++ `Box::mObject` Python reference.
/// 
/// A separate Bevy component models a nullable Python pointer, and PyO3
/// manages INCREF/DECREF on assignment/removal/despawn. This component is
/// deliberately distinct from spatial keys and physics state.
#[derive(Component, Debug)]
pub struct BoxObject(pub Py<PyAny>);

impl PartitionBox {
    /// C++ `Box::SetObject`: replaces (or clears) an owned Python reference.
    /// A missing or despawned Box is rejected; a Python None is NOT a null
    /// `PyObject*` (use `None` in Rust to clear the association).
    pub fn set_object(world: &mut World, box_entity: Entity, object: Option<Py<PyAny>>) -> bool {
        if world.get::<Self>(box_entity).is_none() {
            return false;
        }
        match object {
            Some(value) => {
                world.entity_mut(box_entity).insert(BoxObject(value));
            }
            None => {
                world.entity_mut(box_entity).remove::<BoxObject>();
            }
        }
        true
    }

    #[must_use]
    pub fn object(world: &World, box_entity: Entity) -> Option<&Py<PyAny>> {
        world.get::<BoxObject>(box_entity).map(|value| &value.0)
    }

    /// C++ Box::operator== compares the keys, not object identity.
    #[must_use]
    pub fn same_key(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
