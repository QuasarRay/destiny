// Copyright © 2014 CCP ehf. Original Box.cpp / Box.h.
// The ECS owns boxes as entities; Partition manages their index and lifetime.
use std::collections::BTreeMap;

use bevy::{math::DVec3, prelude::{Component, Entity, World}};
use crate::partitionable::BoxKey;
use pyo3::{prelude::*, types::PyAny};

type BallSortKey = (u8, i64, i64);

/// Mirrors Box.h: an ECS entity owns each spatial node; Partition indexes them.
/// Box member collections order ball and collidable identities by the original
/// C++ comparators, rather than by Bevy's Entity IDs.
#[derive(Component, Debug)]
#[allow(non_snake_case)] // Original C++ Box::mObject identity
pub struct Box {
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
    /// Owns the C++ Box::mObject Python reference within the Box itself.
    pub mObject: Option<Py<PyAny>>,
}
impl Box {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty() && self.balls.is_empty() && self.statics.is_empty()
    }
    pub(crate) fn contains_collidables(&self) -> bool {
        !self.balls.is_empty() || !self.statics.is_empty()
    }
}


impl Box {
    /// C++ `Box::SetObject`: replaces (or clears) an owned Python reference.
    /// A missing or despawned Box is rejected; a Python None is NOT a null
    /// `PyObject*` (use `None` in Rust to clear the association).
    pub fn set_object(world: &mut World, box_entity: Entity, object: Option<Py<PyAny>>) -> bool {
        let Some(mut box_ref) = world.get_mut::<Self>(box_entity) else {
            return false;
        };
        box_ref.mObject = object;
        true
    }

    #[must_use]
    pub fn object(world: &World, box_entity: Entity) -> Option<&Py<PyAny>> {
        world.get::<Self>(box_entity)?.mObject.as_ref()
    }

    /// C++ Box::operator== compares the keys, not object identity.
    #[must_use]
    pub fn same_key(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

/// Legacy Rust spelling retained temporarily for existing ECS consumers.
/// The canonical C++ correspondence is `Box`.
pub type PartitionBox = Box;
