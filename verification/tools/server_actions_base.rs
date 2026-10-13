// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
// Source: python/destiny/net/server/_actions.py at
// ef738406b61ad06bae2f402857e464735ab106a2.
// Concrete Python action-history semantics, with eight queued actions, eight
// arguments per action and four ball projections. The catalog scenarios using
// an empty park fit these bounds. Native ball construction, Blue streams,
// weak-reference signal delivery and Python module loading remain separate.

use crate::BoundedList;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl ActionValue {
    #[must_use]
    pub fn truthy(self) -> bool {
        match self {
            Self::Integer(value) => value != 0,
            Self::Float(value) => value != 0.0,
            Self::Boolean(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkAction {
    pub name: ActionName,
    pub parameters: BoundedList<ActionValue, 8>,
    pub local_argument: Option<ActionValue>,
    pub stream:Option<crate::OriginalStream>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkActionRecord {
    pub ball_id: ActionValue,
    pub timestamp: i64,
    pub action: NetworkAction,
}

pub type NetworkActionHistory = BoundedList<NetworkActionRecord, 8>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkBallProjection {
    pub id: ActionValue,
    pub old_bubble_id: i64,
    pub new_bubble_id: i64,
    pub cloak_mode: i64,
    pub is_interactive: bool,
    pub speed_fraction: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkActionError {
    CapacityExceeded,
    DynamicalOrientationDisabled,
}

pub struct NetworkActions {
    pub dynamical_orientation: bool,
    pub balls: BoundedList<NetworkBallProjection, 4>,
    system_history: NetworkActionHistory,
    stamp_for_system: i64,
}

impl Default for NetworkActions {
    fn default() -> Self {
        Self::new_for_empty_park_at_time_zero()
    }
}

impl NetworkActions {
    pub fn add_stream_to_system_history(&mut self,id:i64,stream:crate::OriginalStream)->Result<(),NetworkActionError> {
        let record=NetworkActionRecord{ball_id:ActionValue::Integer(id),timestamp:self.stamp_for_system,
            action:NetworkAction{name:ActionName::AddBallsToPark,parameters:BoundedList::new(),local_argument:None,stream:Some(stream)}};
        if self.system_history.get(self.system_history.len().saturating_sub(1))!=Some(record)
            && !self.system_history.push(record) { return Err(NetworkActionError::CapacityExceeded); }Ok(())
    }
    pub fn add_ball_not_global(&mut self,id:i64,bubble:i64)->Result<(),NetworkActionError> {
        self.add_to_system_history(ActionValue::Integer(id),ActionName::BallNotGlobal,&[ActionValue::Integer(bubble)],false,None)
    }
    /// TestActions.setUp creates an empty master park at currentTime=0. This
    /// constructor models that observed state, not arbitrary Python parks.
    #[must_use]
    pub const fn new_for_empty_park_at_time_zero() -> Self {
        Self {
            dynamical_orientation: false,
            balls: BoundedList::new(),
            system_history: NetworkActionHistory::new(),
            stamp_for_system: 0,
        }
    }

    pub fn set_stamp_for_system(&mut self, stamp: i64) {
        self.stamp_for_system = stamp;
    }

    pub fn reset_settings_to_default(&mut self) {
        self.dynamical_orientation = false;
    }

    pub fn clear_park(&mut self) {
        self.balls = BoundedList::new();
    }

    pub fn flush_history(&mut self) -> NetworkActionHistory {
        let result = self.system_history;
        self.system_history = NetworkActionHistory::new();
        result
    }

    fn add_to_system_history(
        &mut self,
        ball_id: ActionValue,
        name: ActionName,
        arguments: &[ActionValue],
        insert_ball_id: bool,
        local_argument: Option<ActionValue>,
    ) -> Result<(), NetworkActionError> {
        let mut parameters = BoundedList::new();
        if insert_ball_id && !parameters.push(ball_id) {
            return Err(NetworkActionError::CapacityExceeded);
        }
        let mut i: usize = 0;
        while i < arguments.len() {
            if let Some(value) = arguments.get(i) {
                if !parameters.push(*value) {
                    return Err(NetworkActionError::CapacityExceeded);
                }
            }
            i = i.saturating_add(1);
        }
        let record = NetworkActionRecord {
            ball_id,
            timestamp: self.stamp_for_system,
            action: NetworkAction { name, parameters, local_argument, stream:None },
        };
        if self.system_history.is_empty()
            || self.system_history.get(self.system_history.len().saturating_sub(1)) != Some(record)
        {
            if !self.system_history.push(record) {
                return Err(NetworkActionError::CapacityExceeded);
            }
        }
        // The fixture's on_add_to_system_history Signal has no connected slots.
        // No callback delivery or weakref theorem is claimed by this model.
        Ok(())
    }

    fn ball(&self, src_id: ActionValue) -> Option<NetworkBallProjection> {
        let mut i: usize = 0;
        while i < self.balls.len() {
            if let Some(ball) = self.balls.get(i) {
                if ball.id == src_id {
                    return Some(ball);
                }
            }
            i = i.saturating_add(1);
        }
        None
    }

    fn last_pending_action(&self, src_id: ActionValue, name: ActionName) -> Option<NetworkAction> {
        let mut result = None;
        let mut i: usize = 0;
        while i < self.system_history.len() {
            if let Some(record) = self.system_history.get(i) {
                if record.ball_id == src_id && record.action.name == name {
                    result = Some(record.action);
                }
            }
            i = i.saturating_add(1);
        }
        result
    }

    pub fn cloak_ball(
        &mut self,
        src_id: ActionValue,
        cloak_mode: ActionValue,
        uncloak_range_meters: ActionValue,
    ) -> Result<bool, NetworkActionError> {
        let Some(ball) = self.ball(src_id) else { return Ok(false); };
        let bubble_id = if ball.new_bubble_id != ball.old_bubble_id {
            ball.old_bubble_id
        } else {
            ball.new_bubble_id
        };
        self.add_to_system_history(src_id, ActionName::CloakBall,
            &[cloak_mode, uncloak_range_meters], true, Some(ActionValue::Integer(bubble_id)))?;
        Ok(true)
    }

    pub fn undo_pending_cloak(&mut self, src_id: ActionValue) {
        let mut retained = NetworkActionHistory::new();
        let mut i: usize = 0;
        while i < self.system_history.len() {
            if let Some(record) = self.system_history.get(i) {
                if record.ball_id != src_id || record.action.name != ActionName::CloakBall {
                    // The filtered list cannot exceed its input capacity.
                    assert!(retained.push(record));
                }
            }
            i = i.saturating_add(1);
        }
        self.system_history = retained;
    }

    #[must_use]
    pub fn has_pending_cloak(&self, src_id: ActionValue) -> bool {
        self.last_pending_action(src_id, ActionName::CloakBall).is_some()
    }

    #[must_use]
    pub fn has_pending_uncloak(&self, src_id: ActionValue) -> bool {
        self.last_pending_action(src_id, ActionName::UncloakBall).is_some()
    }

    #[must_use]
    pub fn get_pending_cloak_mode(&self, src_id: ActionValue) -> Option<ActionValue> {
        if let Some(action) = self.last_pending_action(src_id, ActionName::CloakBall) {
            return action.parameters.get(1);
        }
        self.ball(src_id).map(|ball| ActionValue::Integer(ball.cloak_mode))
    }

    #[must_use]
    pub fn has_pending_set_free(&self, src_id: ActionValue) -> ActionValue {
        if let Some(action) = self.last_pending_action(src_id, ActionName::SetBallFree) {
            ActionValue::Boolean(action.parameters.get(1) == Some(ActionValue::Boolean(true)))
        } else {
            // Python's `len(events) and ...` returns integer zero, not False.
            ActionValue::Integer(0)
        }
    }

    #[must_use]
    pub fn has_pending_set_not_free(&self, src_id: ActionValue) -> ActionValue {
        if let Some(action) = self.last_pending_action(src_id, ActionName::SetBallFree) {
            ActionValue::Boolean(action.parameters.get(1) == Some(ActionValue::Boolean(false)))
        } else {
            ActionValue::Integer(0)
        }
    }

    #[must_use]
    pub fn get_pending_interactive_state(&self, src_id: ActionValue) -> Option<ActionValue> {
        if let Some(action) = self.last_pending_action(src_id, ActionName::SetBallInteractive) {
            return action.parameters.get(1).map(|value| ActionValue::Boolean(value.truthy()));
        }
        self.ball(src_id).map(|ball| ActionValue::Boolean(ball.is_interactive))
    }

    #[must_use]
    pub fn get_pending_speed_fraction(&self, src_id: ActionValue) -> Option<ActionValue> {
        if let Some(action) = self.last_pending_action(src_id, ActionName::SetSpeedFraction) {
            return action.parameters.get(1);
        }
        self.ball(src_id).map(|ball| ActionValue::Float(ball.speed_fraction))
    }
}

// The generator appends only source-reviewed straight history constructors,
// orientation guards and the forcefield's ordered massive/harmonic operations.
