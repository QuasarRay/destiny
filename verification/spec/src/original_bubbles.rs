// Copyright © 2015-2023 CCP ehf. MIT license, retained from original Destiny.
//! The dictionaries and lists in AddToBubble, CopyBubbles and
//! PyAdditionsAndDeletions at the behavioral reference revision.
//!
//! Dictionary equality is independent of key order; every value that is a
//! list preserves insertion order. Interactive membership is a separate
//! registry, not inferred from a ball flag at query time. Capacity failures
//! are explicit. Python allocation, callback dispatch and native pointer
//! correspondence remain separate obligations.

use crate::{BoundedList, OriginalMode, OriginalParkBall, OriginalParkError};

pub type OriginalBubbleIds = BoundedList<i64, 8>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OriginalBubbleDelta {
    Bubble(i64),
    Balls(OriginalBubbleIds),
}

pub type OriginalBubbleDeltaMap = BoundedList<(i64, OriginalBubbleDelta), 8>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalBubbleUpdates {
    pub additions_per_player: OriginalBubbleDeltaMap,
    pub deletions_per_player: OriginalBubbleDeltaMap,
    pub additions_per_bubble: OriginalBubbleDeltaMap,
    pub deletions_per_bubble: OriginalBubbleDeltaMap,
    pub transitions: BoundedList<(i64, i64, i64), 8>,
}

