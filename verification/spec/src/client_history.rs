// Copyright © 2007, 2023 CCP ehf.
// Licensed under the original repository's MIT license.
// Translated from python/destiny/net/client/_util.py at
// ef738406b61ad06bae2f402857e464735ab106a2.

use crate::BoundedList;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientHistoryAction {
    SetState { stream_index:usize,ego_id:i64 },
    GotoDirection { id: i64, direction: [f64; 3] },
    Stop { id: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientHistoryEvent {
    pub timestamp: i64,
    pub action: ClientHistoryAction,
}

pub type ClientHistoryState = BoundedList<ClientHistoryEvent, 4>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientHistoryBatch {
    pub state: ClientHistoryState,
    pub wait_for_bubble: bool,
}

pub type ClientHistory = BoundedList<ClientHistoryBatch, 4>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMergeError {
    CapacityExceeded,
    // The original reads time_list[0] when the incoming list is empty but the
    // old history is nonempty. Preserve that IndexError instead of correcting it.
    EmptyIncomingWithExistingHistory,
    EmptyHistoryBatch,
}

/// The original grouping, sorted-time traversal and mutation order. The eleven
/// catalog examples use at most two batches with two actions per batch. This
/// model has four slots for each; capacity failure is explicit. It does not
/// model Python list aliasing or interpreter/module-loading behavior.
pub fn merge_client_state_into_history(
    state: &ClientHistoryState,
    history: &mut ClientHistory,
    mut wait_for_bubble: bool,
) -> Result<(), HistoryMergeError> {
    let mut groups = ClientHistory::new();
    let mut incoming_index = 0;
    while incoming_index < state.len() {
        if let Some(event) = state.get(incoming_index) {
            let mut group_index = 0;
            let mut found = false;
            while group_index < groups.len() {
                if let Some(mut group) = groups.get(group_index) {
                    if let Some(first) = group.state.get(0) {
                        if first.timestamp == event.timestamp {
                            if !group.state.push(event) || !groups.set(group_index, group) {
                                return Err(HistoryMergeError::CapacityExceeded);
                            }
                            found = true;
                            break;
                        }
                        if first.timestamp > event.timestamp {
                            break;
                        }
                    }
                }
                group_index = group_index.saturating_add(1);
            }
            if !found {
                let mut group_state = ClientHistoryState::new();
                if !group_state.push(event)
                    || !groups.insert(
                        group_index,
                        ClientHistoryBatch {
                            state: group_state,
                            wait_for_bubble: false,
                        },
                    )
                {
                    return Err(HistoryMergeError::CapacityExceeded);
                }
            }
        }
        incoming_index = incoming_index.saturating_add(1);
    }

    let mut time_list_idx = 0;
    let mut history_idx = 0;
    // enumerate(history_list) observes the list after insertion, just as Python
    // does. In particular do not replace this with a newly sorted history.
    while history_idx < history.len() {
        let Some(mut history_entries) = history.get(history_idx) else {
            return Err(HistoryMergeError::EmptyHistoryBatch);
        };
        let Some(first_history_entry) = history_entries.state.get(0) else {
            return Err(HistoryMergeError::EmptyHistoryBatch);
        };
        let Some(mut incoming_entries) = groups.get(time_list_idx) else {
            return Err(HistoryMergeError::EmptyIncomingWithExistingHistory);
        };
        let Some(first_incoming_entry) = incoming_entries.state.get(0) else {
            return Err(HistoryMergeError::EmptyHistoryBatch);
        };
        let history_time = first_history_entry.timestamp;
        let entry_time = first_incoming_entry.timestamp;
        if history_time < entry_time {
            wait_for_bubble = false;
            history_idx = history_idx.saturating_add(1);
            continue;
        }
        if history_time == entry_time {
            if !history_entries.state.extend(&incoming_entries.state) {
                return Err(HistoryMergeError::CapacityExceeded);
            }
            history_entries.wait_for_bubble = false;
            if !history.set(history_idx, history_entries) {
                return Err(HistoryMergeError::CapacityExceeded);
            }
        } else {
            incoming_entries.wait_for_bubble = wait_for_bubble;
            if !history.insert(history_idx, incoming_entries) {
                return Err(HistoryMergeError::CapacityExceeded);
            }
            wait_for_bubble = false;
        }
        time_list_idx = time_list_idx.saturating_add(1);
        if time_list_idx >= groups.len() {
            break;
        }
        history_idx = history_idx.saturating_add(1);
    }
    while time_list_idx < groups.len() {
        if let Some(mut group) = groups.get(time_list_idx) {
            group.wait_for_bubble = wait_for_bubble;
            if !history.push(group) {
                return Err(HistoryMergeError::CapacityExceeded);
            }
        }
        time_list_idx = time_list_idx.saturating_add(1);
    }
    Ok(())
}
