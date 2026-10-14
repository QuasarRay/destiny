// Copyright © 2023 CCP ehf. Original Destiny MIT license retained.
//! `_baseticker.py`/`_ticker.py`'s actual server action ordering and delivery.
//!
//! This uses the original action queue, park operations, bubble updater, binary
//! stream codec and update batcher. Catalog inputs use three bodies and the
//! listed actions. Unsupported action/type/capacity is an explicit scope error,
//! never an assumption or a successful omission. Native parent-method lookup,
//! logging/timers, Blue/Python object identity and weak-reference signals remain
//! executable correspondence requirements.

use crate::{
    ActionName, ActionValue, BatcherEvent, BatcherPayload, BoundedList, NetworkAction,
    NetworkActionHistory, NetworkBallProjection, OriginalBubbleUpdater, OriginalNetworkInterests,
    OriginalNetworkOutput, OriginalParkError, OriginalParkProjection, OriginalServerParkActions,
    OriginalUpdateBatcher, Vec3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerTickerSignal {
    Cloaking(i64),
    Uncloaking(i64),
    SetPosition(i64, bool),
}

pub struct OriginalServerTicker {
    pub actions: OriginalServerParkActions,
    pub batcher: OriginalUpdateBatcher,
    pub updater: OriginalBubbleUpdater,
    pub interests: OriginalNetworkInterests,
    pub network: OriginalNetworkOutput,
    pub stamp_for_system: i64,
    pub current_system_history: NetworkActionHistory,
    /// Source Signal slots are connected to mock.Mock only in the selected
    /// cases. Preserve their complete ordered invocation arguments and counts.
    pub signal_calls: BoundedList<ServerTickerSignal, 8>,
    pub cloaking_connected: bool,
    pub uncloaking_connected: bool,
    pub position_connected: bool,
}
impl Default for OriginalServerTicker {
    fn default() -> Self {
        Self {
            actions: OriginalServerParkActions::default(),
            batcher: OriginalUpdateBatcher::default(),
            updater: OriginalBubbleUpdater::default(),
            interests: OriginalNetworkInterests::default(),
            network: OriginalNetworkOutput::default(),
            stamp_for_system: 1,
            current_system_history: NetworkActionHistory::new(),
            signal_calls: BoundedList::new(),
            cloaking_connected: false,
            uncloaking_connected: false,
            position_connected: false,
        }
    }
}
fn integer(value: ActionValue) -> Result<i64, OriginalParkError> {
    match value {
        ActionValue::Integer(i) => Ok(i),
        _ => Err(OriginalParkError::OutsideDeclaredScope),
    }
}
fn number(value: ActionValue) -> f64 {
    match value {
        ActionValue::Float(f) => f,
        ActionValue::Integer(i) => i as f64,
        ActionValue::Boolean(b) => f64::from(u8::from(b)),
    }
}
fn parameter(action: &NetworkAction, index: usize) -> Result<ActionValue, OriginalParkError> {
    action
        .parameters
        .get(index)
        .ok_or(OriginalParkError::OutsideDeclaredScope)
}
fn id_parameter(action: &NetworkAction, index: usize) -> Result<i64, OriginalParkError> {
    integer(parameter(action, index)?)
}
fn point_parameters(action: &NetworkAction) -> Result<Vec3, OriginalParkError> {
    Ok(Vec3 {
        x: number(parameter(action, 1)?),
        y: number(parameter(action, 2)?),
        z: number(parameter(action, 3)?),
    })
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

impl OriginalServerTicker {
    pub fn refresh_action_balls(
        &mut self,
        park: &OriginalParkProjection,
    ) -> Result<(), OriginalParkError> {
        self.actions.queue.balls = BoundedList::new();
        let balls = park.live_ball_sequence()?;
        let mut i = 0;
        while i < balls.len() {
            let ball = balls
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            push(
                &mut self.actions.queue.balls,
                NetworkBallProjection {
                    id: ActionValue::Integer(ball.id),
                    old_bubble_id: ball.old_bubble,
                    new_bubble_id: ball.new_bubble,
                    cloak_mode: i64::from(ball.is_cloaked),
                    is_interactive: ball.is_interactive,
                    speed_fraction: f64::from(ball.speed_fraction),
                },
            )?;
            i += 1;
        }
        self.actions.queue.dynamical_orientation = park.dynamical_orientation;
        Ok(())
    }
    fn event(
        &mut self,
        stamp: i64,
        action: NetworkAction,
    ) -> Result<BatcherEvent, OriginalParkError> {
        let payload = if let Some(stream) = action.stream {
            let index = self
                .batcher
                .retain_event_stream(stream)
                .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
            BatcherPayload::Stream {
                stream_index: index,
                reference: None,
            }
        } else {
            BatcherPayload::Tuple(action.parameters)
        };
        Ok(BatcherEvent {
            timestamp: stamp,
            name: action.name,
            payload,
        })
    }
    fn bubble_event(&mut self, bubble: i64, event: BatcherEvent) -> Result<(), OriginalParkError> {
        self.batcher
            .add_to_bubble_history(bubble, event)
            .map_err(|_| OriginalParkError::OutsideDeclaredScope)
    }
    fn character_event(
        &mut self,
        character: i64,
        event: BatcherEvent,
        in_front: bool,
    ) -> Result<(), OriginalParkError> {
        self.batcher
            .add_to_character_history(character, event, in_front)
            .map_err(|_| OriginalParkError::OutsideDeclaredScope)
    }
    fn bubble_ids(park: &OriginalParkProjection) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        let mut ids = BoundedList::new();
        let entries = park.bubble_interactive_entries();
        let mut i = 0;
        while i < entries.len() {
            let (bubble, _) = entries
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            let mut exists = false;
            let mut j = 0;
            while j < ids.len() {
                exists |= ids.get(j) == Some(bubble);
                j += 1;
            }
            if !exists {
                push(&mut ids, bubble)?;
            }
            i += 1;
        }
        Ok(ids)
    }
    fn apply_parent_method(
        park: &mut OriginalParkProjection,
        action: NetworkAction,
    ) -> Result<(), OriginalParkError> {
        match action.name {
            ActionName::CloakBall
            | ActionName::RemoveBall
            | ActionName::AddBallsToPark
            | ActionName::BallNotGlobal
            | ActionName::RemoveGlobalBall => (),
            ActionName::GotoPoint => {
                park.goto_point(id_parameter(&action, 0)?, point_parameters(&action)?)
            }
            ActionName::SetBallPosition => {
                park.set_position(id_parameter(&action, 0)?, point_parameters(&action)?)
            }
            ActionName::UncloakBall => park.uncloak_ball(id_parameter(&action, 0)?),
            ActionName::SetBallInteractive => {
                park.set_interactive(id_parameter(&action, 0)?, parameter(&action, 1)?.truthy())
            }
            ActionName::SetSpeedFraction => park.set_speed_fraction(
                id_parameter(&action, 0)?,
                number(parameter(&action, 1)?) as f32,
            ),
            ActionName::Stop => park.stop(id_parameter(&action, 0)?),
            _ => return Err(OriginalParkError::OutsideDeclaredScope),
        }
        if park.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }
    fn apply_history(
        &mut self,
        park: &mut OriginalParkProjection,
    ) -> Result<(), OriginalParkError> {
        let history = self.current_system_history;
        let mut i = 0;
        while i < history.len() {
            let entry = history
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            let mut event_ball_id = integer(entry.ball_id)?;
            let action = entry.action;
            let event = self.event(entry.timestamp, action)?; // strip local_args
            match action.name {
                ActionName::CloakBall => {
                    let id = id_parameter(&action, 0)?;
                    if park.contains(id) && self.cloaking_connected {
                        push(&mut self.signal_calls, ServerTickerSignal::Cloaking(id))?;
                    }
                    let bubble = integer(
                        action
                            .local_argument
                            .ok_or(OriginalParkError::OutsideDeclaredScope)?,
                    )?;
                    self.bubble_event(bubble, event)?;
                    let mode = i8::try_from(id_parameter(&action, 1)?)
                        .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
                    park.cloak_ball(id, mode, number(parameter(&action, 2)?) as f32)?;
                    i += 1;
                    continue;
                }
                ActionName::UncloakBall => {
                    let id = id_parameter(&action, 0)?;
                    if id != event_ball_id {
                        event_ball_id = id;
                    }
                    if !park.contains(event_ball_id) {
                        i += 1;
                        continue;
                    }
                    if self.uncloaking_connected {
                        push(
                            &mut self.signal_calls,
                            ServerTickerSignal::Uncloaking(event_ball_id),
                        )?;
                    }
                    let bubble = park.retained(event_ball_id).new_bubble;
                    let interactives = park.bubble_interactive_entries();
                    let mut add_event = None;
                    let mut j = 0;
                    while j < interactives.len() {
                        let (key, ball) = interactives
                            .get(j)
                            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                        if key == bubble {
                            // Source reads element 0 on every iteration. Empty
                            // TestBallInfo lists must fail instead of becoming None.
                            let characters = self
                                .interests
                                .ball_info_characters(event_ball_id)
                                .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
                            let character = characters
                                .get(0)
                                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                            if ball == event_ball_id {
                                self.character_event(character, event, false)?;
                            } else {
                                if add_event.is_none() {
                                    let mut ids = BoundedList::new();
                                    push(&mut ids, event_ball_id)?;
                                    let stream = park.write_original_balls(&ids)?;
                                    let index = self
                                        .batcher
                                        .retain_event_stream(stream)
                                        .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
                                    add_event = Some(BatcherEvent {
                                        timestamp: entry.timestamp,
                                        name: ActionName::AddBallsToPark,
                                        payload: BatcherPayload::Stream {
                                            stream_index: index,
                                            reference: None,
                                        },
                                    });
                                }
                                let recipients = self
                                    .interests
                                    .ball_info_characters(ball)
                                    .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
                                let mut k = 0;
                                while k < recipients.len() {
                                    let recipient = recipients
                                        .get(k)
                                        .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                                    if recipient != 0 {
                                        self.character_event(
                                            recipient,
                                            add_event
                                                .ok_or(OriginalParkError::OutsideDeclaredScope)?,
                                            true,
                                        )?;
                                    }
                                    k += 1;
                                }
                            }
                        }
                        j += 1;
                    }
                    i += 1;
                    continue;
                }
                ActionName::SetBallPosition => {
                    let local = action.local_argument == Some(ActionValue::Boolean(true));
                    if self.position_connected {
                        push(
                            &mut self.signal_calls,
                            ServerTickerSignal::SetPosition(event_ball_id, local),
                        )?;
                    }
                    if local {
                        i += 1;
                        continue;
                    }
                }
                ActionName::AddBallsToPark
                | ActionName::BallNotGlobal
                | ActionName::RemoveGlobalBall => {
                    let ids = Self::bubble_ids(park)?;
                    let mut j = 0;
                    while j < ids.len() {
                        let bubble = ids.get(j).ok_or(OriginalParkError::OutsideDeclaredScope)?;
                        if action.name == ActionName::AddBallsToPark {
                            self.bubble_event(bubble, event)?;
                        } else if action.name == ActionName::RemoveGlobalBall
                            || bubble != id_parameter(&action, 0)?
                        {
                            let mut args = BoundedList::new();
                            push(&mut args, ActionValue::Integer(event_ball_id))?;
                            self.bubble_event(
                                bubble,
                                BatcherEvent {
                                    timestamp: entry.timestamp,
                                    name: ActionName::RemoveBall,
                                    payload: BatcherPayload::Tuple(args),
                                },
                            )?;
                        }
                        j += 1;
                    }
                    i += 1;
                    continue;
                }
                _ => (),
            }
            let bubble = if park.contains(event_ball_id) {
                park.retained(event_ball_id).new_bubble
            } else if action.name == ActionName::RemoveBall {
                id_parameter(&action, 1)?
            } else {
                -1
            };
            let entries = park.bubble_dictionary_entries();
            let mut exists = false;
            let mut j = 0;
            while j < entries.len() {
                exists |= entries.get(j).is_some_and(|e| e.bubble == bubble);
                j += 1;
            }
            let interactives = park.bubble_interactive_entries();
            let mut interactive = false;
            j = 0;
            while j < interactives.len() {
                interactive |= interactives.get(j).is_some_and(|(key, _)| key == bubble);
                j += 1;
            }
            if exists && interactive && event_ball_id > 0 && park.contains(event_ball_id) {
                if park.retained(event_ball_id).is_cloaked == 0 {
                    self.bubble_event(bubble, event)?;
                } else {
                    let characters = self
                        .interests
                        .ball_info_characters(event_ball_id)
                        .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
                    let character = characters
                        .get(0)
                        .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                    self.character_event(character, event, false)?;
                }
            }
            i += 1;
        }
        Ok(())
    }
    pub fn tick(&mut self, park: &mut OriginalParkProjection) -> Result<(), OriginalParkError> {
        self.actions.clean_up_followers(park)?;
        self.stamp_for_system = park
            .current_time
            .checked_add(1)
            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
        self.actions
            .queue
            .set_stamp_for_system(self.stamp_for_system);
        self.current_system_history = self.actions.queue.flush_history();
        let mut i = 0;
        while i < self.current_system_history.len() {
            let entry = self
                .current_system_history
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            Self::apply_parent_method(park, entry.action)?;
            i += 1;
        }
        self.batcher
            .update_bubbles(park, &mut self.updater, &self.interests)
            .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
        self.apply_history(park)
    }
    pub fn send_batch(&mut self, park: &OriginalParkProjection) -> Result<(), OriginalParkError> {
        let sent = self
            .batcher
            .send_batch(park, &self.interests)
            .map_err(|_| OriginalParkError::OutsideDeclaredScope)?;
        // TestNetwork appends the two lists; it does not replace earlier casts.
        if !self.network.singlecasts.extend(&sent.singlecasts)
            || !self.network.narrowcasts.extend(&sent.narrowcasts)
        {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.network.dispatch_order = sent.dispatch_order;
        self.network.timestamp_mismatches = sent.timestamp_mismatches;
        Ok(())
    }
    pub fn clear_network(&mut self) {
        self.network = OriginalNetworkOutput::default();
    }
}