impl Default for OriginalBubbleUpdates {
    fn default() -> Self {
        Self {
            additions_per_player: BoundedList::new(),
            deletions_per_player: BoundedList::new(),
            additions_per_bubble: BoundedList::new(),
            deletions_per_bubble: BoundedList::new(),
            transitions: BoundedList::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalBubbleEntry {
    pub bubble: i64,
    pub ball: i64,
    pub action: i8,
}

pub(crate) struct OriginalBubbleDictionary {
    // Empty inner dictionaries remain until CopyBubbles removes them.
    pub groups: BoundedList<i64, 16>,
    pub entries: BoundedList<OriginalBubbleEntry, 16>,
    pub interactives: BoundedList<(i64, i64), 16>,
}

impl Default for OriginalBubbleDictionary {
    fn default() -> Self {
        Self {
            groups: BoundedList::new(),
            entries: BoundedList::new(),
            interactives: BoundedList::new(),
        }
    }
}

fn contains<const N: usize>(list: &BoundedList<i64, N>, key: i64) -> bool {
    let mut i = 0;
    while i < list.len() {
        if list.get(i) == Some(key) {
            return true;
        }
        i += 1;
    }
    false
}

fn ball(balls: &BoundedList<OriginalParkBall, 8>, id: i64) -> Option<OriginalParkBall> {
    let mut i = 0;
    while i < balls.len() {
        if let Some(b) = balls.get(i) {
            if b.id == id {
                return Some(b);
            }
        }
        i += 1;
    }
    None
}

fn push<T: Copy, const N: usize>(
    list: &mut BoundedList<T, N>,
    value: T,
) -> Result<(), OriginalParkError> {
    if list.push(value) {
        Ok(())
    } else {
        Err(OriginalParkError::OutsideDeclaredScope)
    }
}

impl OriginalBubbleDictionary {
    fn entry(&self, bubble: i64, ball: i64) -> Option<usize> {
        let mut i = 0;
        while i < self.entries.len() {
            if self
                .entries
                .get(i)
                .is_some_and(|e| e.bubble == bubble && e.ball == ball)
            {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn set_action(&mut self, index: usize, action: i8) -> Result<(), OriginalParkError> {
        let Some(mut e) = self.entries.get(index) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        e.action = action;
        if self.entries.set(index, e) {
            Ok(())
        } else {
            Err(OriginalParkError::OutsideDeclaredScope)
        }
    }

    fn remove_entry(&mut self, index: usize) -> Result<(), OriginalParkError> {
        let mut kept = BoundedList::new();
        let mut i = 0;
        while i < self.entries.len() {
            if i != index {
                if let Some(e) = self.entries.get(i) {
                    push(&mut kept, e)?;
                }
            }
            i += 1;
        }
        self.entries = kept;
        Ok(())
    }

    pub fn transition(&mut self, b: OriginalParkBall) -> Result<(), OriginalParkError> {
        if b.new_bubble != -1 {
            if !contains(&self.groups, b.new_bubble) {
                push(&mut self.groups, b.new_bubble)?;
            }
            if let Some(index) = self.entry(b.new_bubble, b.id) {
                if self.entries.get(index).is_some_and(|e| e.action == -1) {
                    self.set_action(index, 0)?;
                }
            } else {
                push(
                    &mut self.entries,
                    OriginalBubbleEntry {
                        bubble: b.new_bubble,
                        ball: b.id,
                        action: 1,
                    },
                )?;
            }
        }
        if b.old_bubble != -1 {
            if let Some(index) = self.entry(b.old_bubble, b.id) {
                match self.entries.get(index).map(|e| e.action) {
                    Some(0) => self.set_action(index, -1)?,
                    Some(1) => self.remove_entry(index)?,
                    _ => (),
                }
            }
        }
        // UpdateInteractiveCnt calls Increase before Decrease, including the
        // original equal-bubble case. This is separate from raw flag writes.
        if b.is_interactive {
            if b.new_bubble != -1 {
                let mut exists = false;
                let mut i = 0;
                while i < self.interactives.len() {
                    exists |= self.interactives.get(i) == Some((b.new_bubble, b.id));
                    i += 1;
                }
                if !exists {
                    push(&mut self.interactives, (b.new_bubble, b.id))?;
                }
            }
            let mut retained = BoundedList::new();
            let mut i = 0;
            while i < self.interactives.len() {
                if let Some(entry) = self.interactives.get(i) {
                    if entry != (b.old_bubble, b.id) {
                        push(&mut retained, entry)?;
                    }
                }
                i += 1;
            }
            self.interactives = retained;
        }
        Ok(())
    }

    pub fn set_interactive_membership(
        &mut self,
        id: i64,
        bubble: i64,
        enabled: bool,
    ) -> Result<(), OriginalParkError> {
        if bubble == -1 {
            return Ok(());
        }
        let mut entries = BoundedList::new();
        let mut exists = false;
        let mut i = 0;
        while i < self.interactives.len() {
            let entry = self
                .interactives
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            if entry == (bubble, id) {
                exists = true;
                if enabled {
                    push(&mut entries, entry)?;
                }
            } else {
                push(&mut entries, entry)?;
            }
            i += 1;
        }
        if enabled && !exists {
            push(&mut entries, (bubble, id))?;
        }
        self.interactives = entries;
        Ok(())
    }

    pub fn copy_bubbles(&mut self) -> Result<(), OriginalParkError> {
        let mut kept = BoundedList::new();
        let mut i = 0;
        while i < self.entries.len() {
            if let Some(mut entry) = self.entries.get(i) {
                if entry.action == 1 {
                    entry.action = 0;
                }
                if entry.action != -1 {
                    push(&mut kept, entry)?;
                }
            }
            i += 1;
        }
        self.entries = kept;
        let mut groups = BoundedList::new();
        i = 0;
        while i < self.groups.len() {
            if let Some(group) = self.groups.get(i) {
                let mut nonempty = false;
                let mut j = 0;
                while j < self.entries.len() {
                    nonempty |= self.entries.get(j).is_some_and(|e| e.bubble == group);
                    j += 1;
                }
                if nonempty {
                    push(&mut groups, group)?;
                }
            }
            i += 1;
        }
        self.groups = groups;
        Ok(())
    }

    pub fn user_balls(&self) -> Result<OriginalBubbleIds, OriginalParkError> {
        // PySet iteration order is a residual Python representation obligation.
        // The concrete BubbleUpdater fixtures compare dictionaries of user
        // results, not the order of this list. Each per-bubble list still uses
        // the entries' dictionary insertion order.
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < self.interactives.len() {
            if let Some((_, id)) = self.interactives.get(i) {
                push(&mut result, id)?;
            }
            i += 1;
        }
        Ok(result)
    }

    fn populate(
        &self,
        bubble: i64,
        balls: &BoundedList<OriginalParkBall, 8>,
    ) -> Result<(OriginalBubbleIds, OriginalBubbleIds), OriginalParkError> {
        let mut adds = BoundedList::new();
        let mut dels = BoundedList::new();
        let mut i = 0;
        while i < self.entries.len() {
            if let Some(entry) = self.entries.get(i) {
                if entry.bubble == bubble && entry.ball >= -(1_i64 << 30) {
                    let b = ball(balls, entry.ball);
                    let eligible = if entry.action != -1 {
                        b.is_some_and(|b| !b.is_global && b.is_cloaked == 0)
                    } else {
                        !b.is_some_and(|b| b.is_cloaked != 0)
                    };
                    if eligible {
                        if entry.action == 1 {
                            push(&mut adds, entry.ball)?;
                        } else if entry.action == -1 {
                            push(&mut dels, entry.ball)?;
                        }
                    }
                }
            }
            i += 1;
        }
        Ok((adds, dels))
    }

    pub fn updates(
        &self,
        balls: &BoundedList<OriginalParkBall, 8>,
        users: &BoundedList<Option<i64>, 8>,
    ) -> Result<OriginalBubbleUpdates, OriginalParkError> {
        // The concrete catalog supplies distinct users. Repeated-user input
        // needs sequential mutation/overwrite semantics beyond this scope.
        let mut seen = BoundedList::<i64, 8>::new();
        let mut user_index = 0;
        while user_index < users.len() {
            if let Some(Some(id)) = users.get(user_index) {
                if contains(&seen, id) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                push(&mut seen, id)?;
            }
            user_index += 1;
        }
        let mut result = OriginalBubbleUpdates::default();
        let mut i = 0;
        while i < self.groups.len() {
            if let Some(group) = self.groups.get(i) {
                let mut interactive = false;
                let mut j = 0;
                while j < self.interactives.len() {
                    interactive |= self
                        .interactives
                        .get(j)
                        .is_some_and(|(bubble, _)| bubble == group);
                    j += 1;
                }
                if interactive {
                    let (adds, dels) = self.populate(group, balls)?;
                    push(
                        &mut result.additions_per_bubble,
                        (group, OriginalBubbleDelta::Balls(adds)),
                    )?;
                    push(
                        &mut result.deletions_per_bubble,
                        (group, OriginalBubbleDelta::Balls(dels)),
                    )?;
                }
            }
            i += 1;
        }
        i = 0;
        while i < users.len() {
            let Some(Some(id)) = users.get(i) else {
                return Err(OriginalParkError::TypeError);
            };
            let Some(user) = ball(balls, id) else {
                i += 1;
                continue;
            };
            let new_exists = contains(&self.groups, user.new_bubble);
            let old_exists = contains(&self.groups, user.old_bubble);
            if user.new_bubble == user.old_bubble && user.new_bubble != -1 && new_exists {
                let mut precalculated = false;
                let mut j = 0;
                while j < result.additions_per_bubble.len() {
                    precalculated |= result
                        .additions_per_bubble
                        .get(j)
                        .is_some_and(|(key, _)| key == user.new_bubble);
                    j += 1;
                }
                let (adds, dels) = if precalculated {
                    (
                        OriginalBubbleDelta::Bubble(user.new_bubble),
                        OriginalBubbleDelta::Bubble(user.new_bubble),
                    )
                } else {
                    let (adds, dels) = self.populate(user.new_bubble, balls)?;
                    (
                        OriginalBubbleDelta::Balls(adds),
                        OriginalBubbleDelta::Balls(dels),
                    )
                };
                push(&mut result.additions_per_player, (id, adds))?;
                push(&mut result.deletions_per_player, (id, dels))?;
            } else {
                let mut adds = BoundedList::new();
                let mut dels = BoundedList::new();
                let mut j = 0;
                while j < self.entries.len() {
                    if let Some(entry) = self.entries.get(j) {
                        if entry.bubble == user.new_bubble && entry.ball >= -(1_i64 << 30) {
                            let b = ball(balls, entry.ball);
                            if !b.is_some_and(|b| b.is_global || b.is_cloaked != 0) {
                                let was_in =
                                    old_exists && self.entry(user.old_bubble, entry.ball).is_some();
                                if entry.action >= 0 && !was_in {
                                    push(&mut adds, entry.ball)?;
                                } else if entry.action == -1 && (was_in || !old_exists) {
                                    push(&mut dels, entry.ball)?;
                                }
                            }
                        }
                    }
                    j += 1;
                }
                j = 0;
                while j < self.entries.len() {
                    if let Some(entry) = self.entries.get(j) {
                        if entry.bubble == user.old_bubble && entry.ball >= -(1_i64 << 30) {
                            let b = ball(balls, entry.ball);
                            let eligible = if entry.action != -1 {
                                b.is_some_and(|b| !b.is_global && b.is_cloaked == 0)
                            } else {
                                !b.is_some_and(|b| b.is_cloaked != 0)
                            };
                            let is_in =
                                new_exists && self.entry(user.new_bubble, entry.ball).is_some();
                            if eligible && !is_in && entry.action <= 0 {
                                push(&mut dels, entry.ball)?;
                            }
                        }
                    }
                    j += 1;
                }
                push(
                    &mut result.additions_per_player,
                    (id, OriginalBubbleDelta::Balls(adds)),
                )?;
                push(
                    &mut result.deletions_per_player,
                    (id, OriginalBubbleDelta::Balls(dels)),
                )?;
                if user.mode != OriginalMode::Troll {
                    push(
                        &mut result.transitions,
                        (id, user.old_bubble, user.new_bubble),
                    )?;
                }
            }
            i += 1;
        }
        Ok(result)
    }
}

/// Python BubbleUpdater's per-currentTime cache. Its park ownership/weakrefs
/// and destruction remain executable boundary obligations.
pub struct OriginalBubbleUpdater {
    pub tick: Option<i64>,
    pub user_balls: OriginalBubbleIds,
    pub updates: OriginalBubbleUpdates,
}

impl Default for OriginalBubbleUpdater {
    fn default() -> Self {
        Self {
            tick: None,
            user_balls: BoundedList::new(),
            updates: OriginalBubbleUpdates::default(),
        }
    }
}

impl OriginalBubbleUpdater {
    pub fn additions_and_deletions(
        &mut self,
        park: &mut crate::OriginalParkProjection,
    ) -> Result<(), OriginalParkError> {
        if self.tick != Some(park.current_time) {
            self.user_balls = park.bubble_user_balls()?;
            let mut users = BoundedList::new();
            let mut i = 0;
            while i < self.user_balls.len() {
                if let Some(id) = self.user_balls.get(i) {
                    push(&mut users, Some(id))?;
                }
                i += 1;
            }
            self.updates = park.bubble_additions_and_deletions(&users)?;
            park.copy_query_bubbles()?;
            self.tick = Some(park.current_time);
        }
        Ok(())
    }
}
