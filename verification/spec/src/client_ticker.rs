// Copyright © 2023 CCP ehf. Licensed under the original MIT license.
//! `_baseticker.py` and `_ticker.py` at the pinned behavioral revision.
//!
//! The catalog's SetState/Stop/GotoDirection inputs use four history groups,
//! four actions per group, eight immutable received streams and eleven saved
//! snapshots (including the source's append before trimming). Exhaustion is
//! an explicit scope error. This executes the original binary codec and real
//! old-orientation evolution; it does not substitute timestamps for evolution.
//! Python dispatch/aliases, Blue objects, logging, weakrefs and native callback
//! lifetime still require executable correspondence. Packaged/local actions
//! outside these ten scenarios are not part of this operation's input type.

use crate::{
    BoundedList, ClientHistory, ClientHistoryAction, ClientHistoryState, HistoryMergeError,
    OriginalParkError, OriginalParkProjection, OriginalStream, Vec3,
    merge_client_state_into_history,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientSnapshot {
    pub stream: OriginalStream,
    pub timestamp: i64,
    pub mid_tick: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientTickerError {
    Park(OriginalParkError),
    History(HistoryMergeError),
    CapacityExceeded,
    InvalidState,
}
impl From<OriginalParkError> for ClientTickerError {
    fn from(error: OriginalParkError) -> Self {
        Self::Park(error)
    }
}
impl From<HistoryMergeError> for ClientTickerError {
    fn from(error: HistoryMergeError) -> Self {
        Self::History(error)
    }
}

pub struct OriginalClientTicker {
    pub park: OriginalParkProjection,
    pub has_ballpark: bool,
    pub history: ClientHistory,
    pub latest_set_state_time: i64,
    pub should_rebase: bool,
    pub last_stamp: i64,
    pub states: BoundedList<ClientSnapshot, 11>,
    pub state_is_valid: bool,
    pub fatal_desync_occurred: bool,
    pub recoverable_desync_occurred: bool,
    /// The original test explicitly replaces the synchronization method with
    /// mock.Mock(return_value=False). None always executes the actual method.
    pub synchronization_mock_return: Option<bool>,
    streams: BoundedList<OriginalStream, 8>,
}
impl Default for OriginalClientTicker {
    fn default() -> Self {
        Self {
            park: OriginalParkProjection::default(),
            has_ballpark: false,
            history: ClientHistory::new(),
            latest_set_state_time: 0,
            should_rebase: false,
            last_stamp: -1,
            states: BoundedList::new(),
            state_is_valid: false,
            fatal_desync_occurred: false,
            recoverable_desync_occurred: false,
            synchronization_mock_return: None,
            streams: BoundedList::new(),
        }
    }
}
impl OriginalClientTicker {
    pub fn set_ballpark(&mut self, park: Option<OriginalParkProjection>) {
        self.has_ballpark = park.is_some();
        if let Some(park) = park {
            self.park = park;
        }
        // Both test interface set_ballpark methods retain the supplied park;
        // their abstract base implementations have no other observable work.
    }
    pub fn retain_received_stream(
        &mut self,
        stream: OriginalStream,
    ) -> Result<usize, ClientTickerError> {
        let index = self.streams.len();
        if !self.streams.push(stream) {
            return Err(ClientTickerError::CapacityExceeded);
        }
        Ok(index)
    }
    pub fn update(
        &mut self,
        state: &ClientHistoryState,
        wait_for_bubble: bool,
    ) -> Result<(), ClientTickerError> {
        if !self.has_ballpark {
            return Ok(());
        }
        // _expand_states appends these non-packaged actions unchanged.
        if let Some(first) = state.get(0) {
            let mut i = 1;
            while i < state.len() {
                let event = state.get(i).ok_or(ClientTickerError::InvalidState)?;
                if event.timestamp != first.timestamp {
                    self.fatal_desync_occurred = true;
                }
                i += 1;
            }
        }
        self.flush_state(state, wait_for_bubble)
    }
    fn flush_state(
        &mut self,
        state: &ClientHistoryState,
        wait_for_bubble: bool,
    ) -> Result<(), ClientTickerError> {
        let Some(first) = state.get(0) else {
            return Ok(());
        };
        if matches!(first.action, ClientHistoryAction::SetState { .. }) {
            self.latest_set_state_time = first.timestamp;
            let mut kept = ClientHistory::new();
            let mut i = 0;
            while i < self.history.len() {
                let entry = self.history.get(i).ok_or(ClientTickerError::InvalidState)?;
                let stamp = entry
                    .state
                    .get(0)
                    .ok_or(ClientTickerError::InvalidState)?
                    .timestamp;
                if stamp >= self.latest_set_state_time && !kept.push(entry) {
                    return Err(ClientTickerError::CapacityExceeded);
                }
                i += 1;
            }
            self.history = kept;
        } else if first.timestamp < self.latest_set_state_time {
            return Ok(());
        }
        merge_client_state_into_history(state, &mut self.history, wait_for_bubble)?;
        Ok(())
    }
    pub fn parent_evolve(&mut self) -> Result<(), ClientTickerError> {
        self.park.evolve_two_spherical_bodies()?;
        Ok(())
    }
    pub fn do_pre_tick(&mut self) -> Result<(), ClientTickerError> {
        while !self.history.is_empty() {
            let entry = self.history.get(0).ok_or(ClientTickerError::InvalidState)?;
            if entry.wait_for_bubble {
                return Ok(());
            }
            let stamp = entry
                .state
                .get(0)
                .ok_or(ClientTickerError::InvalidState)?
                .timestamp;
            let difference = stamp
                .checked_sub(self.park.current_time)
                .ok_or(ClientTickerError::InvalidState)?;
            if stamp > self.park.current_time && difference < 3 {
                break;
            }
            self.real_flush_state(&entry.state)?;
            let mut remaining = ClientHistory::new();
            let mut i = 1;
            while i < self.history.len() {
                let item = self.history.get(i).ok_or(ClientTickerError::InvalidState)?;
                if !remaining.push(item) {
                    return Err(ClientTickerError::CapacityExceeded);
                }
                i += 1;
            }
            self.history = remaining;
            if self.state_is_valid && self.should_rebase {
                self.store_state(true)?;
            }
            // Retain the source's >1 condition after deletion, including its
            // behavior when exactly one queued group remains.
            if self.history.len() > 1 {
                if self
                    .history
                    .get(0)
                    .ok_or(ClientTickerError::InvalidState)?
                    .wait_for_bubble
                {
                    return Ok(());
                }
                self.parent_evolve()?;
            }
        }
        Ok(())
    }
    fn set_state(&mut self, stream_index: usize, ego_id: i64) -> Result<(), ClientTickerError> {
        let stream = self
            .streams
            .get(stream_index)
            .ok_or(ClientTickerError::InvalidState)?;
        // TestClientTicker.on_set_state is a source-defined no-op.
        self.park.clear_all();
        self.park.read_original_state(&stream, 0)?;
        self.park.ego_id = ego_id;
        self.park.start();
        self.state_is_valid = true;
        self.flush_simulation_history(true)
    }
    fn real_flush_state(&mut self, state: &ClientHistoryState) -> Result<(), ClientTickerError> {
        let Some(first) = state.get(0) else {
            return Ok(());
        };
        if let ClientHistoryAction::SetState {
            stream_index,
            ego_id,
        } = first.action
        {
            self.set_state(stream_index, ego_id)?;
        }
        if !self.state_is_valid {
            return Ok(());
        }
        self.should_rebase = false;
        let mut synchronised = false;
        let mut i = 0;
        while i < state.len() {
            let event = state.get(i).ok_or(ClientTickerError::InvalidState)?;
            if !matches!(event.action, ClientHistoryAction::SetState { .. }) {
                if !synchronised {
                    synchronised = if let Some(value) = self.synchronization_mock_return {
                        value
                    } else {
                        self.synchronize_to_simulation_time(event.timestamp)?
                    };
                }
                if !synchronised {
                    self.recoverable_desync_occurred = true;
                    return Ok(());
                }
                self.should_rebase = true;
                match event.action {
                    ClientHistoryAction::Stop { id } => self.park.stop(id),
                    ClientHistoryAction::GotoDirection { id, direction } => {
                        self.park.goto_direction(
                            id,
                            Vec3 {
                                x: direction[0],
                                y: direction[1],
                                z: direction[2],
                            },
                        )
                    }
                    ClientHistoryAction::SetState { .. } => {
                        return Err(ClientTickerError::InvalidState);
                    }
                }
                // TestClientTicker.on_ballpark_local_action is a no-op. Native
                // Python exception/dispatch correspondence is not inferred.
                if self.park.scope_error {
                    return Err(ClientTickerError::Park(
                        OriginalParkError::OutsideDeclaredScope,
                    ));
                }
            }
            i += 1;
        }
        Ok(())
    }
    pub fn do_post_tick(&mut self, stamp: i64) -> Result<(), ClientTickerError> {
        if self.should_rebase {
            self.flush_simulation_history(true)?;
            self.should_rebase = false;
        } else if stamp
            > self
                .last_stamp
                .checked_add(10)
                .ok_or(ClientTickerError::InvalidState)?
        {
            self.store_state(false)?;
        }
        Ok(())
    }
    pub fn clear_states(&mut self) {
        self.states = BoundedList::new();
    }
    pub fn store_state(&mut self, mid_tick: bool) -> Result<(), ClientTickerError> {
        if !self.park.is_running {
            return Ok(());
        }
        let stream = self.park.write_original_full_state(None)?;
        if !self.states.push(ClientSnapshot {
            stream,
            timestamp: self.park.current_time,
            mid_tick,
        }) {
            return Err(ClientTickerError::CapacityExceeded);
        }
        if self.states.len() > 10 {
            let mut trimmed = BoundedList::new();
            if !trimmed.push(self.states.get(0).ok_or(ClientTickerError::InvalidState)?) {
                return Err(ClientTickerError::CapacityExceeded);
            }
            // self._states[3:3] is empty in the original. Do not repair it.
            let mut i = self.states.len() - 5;
            while i < self.states.len() {
                if !trimmed.push(self.states.get(i).ok_or(ClientTickerError::InvalidState)?) {
                    return Err(ClientTickerError::CapacityExceeded);
                }
                i += 1;
            }
            self.states = trimmed;
        }
        self.last_stamp = self.park.current_time;
        Ok(())
    }
    pub fn synchronize_to_simulation_time(
        &mut self,
        stamp: i64,
    ) -> Result<bool, ClientTickerError> {
        let last_stamp;
        if stamp < self.park.current_time {
            let mut selected = None;
            let mut i = 0;
            while i < self.states.len() {
                let item = self.states.get(i).ok_or(ClientTickerError::InvalidState)?;
                if item.timestamp <= stamp {
                    selected = Some(item);
                }
                i += 1;
            }
            let Some(item) = selected else {
                return Ok(false);
            };
            self.park.read_original_state(&item.stream, 1)?;
            last_stamp = item.timestamp;
        } else {
            last_stamp = self.park.current_time;
        }
        let steps = stamp
            .checked_sub(last_stamp)
            .ok_or(ClientTickerError::InvalidState)?;
        let mut i = 0;
        while i < steps {
            self.parent_evolve()?;
            i += 1;
        }
        Ok(true)
    }
    pub fn flush_simulation_history(
        &mut self,
        new_base_snapshot: bool,
    ) -> Result<(), ClientTickerError> {
        let mut last_mid = None;
        if new_base_snapshot && !self.states.is_empty() {
            let last = self
                .states
                .get(self.states.len() - 1)
                .ok_or(ClientTickerError::InvalidState)?;
            let previous = self
                .park
                .current_time
                .checked_sub(1)
                .ok_or(ClientTickerError::InvalidState)?;
            if last.mid_tick && last.timestamp == previous {
                last_mid = Some(last);
            }
        }
        self.clear_states();
        if new_base_snapshot {
            if let Some(last) = last_mid {
                if !self.states.push(last) {
                    return Err(ClientTickerError::CapacityExceeded);
                }
            }
            self.store_state(false)?;
        }
        Ok(())
    }
}
