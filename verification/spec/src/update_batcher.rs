// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
//! ParkUpdateBatcher history operations at the behavioral reference pin.
//!
//! Dictionary insertion order, front/back insertion, copied getters, deletion
//! and signal emission are retained. Four dictionary keys and eight events per
//! key bound the original examples. Signals have no connected slots in these
//! fixtures. Python weak-reference/module/native lifetime correspondence and
//! send_batch/streams/bubble updates are separate obligations.

use crate::{ActionName, ActionValue, BoundedList};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatcherPayload {
    Scalar(ActionValue),
    Tuple(BoundedList<ActionValue, 8>),
    Stream {
        stream_index: usize,
        reference: Option<i64>,
    },
    Packaged {
        package_index: usize,
    },
    BallIds(BoundedList<i64, 8>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatcherEvent {
    pub timestamp: i64,
    pub name: ActionName,
    pub payload: BatcherPayload,
}

pub type BatcherHistory = BoundedList<BatcherEvent, 8>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatcherHistoryEntry {
    pub key: i64,
    pub history: BatcherHistory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatcherSignal {
    Character(ActionName),
    Bubble(ActionName),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatcherScopeError {
    CapacityExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalNetworkCast {
    pub clients: BoundedList<i64, 8>,
    pub history: BatcherHistory,
    pub wait_for_bubble: bool,
    pub casts_in_batch: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastDispatch {
    Singlecasts,
    SingleBatchNarrowcasts,
    DualBatchNarrowcasts,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OriginalNetworkOutput {
    pub singlecasts: BoundedList<OriginalNetworkCast, 8>,
    pub narrowcasts: BoundedList<OriginalNetworkCast, 8>,
    pub dispatch_order: BoundedList<CastDispatch, 3>,
    pub timestamp_mismatches: BoundedList<BatcherEvent, 32>,
}

fn has_id(ids: &BoundedList<i64, 8>, id: i64) -> bool {
    let mut i = 0;
    while i < ids.len() {
        if ids.get(i) == Some(id) {
            return true;
        }
        i += 1;
    }
    false
}
fn add_id(ids: &mut BoundedList<i64, 8>, id: i64) -> Result<(), BatcherScopeError> {
    if !has_id(ids, id) && !ids.push(id) {
        return Err(BatcherScopeError::CapacityExceeded);
    }
    Ok(())
}

pub struct OriginalUpdateBatcher {
    character_history: BoundedList<BatcherHistoryEntry, 4>,
    bubble_history: BoundedList<BatcherHistoryEntry, 4>,
    pub signals: BoundedList<BatcherSignal, 32>,
    // Python histories retain references to immutable byte objects. This
    // append-only eight-stream arena preserves their values and lifetime for
    // these scenarios without copying 4096 bytes into every event/cast slot.
    streams: BoundedList<crate::OriginalStream, 8>,
    packages: BoundedList<crate::OriginalStream, 8>,
}

impl Default for OriginalUpdateBatcher {
    fn default() -> Self {
        Self {
            character_history: BoundedList::new(),
            bubble_history: BoundedList::new(),
            signals: BoundedList::new(),
            streams: BoundedList::new(),
            packages: BoundedList::new(),
        }
    }
}

impl OriginalUpdateBatcher {
    pub fn retain_event_stream(
        &mut self,
        stream: crate::OriginalStream,
    ) -> Result<usize, BatcherScopeError> {
        let index = self.streams.len();
        if !self.streams.push(stream) {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        Ok(index)
    }
    pub fn send_full_state_update(
        &mut self,
        park: &crate::OriginalParkProjection,
        character: i64,
        source: i64,
    ) -> Result<(), BatcherScopeError> {
        let bytes = park
            .write_original_full_state(Some(source))
            .map_err(|_| BatcherScopeError::CapacityExceeded)?;
        let stream_index = self.streams.len();
        if !self.streams.push(bytes) {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        self.add_to_character_history(
            character,
            BatcherEvent {
                timestamp: park.current_time,
                name: ActionName::SetState,
                payload: BatcherPayload::Stream {
                    stream_index,
                    reference: Some(source),
                },
            },
            true,
        )
    }
    #[must_use]
    pub fn stream_bytes(&self, index: usize) -> Option<crate::OriginalStream> {
        self.streams.get(index)
    }
    #[must_use]
    pub fn package_bytes(&self, index: usize) -> Option<crate::OriginalStream> {
        self.packages.get(index)
    }

    fn package_event(
        &mut self,
        park: &crate::OriginalParkProjection,
        ids: BoundedList<i64, 8>,
        additions: bool,
    ) -> Result<BatcherEvent, BatcherScopeError> {
        let action = if additions {
            crate::OriginalMarshalledAction::AddBalls {
                stream: park
                    .write_original_balls(&ids)
                    .map_err(|_| BatcherScopeError::CapacityExceeded)?,
            }
        } else {
            crate::OriginalMarshalledAction::RemoveBalls { ids }
        };
        let bytes = crate::marshal_original_event(crate::OriginalMarshalledEvent {
            timestamp: park.current_time,
            root_is_list: additions,
            action_count: 1,
            parameter_count: 1,
            action,
        })
        .map_err(|_| BatcherScopeError::CapacityExceeded)?;
        let package_index = self.packages.len();
        if !self.packages.push(bytes) {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        Ok(BatcherEvent {
            timestamp: park.current_time,
            name: ActionName::PackagedAction,
            payload: BatcherPayload::Packaged { package_index },
        })
    }

    pub fn update_bubbles(
        &mut self,
        park: &mut crate::OriginalParkProjection,
        updater: &mut crate::OriginalBubbleUpdater,
        interests: &crate::OriginalNetworkInterests,
    ) -> Result<(), BatcherScopeError> {
        updater
            .additions_and_deletions(park)
            .map_err(|_| BatcherScopeError::CapacityExceeded)?;
        let updates = updater.updates;
        let mut deletions = BoundedList::<(i64, Option<BatcherEvent>), 8>::new();
        let mut additions = BoundedList::<(i64, BatcherEvent), 8>::new();
        let mut i = 0;
        // PreBuildDeletions mutates nonempty list values into packaged events;
        // empty lists remain false. Preserve this before building additions.
        while i < updates.deletions_per_bubble.len() {
            let Some((bubble, delta)) = updates.deletions_per_bubble.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let crate::OriginalBubbleDelta::Balls(ids) = delta else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let event = if ids.is_empty() {
                None
            } else {
                Some(self.package_event(park, ids, false)?)
            };
            if !deletions.push((bubble, event)) {
                return Err(BatcherScopeError::CapacityExceeded);
            }
            i += 1;
        }
        i = 0;
        while i < updates.additions_per_bubble.len() {
            let Some((bubble, delta)) = updates.additions_per_bubble.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let crate::OriginalBubbleDelta::Balls(ids) = delta else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            if !ids.is_empty() {
                let event = self.package_event(park, ids, true)?;
                if !additions.push((bubble, event)) {
                    return Err(BatcherScopeError::CapacityExceeded);
                }
            }
            i += 1;
        }
        i = 0;
        while i < updates.deletions_per_player.len() {
            let Some((ball, delta)) = updates.deletions_per_player.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let event = match delta {
                crate::OriginalBubbleDelta::Bubble(bubble) => {
                    let mut found = None;
                    let mut exists = false;
                    let mut j = 0;
                    while j < deletions.len() {
                        if let Some((key, value)) = deletions.get(j) {
                            if key == bubble {
                                exists = true;
                                found = value;
                                break;
                            }
                        }
                        j += 1;
                    }
                    if !exists {
                        return Err(BatcherScopeError::CapacityExceeded);
                    }
                    found
                }
                crate::OriginalBubbleDelta::Balls(ids) => {
                    if ids.is_empty() {
                        None
                    } else {
                        Some(BatcherEvent {
                            timestamp: park.current_time,
                            name: ActionName::RemoveBalls,
                            payload: BatcherPayload::BallIds(ids),
                        })
                    }
                }
            };
            if let Some(event) = event {
                let characters = interests
                    .characters_for_ball(ball)
                    .map_err(|_| BatcherScopeError::CapacityExceeded)?;
                let mut j = 0;
                while j < characters.len() {
                    let Some(character) = characters.get(j) else {
                        return Err(BatcherScopeError::CapacityExceeded);
                    };
                    self.add_to_character_history(character, event, false)?;
                    j += 1;
                }
            }
            i += 1;
        }
        i = 0;
        while i < updates.additions_per_player.len() {
            let Some((ball, delta)) = updates.additions_per_player.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let characters = interests
                .characters_for_ball(ball)
                .map_err(|_| BatcherScopeError::CapacityExceeded)?;
            let event = match delta {
                crate::OriginalBubbleDelta::Bubble(bubble) => {
                    let mut found = None;
                    let mut j = 0;
                    while j < additions.len() {
                        if let Some((key, value)) = additions.get(j) {
                            if key == bubble {
                                found = Some(value);
                                break;
                            }
                        }
                        j += 1;
                    }
                    found
                }
                crate::OriginalBubbleDelta::Balls(ids) => {
                    if ids.is_empty() || characters.is_empty() {
                        None
                    } else {
                        Some(self.package_event(park, ids, true)?)
                    }
                }
            };
            if let Some(event) = event {
                let mut j = 0;
                while j < characters.len() {
                    let Some(character) = characters.get(j) else {
                        return Err(BatcherScopeError::CapacityExceeded);
                    };
                    self.add_to_character_history(character, event, false)?;
                    j += 1;
                }
            }
            i += 1;
        }
        Ok(())
    }
    pub fn get_clients_with_character_history(
        &self,
        interests: &crate::OriginalNetworkInterests,
    ) -> Result<BoundedList<i64, 8>, BatcherScopeError> {
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < self.character_history.len() {
            if let Some(entry) = self.character_history.get(i) {
                if let Some(client) = interests.client_for_character(entry.key) {
                    if !entry.history.is_empty() {
                        add_id(&mut result, client)?;
                    }
                }
            }
            i += 1;
        }
        Ok(result)
    }

    fn check_timestamps(
        history: BatcherHistory,
        time: i64,
        output: &mut OriginalNetworkOutput,
    ) -> Result<(), BatcherScopeError> {
        let mut i = 0;
        while i < history.len() {
            if let Some(event) = history.get(i) {
                if event.timestamp != time
                    && event.timestamp > 0
                    && !output.timestamp_mismatches.push(event)
                {
                    return Err(BatcherScopeError::CapacityExceeded);
                }
            }
            i += 1;
        }
        Ok(())
    }

    pub fn send_batch(
        &mut self,
        park: &crate::OriginalParkProjection,
        interests: &crate::OriginalNetworkInterests,
    ) -> Result<OriginalNetworkOutput, BatcherScopeError> {
        let mut output = OriginalNetworkOutput::default();
        let characters = self.get_clients_with_character_history(interests)?;
        let interactives = park.bubble_interactive_entries();
        let mut waiting = BoundedList::new();
        let mut single = BoundedList::new();
        let mut dual = BoundedList::new();
        let mut i = 0;
        while i < self.bubble_history.len() {
            let Some(entry) = self.bubble_history.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            let mut clients = BoundedList::new();
            let mut exists = false;
            let mut j = 0;
            while j < interactives.len() {
                if let Some((bubble, ball)) = interactives.get(j) {
                    if bubble == entry.key {
                        exists = true;
                        let recipients = interests
                            .clients_for_ball(ball)
                            .map_err(|_| BatcherScopeError::CapacityExceeded)?;
                        let mut k = 0;
                        while k < recipients.len() {
                            let Some(client) = recipients.get(k) else {
                                return Err(BatcherScopeError::CapacityExceeded);
                            };
                            add_id(&mut clients, client)?;
                            k += 1;
                        }
                    }
                }
                j += 1;
            }
            if exists {
                Self::check_timestamps(entry.history, park.current_time, &mut output)?;
                let mut one = BoundedList::new();
                let mut two = BoundedList::new();
                j = 0;
                while j < clients.len() {
                    let Some(client) = clients.get(j) else {
                        return Err(BatcherScopeError::CapacityExceeded);
                    };
                    add_id(
                        if has_id(&characters, client) {
                            &mut two
                        } else {
                            &mut one
                        },
                        client,
                    )?;
                    add_id(&mut waiting, client)?;
                    j += 1;
                }
                if !one.is_empty()
                    && !single.push(OriginalNetworkCast {
                        clients: one,
                        history: entry.history,
                        wait_for_bubble: false,
                        casts_in_batch: 1,
                    })
                {
                    return Err(BatcherScopeError::CapacityExceeded);
                }
                if !two.is_empty()
                    && !dual.push(OriginalNetworkCast {
                        clients: two,
                        history: entry.history,
                        wait_for_bubble: false,
                        casts_in_batch: 2,
                    })
                {
                    return Err(BatcherScopeError::CapacityExceeded);
                }
            }
            i += 1;
        }
        self.bubble_history = BoundedList::new();
        i = 0;
        while i < self.character_history.len() {
            let Some(entry) = self.character_history.get(i) else {
                return Err(BatcherScopeError::CapacityExceeded);
            };
            if let Some(client) = interests.client_for_character(entry.key) {
                if !entry.history.is_empty() {
                    let wait = has_id(&waiting, client);
                    Self::check_timestamps(entry.history, park.current_time, &mut output)?;
                    let mut clients = BoundedList::new();
                    add_id(&mut clients, client)?;
                    if !output.singlecasts.push(OriginalNetworkCast {
                        clients,
                        history: entry.history,
                        wait_for_bubble: wait,
                        casts_in_batch: if wait { 2 } else { 1 },
                    }) {
                        return Err(BatcherScopeError::CapacityExceeded);
                    }
                }
            }
            i += 1;
        }
        self.character_history = BoundedList::new();
        output.narrowcasts = single;
        if !output.narrowcasts.extend(&dual)
            || !output.dispatch_order.push(CastDispatch::Singlecasts)
            || !output
                .dispatch_order
                .push(CastDispatch::SingleBatchNarrowcasts)
            || !output
                .dispatch_order
                .push(CastDispatch::DualBatchNarrowcasts)
        {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        Ok(output)
    }
    fn find(entries: &BoundedList<BatcherHistoryEntry, 4>, key: i64) -> Option<usize> {
        let mut i = 0_usize;
        while i < entries.len() {
            if let Some(entry) = entries.get(i) {
                if entry.key == key {
                    return Some(i);
                }
            }
            i = i.saturating_add(1);
        }
        None
    }

    fn add(
        entries: &mut BoundedList<BatcherHistoryEntry, 4>,
        key: i64,
        event: BatcherEvent,
        in_front: bool,
    ) -> Result<(), BatcherScopeError> {
        if let Some(index) = Self::find(entries, key) {
            if let Some(mut entry) = entries.get(index) {
                let inserted = if in_front {
                    entry.history.insert(0, event)
                } else {
                    entry.history.push(event)
                };
                if !inserted || !entries.set(index, entry) {
                    return Err(BatcherScopeError::CapacityExceeded);
                }
            }
        } else {
            let mut history = BatcherHistory::new();
            if !history.push(event) || !entries.push(BatcherHistoryEntry { key, history }) {
                return Err(BatcherScopeError::CapacityExceeded);
            }
        }
        Ok(())
    }

    fn get(entries: &BoundedList<BatcherHistoryEntry, 4>, key: i64) -> BatcherHistory {
        if let Some(index) = Self::find(entries, key) {
            if let Some(entry) = entries.get(index) {
                return entry.history;
            }
        }
        BatcherHistory::new()
    }

    pub fn add_to_character_history(
        &mut self,
        key: i64,
        event: BatcherEvent,
        in_front: bool,
    ) -> Result<(), BatcherScopeError> {
        Self::add(&mut self.character_history, key, event, in_front)?;
        if !self.signals.push(BatcherSignal::Character(event.name)) {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        Ok(())
    }

    pub fn add_to_bubble_history(
        &mut self,
        key: i64,
        event: BatcherEvent,
    ) -> Result<(), BatcherScopeError> {
        Self::add(&mut self.bubble_history, key, event, false)?;
        if !self.signals.push(BatcherSignal::Bubble(event.name)) {
            return Err(BatcherScopeError::CapacityExceeded);
        }
        Ok(())
    }

    pub fn clear_character_history(&mut self, key: i64) {
        // Rebuilding the retained entries preserves Python dictionary order.
        let mut retained = BoundedList::new();
        let mut i = 0_usize;
        while i < self.character_history.len() {
            if let Some(entry) = self.character_history.get(i) {
                if entry.key != key {
                    assert!(retained.push(entry));
                }
            }
            i = i.saturating_add(1);
        }
        self.character_history = retained;
    }

    #[must_use]
    pub fn get_character_history(&self, key: i64) -> BatcherHistory {
        Self::get(&self.character_history, key)
    }

    #[must_use]
    pub fn get_bubble_history(&self, key: i64) -> BatcherHistory {
        Self::get(&self.bubble_history, key)
    }

    #[must_use]
    pub fn character_has_history(&self, key: i64) -> bool {
        !Self::get(&self.character_history, key).is_empty()
    }

    #[must_use]
    pub fn bubble_has_history(&self, key: i64) -> bool {
        !Self::get(&self.bubble_history, key).is_empty()
    }
}
