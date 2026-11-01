// Original BallSortComparer from CCP Destiny SortedSets.h / Ball.cpp.
// The complete Ball physics object and its lifecycle are not ported.
// This file implements only the original comparator against the already
// existing, *temporary* PartitionBall metadata at the Bevy World boundary.
use bevy::prelude::{Entity, World};
use crate::{partition::PartitionBall, partitionable::Partitionable};

#[derive(Debug, Clone, Copy, Default)]
pub struct BallSortComparer;

impl BallSortComparer {
    /// Transitive key encoding the C++ comparator:
    /// two negative IDs use (owner ID, ID); otherwise compare ball IDs.
    /// Since regular ball IDs are nonnegative, all negative IDs sort first.
    #[must_use]
    pub fn key(world: &World, ball: Entity) -> Option<(u8, i64, i64)> {
        let p = world.get::<Partitionable>(ball)?;
        let info = world.get::<PartitionBall>(ball)?;
        if p.mId < 0 {
            Some((0, info.owner_id, p.mId))
        } else {
            Some((1, 0, p.mId))
        }
    }

    /// Direct C++ BallSortComparer::operator() branch correspondence.
    #[must_use]
    pub fn less(&self, world: &World, lhs: Entity, rhs: Entity) -> Option<bool> {
        let x = world.get::<Partitionable>(lhs)?;
        let y = world.get::<Partitionable>(rhs)?;
        if x.mId < 0 && y.mId < 0 {
            let a = world.get::<PartitionBall>(lhs)?.owner_id;
            let b = world.get::<PartitionBall>(rhs)?.owner_id;
            if a < b { return Some(true); }
            if a > b { return Some(false); }
        }
        Some(x.mId < y.mId)
    }
}
