// Copyright © 2014-2023 CCP ehf. MIT license, retained from original Destiny.
//! Ball/Ballpark state transitions at the pinned behavioral revision.
//!
//! This is a projection of the original in-park operation, not a native park.
//! `boundary_calls` retains the ordered spatial/Blue/clock calls that require
//! separate correspondence. It does not implement or certify those boundaries.
//! Rust's platform math operations execute the original C math formulas.
//! Kani's builtin math models and each original platform's math implementation
//! remain an explicit numerical correspondence obligation.
//! Eight retained objects and 4096 boundary calls bound these concrete scenarios.

use crate::original_partition::{
    CellKey, OriginalBubbleProjection, SphericalCellKeys, TOP_WIDTH, add_ancestor_cells, box_low,
    box_snapshot, box_width, footprints_touch, grid_center, primary_key, spherical_candidates,
    spherical_cell_keys,
};
use crate::{
    Aabb, BoundedList, BoxShape, Triangle, Vec3, free_formation_slot, original_goto_thrust,
    original_integrate_tick, original_stop_velocity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalMode {
    Stop,
    Goto,
    Follow,
    Orbit,
    Missile,
    Warp,
    Formation,
    Boid,
    Troll,
    Rigid,
    Field,
    Miniball,
    Mushroom,
}

/// Blue child descriptors store local coordinates, float radii and signed IDs.
/// They are distinct from the corresponding collidable in the park.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalMiniBall {
    pub id: i64,
    pub position: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalMiniCapsule {
    pub id: i64,
    pub parent_id: i64,
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalMiniBox {
    pub id: i64,
    pub parent_id: i64,
    pub corner: Vec3,
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalParkError {
    ValueError,
    RuntimeError,
    TypeError,
    OutsideDeclaredScope,
}

/// Concrete original SetBoxObject fixtures use this one primitive dictionary
/// key. Object identity, aliases, reference counts and arbitrary Python values
/// remain separate interpreter/boundary obligations, not Rust claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalBoxObjectKey {
    Foo,
}
pub type OriginalBoxObject = BoundedList<(OriginalBoxObjectKey, crate::ActionValue), 4>;

/// Members explicitly handled by Ball::OnModified. Other exposed members
/// fall through to its final `return true` without an additional park call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalBallAttribute {
    Position,
    Velocity,
    Mass,
    Radius,
    MaxSpeed,
    MaxAngularSpeed,
    RotationalAgility,
    Free,
    Massive,
    Global,
    Agility,
    SpeedFraction,
    Harmonic,
    Corporation,
    Alliance,
    NoHandler,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalCapsule {
    pub id: i64,
    pub parent_id: i64,
    pub a: Vec3,
    pub b: Vec3,
    pub radius: f32,
    pub ab: Vec3,
    pub position: Vec3,
    pub old_position: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalOrientedBox {
    pub id: i64,
    pub parent_id: i64,
    pub shape: BoxShape,
    pub bounding_radius: f32,
    pub position: Vec3,
    pub old_position: Vec3,
    pub aabb: Aabb,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalProximitySensor {
    pub range: f32,
    pub period: f64,
    pub elapsed: f64,
    pub active: bool,
    pub only_interactives: bool,
    pub balls: BoundedList<i64, 8>,
}

impl Default for OriginalProximitySensor {
    fn default() -> Self {
        Self {
            range: 0.0,
            period: 2.0,
            elapsed: 0.0,
            active: false,
            only_interactives: false,
            balls: BoundedList::new(),
        }
    }
}

impl OriginalMode {
    fn follows(self) -> bool {
        matches!(
            self,
            Self::Follow | Self::Orbit | Self::Missile | Self::Formation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalParkBall {
    pub rotation: [f32; 4],
    pub old_rotation: [f32; 4],
    pub angular_velocity: [f32; 3],
    pub old_angular_velocity: [f32; 3],
    pub id: i64,
    pub in_park: bool,
    pub client_ball: bool,
    pub mass: f64,
    pub radius: f32,
    pub max_velocity: f32,
    pub max_angular_velocity: f32,
    pub rotational_agility: f32,
    pub client_max_angular_velocity: f32,
    pub show_boxes: bool,
    pub cached_center_distance: f64,
    pub agility: f32,
    pub speed_fraction: f32,
    pub time_factor: f64,
    pub position: Vec3,
    pub old_position: Vec3,
    pub velocity: Vec3,
    pub old_velocity: Vec3,
    pub last_g: Vec3,
    pub last_c: Vec3,
    pub last_collision: f64,
    pub collisions: BoundedList<i64, 8>,
    pub mode: OriginalMode,
    pub follow_id: i64,
    pub follow_range: f32,
    pub owner_id: i64,
    pub effect_stamp: i64,
    pub formation_id: i8,
    pub formation_slots: u16,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_space_junk: bool,
    pub is_moribund: bool,
    pub is_cloaked: i8,
    pub harmonic: i64,
    pub corporation_id: i32,
    pub alliance_id: i32,
    pub goto: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub old_yaw: f64,
    pub old_pitch: f64,
    pub old_roll: f64,
    pub roll_speed: f64,
    pub old_roll_speed: f64,
    pub yaw_delta: f64,
    pub new_time: i64,
    pub old_time: i64,
    pub pos_update_time: i64,
    pub rot_update_time: i64,
    pub new_bubble: i64,
    pub old_bubble: i64,
    pub mini_balls: BoundedList<OriginalMiniBall, 4>,
    pub mini_capsules: BoundedList<OriginalMiniCapsule, 4>,
    pub mini_boxes: BoundedList<OriginalMiniBox, 4>,
    pub sensor: OriginalProximitySensor,
    pub tracking_range: f64,
    pub notification_range: f64,
    pub has_proximity: bool,
    pub tracking_id: Option<i64>,
    pub within_range: bool,
    pub within_tracking_range: bool,
    pub event_spy: bool,
    pub range_callback_args: BoundedList<i64, 8>,
    pub target_tracking_callback_args: BoundedList<i64, 8>,
    pub proximity_callback_args: BoundedList<(i64, i64), 8>,
}

impl Default for OriginalParkBall {
    fn default() -> Self {
        let position = Vec3 {
            x: 1e10,
            y: 1e10,
            z: 1e10,
        };
        Self {
            rotation: [0.0, 0.0, 0.0, 1.0],
            old_rotation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            old_angular_velocity: [0.0; 3],
            id: 0,
            in_park: false,
            client_ball: false,
            mass: 0.0,
            radius: 0.0,
            max_velocity: 0.0,
            max_angular_velocity: 1.0,
            rotational_agility: 0.9,
            client_max_angular_velocity: 0.5_f32 * (core::f64::consts::PI as f32),
            show_boxes: false,
            cached_center_distance: -1.0,
            agility: 1.0,
            speed_fraction: 1.0,
            time_factor: 1.0,
            position,
            old_position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            old_velocity: Vec3::ZERO,
            last_g: Vec3::ZERO,
            last_c: Vec3::ZERO,
            last_collision: -1.0,
            collisions: BoundedList::new(),
            mode: OriginalMode::Stop,
            follow_id: 0,
            follow_range: 10.0,
            owner_id: 0,
            effect_stamp: 0,
            formation_id: -1,
            formation_slots: 0,
            is_free: false,
            is_global: false,
            is_massive: false,
            is_interactive: false,
            is_space_junk: false,
            is_moribund: false,
            is_cloaked: 0,
            harmonic: -1,
            corporation_id: -1,
            alliance_id: -1,
            goto: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            old_yaw: 0.0,
            old_pitch: 0.0,
            old_roll: 0.0,
            roll_speed: 0.0,
            old_roll_speed: 0.0,
            yaw_delta: 0.0,
            new_time: 0,
            old_time: 0,
            pos_update_time: 0,
            rot_update_time: 0,
            new_bubble: -1,
            old_bubble: -1,
            mini_balls: BoundedList::new(),
            mini_capsules: BoundedList::new(),
            mini_boxes: BoundedList::new(),
            sensor: OriginalProximitySensor::default(),
            tracking_range: 0.0,
            notification_range: 0.0,
            has_proximity: false,
            tracking_id: None,
            within_range: false,
            within_tracking_range: false,
            event_spy: false,
            range_callback_args: BoundedList::new(),
            target_tracking_callback_args: BoundedList::new(),
            proximity_callback_args: BoundedList::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalBallInit {
    pub id: i64,
    pub mass: f64,
    pub radius: f32,
    pub max_velocity: f32,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_space_junk: bool,
    pub position: Vec3,
    pub velocity: Vec3,
    pub agility: f32,
    pub speed_fraction: f32,
}

impl Default for OriginalBallInit {
    fn default() -> Self {
        Self {
            id: 0,
            mass: 0.0,
            radius: 0.0,
            max_velocity: 0.0,
            is_free: false,
            is_global: false,
            is_massive: false,
            is_interactive: false,
            is_space_junk: false,
            position: Vec3 {
                x: 1e10,
                y: 1e10,
                z: 1e10,
            },
            velocity: Vec3::ZERO,
            agility: 0.0,
            speed_fraction: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OriginalBoundaryCall {
    InsertBoxes(i64),
    DeleteBoxes(i64),
    AddToBubble(i64),
    RemoveMiniShapes(i64),
    AddMiniShapes(i64),
    UpdateInteractiveMembership(i64),
    ModeChange(i64, OriginalMode, OriginalMode),
    ExitWarp(i64, bool),
    ActivateWarp(i64, i64),
    DeactivateWarp(i64, i64),
    FinalCleanup(i64),
    RegisterTicks,
    UnregisterTicks,
    ClearAll,
    InsertStaticCollidable(i64),
    ProximityMembership(i64, bool),
    AttributeNotification(i64, OriginalBallAttribute),
    ClientQuaternionSnap(i64, f32, f32),
    BallNotInParkCallback(i64),
    RangeEvent(i64, bool),
    TargetTrackingEvent(i64, bool),
    ProximityEvent(i64, i64, bool),
}

pub struct OriginalParkProjection {
    balls: [Option<OriginalParkBall>; 8],
    pub boundary_calls: BoundedList<OriginalBoundaryCall, 4096>,
    pub scope_error: bool,
    pub time: i64,
    pub current_time: i64,
    pub is_running: bool,
    pub is_master: bool,
    pub evolving: bool,
    pub formations_loaded: bool,
    pub ego_id: i64,
    pub dynamical_orientation: bool,
    pub iterative_collision: bool,
    pub friction: f64,
    collision_state: crate::original_iterative_collision::OriginalIterativeState,
    pub use_new_orbit: bool,
    pub capsules: BoundedList<OriginalCapsule, 8>,
    pub oriented_boxes: BoundedList<OriginalOrientedBox, 8>,
    // StaticCollidable::mBoxes records these footprints at insertion. Query
    // and collision passes observe that state; they do not resample shapes.
    capsule_boxes: BoundedList<crate::original_static_partition::StaticCellKeys, 8>,
    oriented_box_boxes: BoundedList<crate::original_static_partition::StaticCellKeys, 8>,
    pub bubble_transitions: BoundedList<(i64, i64, i64), 8>,
    /// Single-park projection of the source's static Python callback. These
    /// cases replace it before calling; interpreter/global reference lifetime
    /// and WrapBlueObject correspondence are separate requirements.
    pub ball_not_in_park_callback_enabled: bool,
    pub ball_not_in_park_callback_called: bool,
    /// Exact EventHandlingBall fixture from evolve/test_add.py. Blue/Python
    /// registration and exception dispatch are external correspondence duties.
    pub warp_finished_add_handler: Option<i64>,
    pub warp_done: bool,
    pub ball_added_during_evolve: Option<i64>,
    pub mode_callback_add_attempted: bool,
    pub mode_callback_error: Option<OriginalParkError>,
    local_high_counter: i64,
    local_counter: i64,
    query_bubbles: Option<OriginalBubbleProjection>,
    ball_boxes: [Option<SphericalCellKeys>; 8],
    free_ids: BoundedList<i64, 8>,
    proximity_ids: BoundedList<i64, 8>,
    box_objects: BoundedList<(CellKey, OriginalBoxObject), 8>,
}

impl Default for OriginalParkProjection {
    fn default() -> Self {
        Self {
            balls: [None; 8],
            boundary_calls: BoundedList::new(),
            scope_error: false,
            time: 0,
            current_time: 0,
            is_running: false,
            is_master: false,
            evolving: false,
            formations_loaded: false,
            ego_id: 0,
            dynamical_orientation: false,
            iterative_collision: false,
            friction: 1_000_000.0,
            collision_state: crate::original_iterative_collision::OriginalIterativeState::default(),
            use_new_orbit: false,
            capsules: BoundedList::new(),
            oriented_boxes: BoundedList::new(),
            capsule_boxes: BoundedList::new(),
            oriented_box_boxes: BoundedList::new(),
            bubble_transitions: BoundedList::new(),
            ball_not_in_park_callback_enabled: false,
            ball_not_in_park_callback_called: false,
            warp_finished_add_handler: None,
            warp_done: false,
            ball_added_during_evolve: None,
            mode_callback_add_attempted: false,
            mode_callback_error: None,
            local_high_counter: -(1_i64 << 30),
            local_counter: -1,
            query_bubbles: None,
            ball_boxes: [None; 8],
            free_ids: BoundedList::new(),
            proximity_ids: BoundedList::new(),
            box_objects: BoundedList::new(),
        }
    }
}

impl OriginalParkProjection {
    /// Actual old-style Python thunk guard, after typed argument conversion.
    /// RuntimeError is raised before native allocation/ID generation. It is
    /// separate from this projection's explicit capacity/representation errors.
    pub fn add_original_ball_checked(
        &mut self,
        init: OriginalBallInit,
    ) -> Result<i64, OriginalParkError> {
        if self.evolving {
            return Err(OriginalParkError::RuntimeError);
        }
        let id = self.add_ball(init);
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(id)
        }
    }
    fn inflated_radius(&self, b: &OriginalParkBall) -> f64 {
        if b.is_free {
            let m = b.mass * f64::from(b.agility);
            let a = self.friction * f64::from(b.speed_fraction) * f64::from(b.max_velocity) / m;
            let v = b.velocity.length();
            f64::from(b.radius)
                + (m * a - (m * a - v * self.friction) * b.time_factor) / self.friction
        } else {
            f64::from(b.radius)
        }
    }

    fn update_free_registry(&mut self, id: i64, enabled: bool) {
        let mut updated = BoundedList::new();
        let mut inserted = false;
        let mut i = 0;
        while i < self.free_ids.len() {
            if let Some(old) = self.free_ids.get(i) {
                if old > id && enabled && !inserted {
                    if !updated.push(id) {
                        self.scope_error = true;
                    }
                    inserted = true;
                }
                if old == id {
                    if enabled && !inserted {
                        if !updated.push(id) {
                            self.scope_error = true;
                        }
                        inserted = true;
                    }
                } else if !updated.push(old) {
                    self.scope_error = true;
                }
            }
            i += 1;
        }
        if enabled && !inserted && !updated.push(id) {
            self.scope_error = true;
        }
        self.free_ids = updated;
    }

    fn in_free_registry(&self, id: i64) -> bool {
        let mut i = 0;
        while i < self.free_ids.len() {
            if self.free_ids.get(i) == Some(id) {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Thunkers.cpp:1193 iterates the actual free map, not isFree flags. It
    /// includes a following body when its target is missing or in a different
    /// bubble. A moribund follower is logged but still included by the source.
    pub fn remote_followers(&self) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < self.free_ids.len() {
            let Some(id) = self.free_ids.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if !self.contains(id) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            let source = self.retained(id);
            if source.mode.follows()
                && (!self.contains(source.follow_id)
                    || source.new_bubble != self.retained(source.follow_id).new_bubble)
                && !result.push(id)
            {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            i += 1;
        }
        Ok(result)
    }

    fn reinsert_query_master_ball(&mut self, id: i64) -> Result<(), OriginalParkError> {
        let mut b = self.retained(id);
        let Some(index) = self.index(id, true) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let Some(new) = spherical_cell_keys(b.position, b.radius, self.inflated_radius(&b)) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let old = self.ball_boxes.get(index).copied().flatten();
        let mut others = BoundedList::new();
        let mut i = 0;
        while i < 8 {
            if i != index
                && self
                    .balls
                    .get(i)
                    .is_some_and(|b| b.is_some_and(|b| b.in_park))
            {
                let Some(keys) = self.ball_boxes.get(i).copied().flatten() else {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                };
                if !others.push(keys) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        let Some(projection) = &mut self.query_bubbles else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let Some(bubble) = projection.reinsert_master_sphere(new, old, &others) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        if b.new_bubble != bubble {
            b.old_bubble = b.new_bubble;
            b.new_bubble = bubble;
            projection.dictionary.transition(b)?;
        }
        self.store(b);
        self.boundary(OriginalBoundaryCall::InsertBoxes(id));
        Ok(())
    }

    fn boundary(&mut self, call: OriginalBoundaryCall) {
        match call {
            OriginalBoundaryCall::InsertBoxes(id) => {
                if let Some(i) = self.index(id, false) {
                    let b = self.retained(id);
                    let boxes = spherical_cell_keys(b.position, b.radius, self.inflated_radius(&b));
                    if let Some(slot) = self.ball_boxes.get_mut(i) {
                        *slot = boxes;
                    }
                }
            }
            OriginalBoundaryCall::DeleteBoxes(id) => {
                if let Some(i) = self.index(id, false) {
                    if let Some(slot) = self.ball_boxes.get_mut(i) {
                        *slot = None;
                    }
                }
            }
            OriginalBoundaryCall::ProximityMembership(id, enabled) => {
                let mut updated = BoundedList::new();
                let mut inserted = false;
                let mut i = 0;
                while i < self.proximity_ids.len() {
                    if let Some(old) = self.proximity_ids.get(i) {
                        if old > id && enabled && !inserted {
                            if !updated.push(id) {
                                self.scope_error = true;
                            }
                            inserted = true;
                        }
                        if old == id {
                            if enabled && !inserted {
                                if !updated.push(id) {
                                    self.scope_error = true;
                                }
                                inserted = true;
                            }
                        } else if !updated.push(old) {
                            self.scope_error = true;
                        }
                    }
                    i += 1;
                }
                if enabled && !inserted && !updated.push(id) {
                    self.scope_error = true;
                }
                self.proximity_ids = updated;
            }
            _ => (),
        }
        self.record_boundary(call);
    }

    fn record_boundary(&mut self, call: OriginalBoundaryCall) {
        if !self.boundary_calls.push(call) {
            self.scope_error = true;
        }
    }

    fn index(&self, id: i64, require_in_park: bool) -> Option<usize> {
        let mut detached = None;
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(b)) = self.balls.get(i) {
                if b.id == id {
                    if b.in_park {
                        return Some(i);
                    }
                    if !require_in_park && detached.is_none() {
                        detached = Some(i);
                    }
                }
            }
            i = i.saturating_add(1);
        }
        detached
    }

    #[must_use]
    pub fn contains(&self, id: i64) -> bool {
        self.index(id, true).is_some()
    }

    /// Retains a removed object as the original Python variable does.
    #[must_use]
    pub fn retained(&self, id: i64) -> OriginalParkBall {
        self.index(id, false)
            .and_then(|i| self.balls.get(i))
            .copied()
            .flatten()
            .unwrap_or_default()
    }

    /// Blue MAP_ATTRIBUTE writes the member directly. Notification dispatch is
    /// retained as a boundary, rather than replacing this with a park setter.
    pub fn write_attribute(&mut self, id: i64, update: impl FnOnce(&mut OriginalParkBall)) {
        if self.index(id, false).is_none() {
            self.scope_error = true;
            return;
        }
        let mut b = self.retained(id);
        update(&mut b);
        self.store(b);
    }

    /// Port of the original C++ notification callback for a member assignment.
    /// The Blue adapter's invocation timing/representation remains external.
    /// Raw writes happen first; setters may reject the already-written value,
    /// which does not restore its previous value in the original callback.
    pub fn write_attribute_notified(
        &mut self,
        id: i64,
        update: impl FnOnce(&mut OriginalParkBall),
        attribute: OriginalBallAttribute,
    ) {
        let previous = self.retained(id);
        self.write_attribute(id, update);
        if attribute == OriginalBallAttribute::MaxAngularSpeed && previous.client_ball {
            // ClientBall's own maxAngularVelocity shadows the base exposure,
            // is READWRITE without NOTIFY, and affects impact rotation only.
            let mut ball = self.retained(id);
            ball.client_max_angular_velocity = ball.max_angular_velocity;
            ball.max_angular_velocity = previous.max_angular_velocity;
            self.store(ball);
            return;
        }
        let b = self.retained(id);
        if !b.in_park {
            return;
        }
        if attribute != OriginalBallAttribute::NoHandler {
            self.boundary(OriginalBoundaryCall::AttributeNotification(id, attribute));
        }
        match attribute {
            OriginalBallAttribute::Position => self.set_position(id, b.position),
            OriginalBallAttribute::Velocity => self.set_velocity(id, b.velocity),
            OriginalBallAttribute::Mass => self.set_mass(id, b.mass),
            OriginalBallAttribute::Radius => self.set_radius(id, b.radius),
            OriginalBallAttribute::MaxSpeed => self.set_max_speed(id, b.max_velocity),
            OriginalBallAttribute::MaxAngularSpeed => {
                if !(b.max_angular_velocity < 0.0) {
                    self.write_attribute(id, |ball| {
                        ball.max_angular_velocity = b.max_angular_velocity
                    });
                }
            }
            OriginalBallAttribute::RotationalAgility => {
                if b.rotational_agility > 0.0 {
                    self.write_attribute(id, |ball| ball.rotational_agility = b.rotational_agility);
                }
            }
            OriginalBallAttribute::Free => self.set_free(id, b.is_free),
            OriginalBallAttribute::Massive => self.set_massive(id, b.is_massive),
            OriginalBallAttribute::Global => self.set_global(id, b.is_global),
            OriginalBallAttribute::Agility => self.set_agility(id, b.agility),
            OriginalBallAttribute::SpeedFraction => self.set_speed_fraction(id, b.speed_fraction),
            OriginalBallAttribute::Harmonic
            | OriginalBallAttribute::Corporation
            | OriginalBallAttribute::Alliance => {
                self.set_harmonic(id, b.harmonic, b.corporation_id, b.alliance_id, false)
            }
            OriginalBallAttribute::NoHandler => {}
        }
    }

    fn store(&mut self, ball: OriginalParkBall) {
        if let Some(i) = self.index(ball.id, false) {
            if let Some(slot) = self.balls.get_mut(i) {
                *slot = Some(ball);
                return;
            }
        }
        self.scope_error = true;
    }

    #[must_use]
    pub fn count(&self) -> usize {
        let mut count = 0_usize;
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(b)) = self.balls.get(i) {
                if b.in_park {
                    count = count.saturating_add(1);
                }
            }
            i = i.saturating_add(1);
        }
        count
    }

    #[must_use]
    pub fn current_ego_position_without_ego(&mut self) -> (f64, f64, f64) {
        if self.ego_id != 0 {
            self.scope_error = true;
        }
        (0.0, 0.0, 0.0)
    }

    fn generate_id(&mut self, requested: i64) -> i64 {
        if (self.is_master && requested < 0) || (!self.is_master && requested == -1) {
            if let Some(next) = self.local_high_counter.checked_sub(1) {
                self.local_high_counter = next;
                next
            } else {
                self.scope_error = true;
                requested
            }
        } else if self.is_master && requested == 0 {
            if let Some(next) = self.local_counter.checked_sub(1) {
                self.local_counter = next;
                next
            } else {
                self.scope_error = true;
                requested
            }
        } else {
            requested
        }
    }

    pub fn add_ball(&mut self, init: OriginalBallInit) -> i64 {
        if self.evolving {
            self.scope_error = true;
            return init.id;
        }
        let id = self.generate_id(init.id);
        let replacing = self.contains(id);
        let mut ball = self.new_ball_fields(init, id);
        Self::update_time_factor(&mut ball, self.friction);
        self.finish_add_ball(ball, replacing);
        self.snap_added_old_style_ball(id);
        id
    }

    /// Concrete fixture branch of SetBallTimeFactor: the actual product must
    /// satisfy the original `mass <= 0` guard. Failure is an explicit scope
    /// error and is checked by the caller; there is no `kani::assume` or math
    /// stub. Positive products use add_ball's real numerical implementation.
    pub fn add_nonpositive_time_factor_ball(&mut self, init: OriginalBallInit) -> i64 {
        if self.evolving {
            self.scope_error = true;
            return init.id;
        }
        let id = self.generate_id(init.id);
        let replacing = self.contains(id);
        let mut ball = self.new_ball_fields(init, id);
        if ball.mass * f64::from(ball.agility) <= 0.0 {
            ball.time_factor = 0.0;
            self.finish_add_ball(ball, replacing);
            self.snap_added_old_style_ball(id);
        } else {
            self.scope_error = true;
        }
        id
    }

    fn new_ball_fields(&mut self, init: OriginalBallInit, id: i64) -> OriginalParkBall {
        let replacing = self.contains(id);
        let mut previous = if replacing {
            self.boundary(OriginalBoundaryCall::DeleteBoxes(id));
            self.retained(id)
        } else {
            OriginalParkBall::default()
        };
        if replacing {
            previous.formation_id = -1;
            previous.effect_stamp = 0;
            previous.owner_id = 0;
            previous.follow_id = 0;
            previous.follow_range = 10.0;
            previous.corporation_id = -1;
            previous.alliance_id = -1;
            previous.harmonic = -1;
            previous.is_moribund = false;
        }
        OriginalParkBall {
            id,
            in_park: true,
            client_ball: !self.is_master,
            position: init.position,
            old_position: if replacing {
                previous.old_position
            } else {
                init.position.sub(init.velocity)
            },
            velocity: init.velocity,
            old_velocity: if replacing {
                previous.old_velocity
            } else {
                init.velocity
            },
            mass: if init.mass < 0.0 { 0.0 } else { init.mass },
            radius: if init.radius < 0.0 { 0.0 } else { init.radius },
            max_velocity: if init.max_velocity < 0.0 {
                0.0
            } else {
                init.max_velocity
            },
            agility: if init.agility <= 0.0 {
                1.0
            } else {
                init.agility
            },
            speed_fraction: if init.speed_fraction < 0.0 {
                0.0
            } else {
                init.speed_fraction
            },
            is_free: init.is_free,
            is_global: init.is_global,
            is_massive: init.is_massive,
            is_interactive: init.is_interactive,
            is_space_junk: init.is_space_junk,
            ..previous
        }
    }

    fn snap_added_old_style_ball(&mut self, id: i64) {
        if self.contains(id) && self.retained(id).is_free {
            let mut ball = self.retained(id);
            self.calculate_yaw_pitch_roll(&mut ball, true);
            self.store(ball);
        }
    }

    pub(crate) fn live_ball_sequence(
        &self,
    ) -> Result<BoundedList<OriginalParkBall, 8>, OriginalParkError> {
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < 8 {
            if let Some(Some(ball)) = self.balls.get(i).copied() {
                if ball.in_park && !result.push(ball) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    pub(crate) fn reset_stream_registries(&mut self) {
        self.free_ids = BoundedList::new();
        self.proximity_ids = BoundedList::new();
        if let Some(projection) = &mut self.query_bubbles {
            projection.dictionary.interactives = BoundedList::new();
        }
    }

    pub(crate) fn remove_stream_children(&mut self, id: i64) -> Result<(), OriginalParkError> {
        let mut ball = self.retained(id);
        let mut i = ball.mini_balls.len();
        while i > 0 {
            i -= 1;
            let Some(child) = ball.mini_balls.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            self.remove_ball(child.id, 0);
        }
        let mut capsules = BoundedList::new();
        let mut capsule_boxes = BoundedList::new();
        i = 0;
        while i < self.capsules.len() {
            let Some(object) = self.capsules.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let Some(keys) = self.capsule_boxes.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if object.parent_id != id && (!capsules.push(object) || !capsule_boxes.push(keys)) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            i += 1;
        }
        let mut boxes = BoundedList::new();
        let mut box_keys = BoundedList::new();
        i = 0;
        while i < self.oriented_boxes.len() {
            let Some(object) = self.oriented_boxes.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let Some(keys) = self.oriented_box_boxes.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if object.parent_id != id && (!boxes.push(object) || !box_keys.push(keys)) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            i += 1;
        }
        self.capsules = capsules;
        self.capsule_boxes = capsule_boxes;
        self.oriented_boxes = boxes;
        self.oriented_box_boxes = box_keys;
        ball.mini_balls = BoundedList::new();
        ball.mini_capsules = BoundedList::new();
        ball.mini_boxes = BoundedList::new();
        self.store(ball);
        Ok(())
    }

    fn finish_add_ball(&mut self, ball: OriginalParkBall, replacing: bool) {
        self.update_free_registry(ball.id, ball.is_free);
        if replacing {
            self.store(ball);
            self.set_mode(ball.id, OriginalMode::Stop);
            self.boundary(OriginalBoundaryCall::InsertBoxes(ball.id));
        } else {
            self.insert_new_ball(ball);
        }
    }

    /// Add the actual child descriptor and, for a non-free parent, its static
    /// ball. The finite mass times zero branch is checked rather than assumed.
    /// Blue ownership and spatial indexing remain boundary obligations.
    pub fn add_mini_ball(
        &mut self,
        parent_id: i64,
        position: Vec3,
        radius: f32,
    ) -> Result<(), OriginalParkError> {
        if !self.contains(parent_id) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut parent = self.retained(parent_id);
        let index = parent.mini_balls.len();
        if !parent.mini_balls.push(OriginalMiniBall {
            id: 0,
            position,
            radius,
        }) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.store(parent);
        if !parent.is_free {
            let id = self.generate_id(-1);
            let absolute = parent.position.add(position);
            let mass = if parent.mass < 0.0 { 0.0 } else { parent.mass };
            if !(mass * 0.0 <= 0.0) {
                self.scope_error = true;
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            let child = OriginalParkBall {
                id,
                in_park: true,
                client_ball: !self.is_master,
                mass,
                radius: if radius < 0.0 { 0.0 } else { radius },
                position: absolute,
                old_position: absolute,
                owner_id: parent_id,
                agility: 0.0,
                speed_fraction: 0.0,
                is_massive: true,
                time_factor: 0.0,
                mode: OriginalMode::Miniball,
                ..OriginalParkBall::default()
            };
            self.insert_new_ball(child);
            parent.mini_balls.set(
                index,
                OriginalMiniBall {
                    id,
                    position,
                    radius,
                },
            );
            self.store(parent);
        }
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }

    /// Ballpark.cpp:3719..3789, old-style orientation branch. The Python
    /// wrapper narrows range to float and keeps time as double before guards.
    pub fn add_mushroom(&mut self, owner_id: i64, range: f32, duration: f64) {
        if range <= 0.0 || duration <= 0.0 || !self.contains(owner_id) {
            return;
        }
        if self.dynamical_orientation {
            self.scope_error = true;
            return;
        }
        let position = self.retained(owner_id).position;
        let id = self.add_ball(OriginalBallInit {
            id: 0,
            mass: f64::from(1.0e20_f32),
            radius: 0.0,
            max_velocity: 0.0,
            is_free: false,
            is_global: false,
            is_massive: true,
            is_interactive: false,
            is_space_junk: false,
            position,
            velocity: Vec3::ZERO,
            agility: 1.0,
            speed_fraction: 1.0,
        });
        if !self.contains(id) {
            self.scope_error = true;
            return;
        }
        let mut mushroom = self.retained(id);
        mushroom.follow_range = range;
        mushroom.goto.x = duration;
        mushroom.owner_id = owner_id;
        mushroom.effect_stamp = self.current_time;
        self.store(mushroom);
        self.set_mode(id, OriginalMode::Mushroom);
    }

    /// The original mushroom examples contain only a non-free owner and its
    /// mushroom. Owner exclusion in Gradient prevents contact; check this
    /// source partition explicitly instead of assuming no collision response.
    /// The original Evolve integration precedes expansion, and expansion skips
    /// position/velocity swap and YPR. Removal during Evolve marks moribund.
    pub fn evolve_owner_and_mushroom(&mut self) -> Result<(), OriginalParkError> {
        if self.is_master
            || self.dynamical_orientation
            || self.iterative_collision
            || !self.capsules.is_empty()
            || !self.oriented_boxes.is_empty()
        {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mushroom = self.retained(0);
        if !self.contains(0) || mushroom.mode != OriginalMode::Mushroom {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(ball)) = self.balls.get(i).copied() {
                if ball.in_park
                    && (ball.sensor.active
                        || ball.tracking_range != 0.0
                        || ball.notification_range != 0.0
                        || (ball.id != 0 && (ball.id != mushroom.owner_id || ball.is_free)))
                {
                    self.scope_error = true;
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i = i.saturating_add(1);
        }
        self.evolving = true;
        if mushroom.is_free && !mushroom.is_moribund {
            let mut ball = mushroom;
            ball.last_g = Vec3::ZERO;
            ball.last_c = Vec3::ZERO;
            ball.collisions = BoundedList::new();
            if ball.is_massive {
                ball.last_collision = -1.0;
            }
            let (p, v) = original_integrate_tick(
                ball.position,
                ball.velocity,
                ball.last_g.add(ball.last_c),
                ball.mass * f64::from(ball.agility),
                1_000_000.0,
                ball.time_factor,
            );
            ball.old_position = p;
            ball.old_velocity = v;
            let elapsed = self
                .current_time
                .checked_sub(ball.effect_stamp)
                .and_then(|ticks| ticks.checked_mul(1_000));
            let Some(elapsed) = elapsed else {
                self.scope_error = true;
                self.evolving = false;
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let fraction = (elapsed as f32) * 0.001_f32 / (ball.goto.x as f32);
            self.store(ball);
            if fraction < 1.0 {
                // <math.h>'s float,float pow overload and the host Rust float
                // intrinsic call the platform float power operation. The
                // portable libm 0.2.16 implementation differs by one float ULP
                // at the original 0.2f sample and fails its unchanged tolerance.
                // Kani support and cross-platform numerical correspondence for
                // this genuine external math call must be checked separately.
                ball.radius = ball.follow_range * fraction.powf(0.25_f32);
                self.store(ball);
                self.boundary(OriginalBoundaryCall::InsertBoxes(ball.id));
            } else {
                self.remove_ball(ball.id, 0);
            }
        }
        let next = self.current_time.checked_add(1);
        self.evolving = false;
        if let Some(next) = next {
            self.current_time = next;
            Ok(())
        } else {
            self.scope_error = true;
            Err(OriginalParkError::OutsideDeclaredScope)
        }
    }

    /// PyAddMiniCapsule narrows radius to float before the nonpositive guard.
    /// NaN does not satisfy this comparison and is therefore not rejected.
    pub fn add_mini_capsule(
        &mut self,
        parent_id: i64,
        a: Vec3,
        b: Vec3,
        radius: f32,
    ) -> Result<(), OriginalParkError> {
        if radius <= 0.0 {
            return Err(OriginalParkError::ValueError);
        }
        if !self.contains(parent_id) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut parent = self.retained(parent_id);
        let index = parent.mini_capsules.len();
        let mut descriptor = OriginalMiniCapsule {
            id: 0,
            parent_id,
            a,
            b,
            radius,
        };
        if !parent.mini_capsules.push(descriptor) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.store(parent);
        if !parent.is_free {
            descriptor.id = self.generate_id(-1);
            let absolute_a = parent.position.add(a);
            let absolute_b = parent.position.add(b);
            let position = absolute_a.add(absolute_b).scale(0.5);
            let capsule = OriginalCapsule {
                id: descriptor.id,
                parent_id,
                a: absolute_a,
                b: absolute_b,
                radius,
                ab: absolute_b.sub(absolute_a),
                position,
                old_position: position,
            };
            let keys = crate::original_static_partition::capsule_keys(&capsule)?;
            if !self.capsules.push(capsule) || !self.capsule_boxes.push(keys) {
                self.scope_error = true;
            }
            self.boundary(OriginalBoundaryCall::InsertStaticCollidable(descriptor.id));
            parent.mini_capsules.set(index, descriptor);
            self.store(parent);
        }
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }

    pub fn add_mini_box(
        &mut self,
        parent_id: i64,
        corner: Vec3,
        x: Vec3,
        y: Vec3,
        z: Vec3,
    ) -> Result<(), OriginalParkError> {
        if !self.contains(parent_id) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut parent = self.retained(parent_id);
        let index = parent.mini_boxes.len();
        let mut descriptor = OriginalMiniBox {
            id: 0,
            parent_id,
            corner,
            x,
            y,
            z,
        };
        if !parent.mini_boxes.push(descriptor) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.store(parent);
        if !parent.is_free {
            descriptor.id = self.generate_id(-1);
            let shape = BoxShape::new(parent.position.add(corner), x, y, z);
            let position = shape.center();
            let mut aabb = Aabb {
                low: shape.corners[0],
                high: shape.corners[0],
            };
            let mut i = 1_usize;
            while i < 8 {
                if let Some(p) = shape.corners.get(i) {
                    aabb.update(*p);
                }
                i = i.saturating_add(1);
            }
            let object = OriginalOrientedBox {
                id: descriptor.id,
                parent_id,
                bounding_radius: shape.bounding_radius() as f32,
                position,
                old_position: position,
                aabb,
                shape,
            };
            let keys = crate::original_static_partition::oriented_box_keys(&object)?;
            if !self.oriented_boxes.push(object) || !self.oriented_box_boxes.push(keys) {
                self.scope_error = true;
            }
            self.boundary(OriginalBoundaryCall::InsertStaticCollidable(descriptor.id));
            parent.mini_boxes.set(index, descriptor);
            self.store(parent);
        }
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }

    fn insert_new_ball(&mut self, ball: OriginalParkBall) {
        let mut i = 0_usize;
        while i < 8 {
            if let Some(slot) = self.balls.get_mut(i) {
                if slot.is_none() {
                    *slot = Some(ball);
                    self.boundary(OriginalBoundaryCall::InsertBoxes(ball.id));
                    return;
                }
            }
            i = i.saturating_add(1);
        }
        self.scope_error = true;
    }

    pub fn set_ball_not_in_park_callback(&mut self, enabled: bool) {
        self.ball_not_in_park_callback_enabled = enabled;
    }

    /// Ball::AddMiniBall allocates/inserts the descriptor first. Its non-free
    /// detached branch then reaches AddActualMiniball/RegisterBallNotInPark,
    /// without creating an in-park collision ball. The reviewed Python handler
    /// only sets self.called. Allocation and actual Python dispatch are retained
    /// boundary requirements; this does not prove the interpreter.
    pub fn add_detached_mini_ball(
        &mut self,
        ball: &mut OriginalParkBall,
        position: Vec3,
        radius: f32,
    ) -> Result<(), OriginalParkError> {
        if ball.in_park {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        if !ball.mini_balls.push(OriginalMiniBall {
            id: 0,
            position,
            radius,
        }) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        if !ball.is_free && self.ball_not_in_park_callback_enabled {
            self.boundary(OriginalBoundaryCall::BallNotInParkCallback(ball.id));
            self.ball_not_in_park_callback_called = true;
        }
        Ok(())
    }

    fn update_time_factor(ball: &mut OriginalParkBall, friction: f64) {
        let mass = ball.mass * f64::from(ball.agility);
        ball.time_factor = if mass <= 0.0 {
            0.0
        } else {
            (-friction / mass).exp()
        };
    }

    fn set_mode(&mut self, id: i64, mode: OriginalMode) {
        let mut b = self.retained(id);
        if b.mode != mode {
            if b.mode == OriginalMode::Warp && mode != OriginalMode::Warp {
                self.boundary(OriginalBoundaryCall::ExitWarp(id, false));
            }
            self.boundary(OriginalBoundaryCall::ModeChange(id, b.mode, mode));
            if self.warp_finished_add_handler == Some(id)
                && b.mode == OriginalMode::Warp
                && mode == OriginalMode::Stop
            {
                self.warp_done = true; // original assignment precedes the failing AddBall
                self.mode_callback_add_attempted = true;
                let result = self.add_original_ball_checked(OriginalBallInit {
                    id: -1,
                    mass: 1.0,
                    radius: 1.0,
                    max_velocity: 1.0,
                    is_free: true,
                    is_global: false,
                    is_massive: true,
                    is_interactive: true,
                    is_space_junk: false,
                    position: Vec3::ZERO,
                    velocity: Vec3::ZERO,
                    agility: 1.0,
                    speed_fraction: 1.0,
                });
                match result {
                    Ok(created) => self.ball_added_during_evolve = Some(created),
                    Err(OriginalParkError::RuntimeError) => {
                        // Callback assignment never completes. SendEvent
                        // reports false and Ball::SetMode invokes PyOS->PyError;
                        // Evolve continues. Native exception/logging remains
                        // separately required, not a fabricated successful None.
                        self.mode_callback_error = Some(OriginalParkError::RuntimeError);
                    }
                    Err(_) => self.scope_error = true,
                }
            }
        }
        b.mode = mode;
        self.store(b);
    }

    pub fn stop(&mut self, id: i64) {
        if !self.contains(id) || self.retained(id).mode == OriginalMode::Stop {
            return;
        }
        self.stop_pointer(id);
    }

    fn stop_pointer(&mut self, id: i64) {
        let mut b = self.retained(id);
        if b.mode.follows() {
            if self.is_master && b.mode == OriginalMode::Formation {
                let mut dest = self.retained(b.follow_id);
                let count = if dest.formation_id == 2 { 1 } else { 4 };
                dest.formation_slots =
                    free_formation_slot(dest.formation_slots, Some(count), b.effect_stamp);
                self.store(dest);
            }
            b.effect_stamp = 0;
            b.follow_id = 0;
            b.owner_id = 0;
            b.follow_range = 0.0;
            self.store(b);
        }
        self.set_mode(id, OriginalMode::Stop);
    }

    fn stop_all_followers(&mut self, id: i64) {
        let mut i = 0;
        while i < 8 {
            if let Some(Some(ball)) = self.balls.get(i).copied() {
                if ball.in_park && ball.mode.follows() && ball.follow_id == id {
                    // Source StopAllFollowers sends missiles and interactive
                    // orbiters along their current velocity. Other followers
                    // stop. A missile also loses its massive flag first.
                    if ball.mode == OriginalMode::Missile
                        || (ball.is_interactive && ball.mode == OriginalMode::Orbit)
                    {
                        if ball.mode == OriginalMode::Missile {
                            self.write_attribute(ball.id, |b| b.is_massive = false);
                        }
                        self.goto_direction(ball.id, ball.velocity);
                    } else {
                        self.stop_pointer(ball.id);
                    }
                }
            }
            i += 1;
        }
    }

    pub fn remove_ball(&mut self, id: i64, delay: i32) {
        if !self.contains(id) {
            return;
        }
        self.stop_pointer(id);
        self.stop_all_followers(id);
        let mut b = self.retained(id);
        b.is_moribund = true;
        self.store(b);
        if !b.is_free {
            if self.remove_stream_children(id).is_err() {
                self.scope_error = true;
            }
            b = self.retained(id);
        }
        if self.evolving || delay > 0 {
            b.is_massive = false;
            if let Some(stamp) = self.current_time.checked_add(i64::from(delay)) {
                b.effect_stamp = stamp;
            } else {
                self.scope_error = true;
            }
            self.store(b);
            return;
        }
        self.boundary(OriginalBoundaryCall::FinalCleanup(id));
        self.boundary(OriginalBoundaryCall::DeleteBoxes(id));
        b.old_bubble = b.new_bubble;
        b.new_bubble = -1;
        b.in_park = false;
        self.store(b);
        self.boundary(OriginalBoundaryCall::AddToBubble(id));
        if let Some(projection) = &mut self.query_bubbles {
            if projection.dictionary.transition(b).is_err() {
                self.scope_error = true;
            }
        }
        if b.is_free {
            self.update_free_registry(id, false);
        }
        if b.has_proximity {
            self.boundary(OriginalBoundaryCall::ProximityMembership(id, false));
        }
    }

    pub fn clear_all(&mut self) {
        self.record_boundary(OriginalBoundaryCall::ClearAll);
        self.balls = [None; 8];
        self.capsules = BoundedList::new();
        self.oriented_boxes = BoundedList::new();
        self.capsule_boxes = BoundedList::new();
        self.oriented_box_boxes = BoundedList::new();
        self.local_counter = -1;
        self.local_high_counter = -(1_i64 << 30);
        self.query_bubbles = None;
        self.ball_boxes = [None; 8];
        self.free_ids = BoundedList::new();
        self.proximity_ids = BoundedList::new();
        self.box_objects = BoundedList::new();
        if self.iterative_collision {
            self.collision_state =
                crate::original_iterative_collision::OriginalIterativeState::default();
        }
    }

    /// Full spherical-cell construction and seed filling for InitializeBubbles.
    /// Static-object topology and master incremental insertion are separately
    /// unimplemented scopes. Reject them, rather than assume a bubble label.
    pub fn initialize_query_bubbles(&mut self, src: i64) -> Result<(), OriginalParkError> {
        if src != 0 {
            // Thunkers.cpp:1123 calls UpdateBallBubble, not InitializeBubbles,
            // for a nonzero ID. That method neither changes oldBubble nor
            // populates the Python bubbles dictionary.
            if !self.contains(src) {
                return Ok(());
            }
            let mut ball = self.retained(src);
            let mut high = -1_i64;
            let mut i = 0;
            while i < 8 {
                if let Some(Some(other)) = self.balls.get(i) {
                    if other.in_park {
                        high = high.max(other.new_bubble);
                        if other.id != src
                            && other.new_bubble != -1
                            && ball.position.sub(other.position).norm_squared()
                                < TOP_WIDTH * TOP_WIDTH
                        {
                            ball.new_bubble = other.new_bubble;
                            break;
                        }
                    }
                }
                i += 1;
            }
            if ball.new_bubble == -1 {
                ball.new_bubble = high + 1;
            }
            self.store(ball);
            return Ok(());
        }
        if self.is_master || !self.capsules.is_empty() || !self.oriented_boxes.is_empty() {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut projection = OriginalBubbleProjection::default();
        let mut primary = [None; 8];
        let mut i = 0;
        while i < 8 {
            if let Some(Some(ball)) = self.balls.get(i) {
                if ball.in_park {
                    let inflated = if ball.is_free {
                        // Ball.cpp:838; dt=1000*0.001 at the original defaults.
                        let m = ball.mass * f64::from(ball.agility);
                        let a = 1_000_000.0
                            * f64::from(ball.speed_fraction)
                            * f64::from(ball.max_velocity)
                            / m;
                        let v = ball.velocity.length();
                        f64::from(ball.radius)
                            + (m * a - (m * a - v * 1_000_000.0) * ball.time_factor) / 1_000_000.0
                    } else {
                        f64::from(ball.radius)
                    };
                    let key = projection.insert_sphere(ball.position, ball.radius, inflated);
                    if key.is_none() {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    }
                    if let Some(slot) = primary.get_mut(i) {
                        *slot = key;
                    }
                }
            }
            i += 1;
        }
        let mut next_bubble = 0_i64;
        i = 0;
        while i < 8 {
            if let Some(Some(mut ball)) = self.balls.get(i).copied() {
                if ball.in_park {
                    let key = primary.get(i).copied().flatten();
                    let Some(key) = key else {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    };
                    let Some(mut bubble) = projection.bubble_for(key) else {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    };
                    if bubble == -1 {
                        if !projection.label_component(key, next_bubble) {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                        bubble = next_bubble;
                        next_bubble += 1;
                    }
                    ball.old_bubble = ball.new_bubble;
                    ball.new_bubble = bubble;
                    // Initializing cleared every old Python bubble dictionary.
                    // AddToBubble inserts +1 for each encountered ball. A
                    // repeated initialization with identical old/new bubble
                    // deletes that just-added entry in its old-bubble phase.
                    projection.dictionary.transition(ball)?;
                    self.store(ball);
                }
            }
            i += 1;
        }
        projection.next_bubble = next_bubble;
        self.query_bubbles = Some(projection);
        Ok(())
    }

    /// Source InsertInBoxes for a newly constructed master spherical ball.
    /// This explicit adapter completes the spatial boundary of add_ball for
    /// these fixtures. Replacement, moving-box deletion/conflict resolution
    /// and static collidables remain unsupported scopes, with checked errors.
    pub fn insert_new_query_ball(&mut self, id: i64) -> Result<(), OriginalParkError> {
        if !self.is_master
            || !self.contains(id)
            || !self.capsules.is_empty()
            || !self.oriented_boxes.is_empty()
        {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut b = self.retained(id);
        if b.new_bubble != -1 || b.old_bubble != -1 {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let inflated = if b.is_free {
            let m = b.mass * f64::from(b.agility);
            let a = 1_000_000.0 * f64::from(b.speed_fraction) * f64::from(b.max_velocity) / m;
            let v = b.velocity.length();
            f64::from(b.radius) + (m * a - (m * a - v * 1_000_000.0) * b.time_factor) / 1_000_000.0
        } else {
            f64::from(b.radius)
        };
        let projection = self
            .query_bubbles
            .get_or_insert_with(OriginalBubbleProjection::default);
        let Some(bubble) = projection.insert_new_master_sphere(b.position, b.radius, inflated)
        else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        b.old_bubble = b.new_bubble;
        b.new_bubble = bubble;
        projection.dictionary.transition(b)?;
        self.store(b);
        Ok(())
    }

    fn bubble_live_balls(&self) -> Result<BoundedList<OriginalParkBall, 8>, OriginalParkError> {
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < 8 {
            if let Some(Some(b)) = self.balls.get(i).copied() {
                if b.in_park && !result.push(b) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    pub fn bubble_user_balls(&self) -> Result<crate::OriginalBubbleIds, OriginalParkError> {
        if let Some(p) = &self.query_bubbles {
            p.dictionary.user_balls()
        } else {
            Ok(BoundedList::new())
        }
    }

    pub fn bubble_dictionary_entries(&self) -> BoundedList<crate::OriginalBubbleEntry, 16> {
        self.query_bubbles
            .as_ref()
            .map(|p| p.dictionary.entries)
            .unwrap_or_default()
    }
    #[must_use]
    pub fn bubble_interactive_entries(&self) -> BoundedList<(i64, i64), 16> {
        self.query_bubbles
            .as_ref()
            .map_or_else(BoundedList::new, |p| p.dictionary.interactives)
    }

    pub fn bubble_additions_and_deletions(
        &mut self,
        users: &BoundedList<Option<i64>, 8>,
    ) -> Result<crate::OriginalBubbleUpdates, OriginalParkError> {
        // Ballpark.cpp's constructor creates the Python bubbles dictionary,
        // even before any partition cell is inserted. None here means an
        // empty spatial projection, not a missing native dictionary.
        let balls = self.bubble_live_balls()?;
        let result = if let Some(projection) = &self.query_bubbles {
            projection.dictionary.updates(&balls, users)?
        } else {
            crate::original_bubbles::OriginalBubbleDictionary::default().updates(&balls, users)?
        };
        let mut i = 0;
        while i < users.len() {
            if let Some(Some(id)) = users.get(i) {
                if self.contains(id) {
                    let mut b = self.retained(id);
                    b.old_bubble = b.new_bubble;
                    self.store(b);
                }
            }
            i += 1;
        }
        self.bubble_transitions = result.transitions;
        Ok(result)
    }

    pub fn copy_query_bubbles(&mut self) -> Result<(), OriginalParkError> {
        if let Some(p) = &mut self.query_bubbles {
            p.dictionary.copy_bubbles()?;
        }
        Ok(())
    }

    /// Source PyGetBubbleAtCoordinates: absent top cell is None. The current
    /// scope retains the initialized spherical partition, not static geometry.
    pub fn query_bubble_at(&self, position: Vec3) -> Result<Option<i64>, OriginalParkError> {
        if !self.capsules.is_empty() || !self.oriented_boxes.is_empty() || self.is_master {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let Some(key) = primary_key(position, 0) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        if self.query_bubbles.is_none() {
            let cells = self.spherical_cells_at_level(0)?;
            let mut i = 0;
            while i < cells.len() {
                if cells.get(i) == Some(key) {
                    return Ok(Some(-1));
                }
                i += 1;
            }
            return Ok(None);
        }
        Ok(self.query_bubbles.as_ref().and_then(|p| p.bubble_for(key)))
    }

    fn spherical_cells_at_level(
        &self,
        level: u32,
    ) -> Result<BoundedList<CellKey, 64>, OriginalParkError> {
        if level >= 8 {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < 8 {
            if self
                .balls
                .get(i)
                .is_some_and(|b| b.is_some_and(|b| b.in_park))
            {
                let Some(keys) = self.ball_boxes.get(i).copied().flatten() else {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                };
                if !add_ancestor_cells(&mut result, &keys.keys, keys.level, level) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        i = 0;
        while i < self.capsules.len() {
            if let Some(keys) = self.capsule_boxes.get(i) {
                if !add_ancestor_cells(&mut result, &keys.keys, keys.level, level) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        i = 0;
        while i < self.oriented_boxes.len() {
            if let Some(keys) = self.oriented_box_boxes.get(i) {
                if !add_ancestor_cells(&mut result, &keys.keys, keys.level, level) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    pub fn get_static_collidable_box(
        &self,
        id: i64,
    ) -> Result<Option<(f64, (f64, f64, f64))>, OriginalParkError> {
        let mut i = 0;
        while i < self.capsules.len() {
            if let Some(object) = self.capsules.get(i) {
                if object.id == id {
                    let Some(keys) = self.capsule_boxes.get(i) else {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    };
                    return box_snapshot(keys.primary, keys.level)
                        .map(Some)
                        .ok_or(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        i = 0;
        while i < self.oriented_boxes.len() {
            if let Some(object) = self.oriented_boxes.get(i) {
                if object.id == id {
                    let Some(keys) = self.oriented_box_boxes.get(i) else {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    };
                    return box_snapshot(keys.primary, keys.level)
                        .map(Some)
                        .ok_or(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(None)
    }

    pub fn get_ball_box(
        &self,
        id: i64,
    ) -> Result<Option<(f64, (f64, f64, f64))>, OriginalParkError> {
        let Some(index) = self.index(id, true) else {
            return Ok(None);
        };
        let Some(keys) = self.ball_boxes.get(index).copied().flatten() else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let Some(key) = keys.keys.get(0) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        box_snapshot(key, keys.level)
            .map(Some)
            .ok_or(OriginalParkError::OutsideDeclaredScope)
    }

    pub fn get_grid_center(
        &self,
        level: i32,
        position: Vec3,
    ) -> Result<(f64, f64, f64), OriginalParkError> {
        if !(0..8).contains(&level) {
            return Err(OriginalParkError::TypeError);
        }
        grid_center(position, level as u32).ok_or(OriginalParkError::OutsideDeclaredScope)
    }

    /// Exact public list only when there is at most one cell. Native hash-map
    /// iteration for larger lists requires an explicit unordered observation
    /// adapter (set/sorted callers), not an invented native enumeration order.
    pub fn get_single_active_box(
        &self,
        level: i32,
    ) -> Result<Option<(f64, BoundedList<Vec3, 64>)>, OriginalParkError> {
        self.get_active_box_view(level, false)
    }

    /// Canonical membership view is allowed only for declared set/sorted
    /// observations. It is not a claim about native unordered-map list order.
    pub fn get_active_box_view(
        &self,
        level: i32,
        canonical: bool,
    ) -> Result<Option<(f64, BoundedList<Vec3, 64>)>, OriginalParkError> {
        if !(0..8).contains(&level) {
            return Ok(None);
        }
        let mut cells = self.spherical_cells_at_level(level as u32)?;
        if cells.is_empty() {
            return Ok(None);
        }
        if cells.len() > 1 && !canonical {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut i = 1;
        while i < cells.len() {
            let Some(key) = cells.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let mut j = i;
            while j > 0 {
                let Some(previous) = cells.get(j - 1) else {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                };
                if (previous.x, previous.y, previous.z) <= (key.x, key.y, key.z) {
                    break;
                }
                if !cells.set(j, previous) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                j -= 1;
            }
            if !cells.set(j, key) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            i += 1;
        }
        let mut points = BoundedList::new();
        i = 0;
        while i < cells.len() {
            let Some(key) = cells.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            let Some(low) = box_low(key, level as u32) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if !points.push(low) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            i += 1;
        }
        let Some(width) = box_width(level as u32) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        Ok(Some((width, points)))
    }

    fn existing_top_cell(&self, position: Vec3) -> Result<Option<CellKey>, OriginalParkError> {
        let Some(key) = primary_key(position, 0) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let cells = self.spherical_cells_at_level(0)?;
        let mut i = 0;
        while i < cells.len() {
            if cells.get(i) == Some(key) {
                return Ok(Some(key));
            }
            i += 1;
        }
        Ok(None)
    }

    pub fn set_box_object(
        &mut self,
        position: Vec3,
        object: OriginalBoxObject,
    ) -> Result<bool, OriginalParkError> {
        let Some(key) = self.existing_top_cell(position)? else {
            return Ok(false);
        };
        let mut i = 0;
        while i < self.box_objects.len() {
            if self
                .box_objects
                .get(i)
                .is_some_and(|(existing, _)| existing == key)
            {
                if !self.box_objects.set(i, (key, object)) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                return Ok(true);
            }
            i += 1;
        }
        if !self.box_objects.push((key, object)) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        Ok(true)
    }

    pub fn get_box_object(
        &self,
        position: Vec3,
    ) -> Result<Option<OriginalBoxObject>, OriginalParkError> {
        let Some(key) = self.existing_top_cell(position)? else {
            return Ok(None);
        };
        let mut i = 0;
        while i < self.box_objects.len() {
            if let Some((existing, value)) = self.box_objects.get(i) {
                if existing == key {
                    return Ok(Some(value));
                }
            }
            i += 1;
        }
        Ok(None)
    }

    fn query_candidates(
        &self,
        src: Option<i64>,
        position: Vec3,
    ) -> Result<BoundedList<OriginalParkBall, 8>, OriginalParkError> {
        if !self.capsules.is_empty() || !self.oriented_boxes.is_empty() || self.is_master {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        if src.is_some_and(|id| !self.contains(id)) {
            return Err(OriginalParkError::RuntimeError);
        }
        let bubble = if let Some(id) = src {
            self.retained(id).new_bubble
        } else {
            self.query_bubble_at(position)?.unwrap_or(-1)
        };
        let mut result = BoundedList::new();
        if let Some(projection) = &self.query_bubbles {
            let mut i = 0;
            while i < projection.dictionary.entries.len() {
                if let Some(member) = projection.dictionary.entries.get(i) {
                    if member.bubble == bubble && self.contains(member.ball) {
                        let ball = self.retained(member.ball);
                        // GetNextValidBallInBubble ignores the action flag,
                        // but excludes removed/moribund objects and the source.
                        if !ball.is_moribund && src != Some(ball.id) && !result.push(ball) {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                    }
                }
                i += 1;
            }
        }
        Ok(result)
    }

    pub fn query_range(
        &self,
        src: Option<i64>,
        position: Vec3,
        range: f64,
        include_cloaked: bool,
    ) -> Result<BoundedList<(f64, i64), 8>, OriginalParkError> {
        if src.is_some_and(|id| !self.contains(id)) {
            return Err(OriginalParkError::RuntimeError);
        }
        let center = src.map_or(position, |id| self.retained(id).position);
        let candidates = self.query_candidates(src, center)?;
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < candidates.len() {
            if let Some(ball) = candidates.get(i) {
                let distance = ball.position.sub(center).norm_squared();
                let radius = range + f64::from(ball.radius);
                if (ball.is_cloaked == 0 || include_cloaked)
                    && distance <= radius * radius
                    && !result.push((distance, ball.id))
                {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    pub fn query_triangle(
        &self,
        src: i64,
        u: Vec3,
        v: Vec3,
        range: f64,
    ) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        if !self.contains(src) {
            return Err(OriginalParkError::RuntimeError);
        }
        let center = self.retained(src).position;
        let triangle = Triangle {
            a: center,
            b: center.add(u),
            c: center.add(v),
        };
        let candidates = self.query_candidates(Some(src), center)?;
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < candidates.len() {
            if let Some(ball) = candidates.get(i) {
                let distance = ball
                    .position
                    .sub(triangle.closest_point(ball.position))
                    .norm_squared();
                let radius = range + f64::from(ball.radius);
                if distance <= radius * radius && !result.push(ball.id) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    pub fn query_capsule(
        &self,
        src: i64,
        segment: Vec3,
        radius: f64,
    ) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        if !self.contains(src) {
            return Err(OriginalParkError::RuntimeError);
        }
        let center = self.retained(src).position;
        let candidates = self.query_candidates(Some(src), center)?;
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < candidates.len() {
            if let Some(ball) = candidates.get(i) {
                let ap = ball.position.sub(center);
                let c1 = ap.dot(segment);
                let c2 = segment.dot(segment);
                let distance = if c1 <= 0.0 {
                    ap.norm_squared()
                } else if c2 <= c1 {
                    ball.position.sub(center.add(segment)).norm_squared()
                } else {
                    center
                        .add(segment.scale(c1 / c2))
                        .sub(ball.position)
                        .norm_squared()
                };
                let inflated = radius + f64::from(ball.radius);
                if distance <= inflated * inflated && !result.push(ball.id) {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
            }
            i += 1;
        }
        Ok(result)
    }

    /// Thunkers.cpp:1523..1612, including the rounded end and source strict
    /// infinite-cone test. Platform sin/cos and Python radians conversion keep
    /// their numerical correspondence requirements.
    pub fn query_cone(
        &self,
        src: i64,
        vector: Vec3,
        angle: f64,
    ) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        if !self.contains(src) {
            return Err(OriginalParkError::RuntimeError);
        }
        let center = self.retained(src).position;
        let height = vector.length();
        let direction = vector.normalize();
        let candidates = self.query_candidates(Some(src), center)?;
        let sine = angle.sin();
        let cosine = angle.cos();
        let reciprocal = if sine > 0.0 { 1.0 / sine } else { 0.0 };
        let sine2 = sine * sine;
        let cosine2 = cosine * cosine;
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < candidates.len() {
            if let Some(ball) = candidates.get(i) {
                let radius2 = f64::from(ball.radius * ball.radius);
                let max_distance = height + f64::from(ball.radius);
                if !(max_distance * max_distance < center.sub(ball.position).norm_squared()) {
                    let apex = center.sub(direction.scale(f64::from(ball.radius) * reciprocal));
                    let mut displacement = ball.position.sub(apex);
                    let mut distance2 = displacement.dot(displacement);
                    let mut projection = direction.dot(displacement);
                    if projection > 0.0 && projection * projection > distance2 * cosine2 {
                        displacement = ball.position.sub(center);
                        distance2 = displacement.dot(displacement);
                        projection = -direction.dot(displacement);
                        let outside_apex = projection > 0.0
                            && projection * projection >= distance2 * sine2
                            && distance2 > radius2;
                        if !outside_apex && !result.push(ball.id) {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                    }
                }
            }
            i += 1;
        }
        Ok(result)
    }

    /// The C RNG result is an external input with its actual range checked.
    /// No property of the generator's sequence is asserted by this projection.
    /// The Python wrapper returns None even if the native method returns -1.
    pub fn add_proximity_sensor(
        &mut self,
        id: i64,
        range: f32,
        period: f64,
        shuffle: i32,
        only_interactives: bool,
        random_draw: u32,
        rand_max: u32,
    ) -> Result<(), OriginalParkError> {
        if !self.contains(id) {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut ball = self.retained(id);
        if ball.radius + range < 0.0 {
            return Ok(());
        }
        ball.sensor.balls = BoundedList::new();
        ball.sensor.range = range;
        ball.sensor.period = period;
        ball.sensor.active = true;
        ball.sensor.only_interactives = only_interactives;
        if shuffle != 0 {
            if rand_max == 0 || random_draw > rand_max {
                self.scope_error = true;
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            ball.sensor.elapsed =
                0.0 + f64::from(random_draw) / f64::from(rand_max) * (period - 0.0);
        }
        ball.has_proximity =
            ball.tracking_range != 0.0 || ball.notification_range != 0.0 || ball.sensor.active;
        self.store(ball);
        self.boundary(OriginalBoundaryCall::ProximityMembership(
            id,
            ball.has_proximity,
        ));
        Ok(())
    }

    fn update_proximity_status(&mut self, id: i64) {
        let mut b = self.retained(id);
        b.has_proximity = b.tracking_range != 0.0 || b.notification_range != 0.0 || b.sensor.active;
        self.store(b);
        self.boundary(OriginalBoundaryCall::ProximityMembership(
            id,
            b.has_proximity,
        ));
    }

    pub fn set_notification_range(&mut self, id: i64, range: f64) {
        if !self.contains(id) || range < 0.0 {
            return;
        }
        let mut b = self.retained(id);
        b.notification_range = range;
        b.within_range = false;
        self.store(b);
        self.update_proximity_status(id);
    }

    pub fn set_target_tracking(&mut self, id: i64, target: i64, range: f64) {
        if !self.contains(id) {
            return;
        }
        if range <= 0.0 || target == 0 {
            let mut b = self.retained(id);
            b.tracking_id = None;
            b.tracking_range = 0.0;
            self.store(b);
            // The original returns here without UpdateProximityStatus.
            return;
        }
        if !self.contains(target) {
            return;
        }
        let mut b = self.retained(id);
        b.tracking_id = Some(target);
        b.tracking_range = range;
        b.within_tracking_range = false;
        self.store(b);
        self.update_proximity_status(id);
    }

    pub fn remove_proximity_sensor(&mut self, id: i64) {
        if !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.sensor.active = false;
        self.store(b);
        self.update_proximity_status(id);
    }

    fn emit_range_event(
        &mut self,
        b: &mut OriginalParkBall,
        tracking: bool,
        inside: bool,
    ) -> Result<(), OriginalParkError> {
        self.boundary(if tracking {
            OriginalBoundaryCall::TargetTrackingEvent(b.id, inside)
        } else {
            OriginalBoundaryCall::RangeEvent(b.id, inside)
        });
        if b.event_spy {
            let list = if tracking {
                &mut b.target_tracking_callback_args
            } else {
                &mut b.range_callback_args
            };
            if !list.push(i64::from(inside)) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
        }
        Ok(())
    }

    fn emit_proximity_event(
        &mut self,
        b: &mut OriginalParkBall,
        other: i64,
        inside: bool,
    ) -> Result<(), OriginalParkError> {
        self.boundary(OriginalBoundaryCall::ProximityEvent(b.id, other, inside));
        if b.event_spy && !b.proximity_callback_args.push((other, i64::from(inside))) {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        Ok(())
    }

    fn original_is_dead_bubble(&self, b: &OriginalParkBall) -> bool {
        // Source InDeadBubble. These projections have no keep-alive registry
        // writes; such inputs still need a separate representation obligation.
        if !self.is_master || b.mode == OriginalMode::Warp {
            return false;
        }
        let entries = self.bubble_interactive_entries();
        let mut i = 0;
        while i < entries.len() {
            if entries
                .get(i)
                .is_some_and(|(bubble, _)| bubble == b.new_bubble)
            {
                return false;
            }
            i += 1;
        }
        true
    }

    fn proximity_candidates(
        &self,
        id: i64,
        only_interactives: bool,
    ) -> Result<BoundedList<i64, 8>, OriginalParkError> {
        let mut result = BoundedList::new();
        let source = self.retained(id);
        if only_interactives {
            let entries = self.bubble_interactive_entries();
            let mut exists = false;
            let mut i = 0;
            while i < entries.len() {
                exists |= entries
                    .get(i)
                    .is_some_and(|(bubble, _)| bubble == source.new_bubble);
                i += 1;
            }
            // GetInteractiveCnt is always zero for a client park.
            if !self.is_master || !exists {
                return Ok(result);
            }
        }
        let Some(index) = self.index(id, true) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let Some(boxes) = self.ball_boxes.get(index).copied().flatten() else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        let mut i = 0;
        while i < 8 {
            if let Some(Some(other)) = self.balls.get(i) {
                let eligible = if only_interactives {
                    other.is_interactive
                } else {
                    (!self.is_master || other.new_bubble == source.new_bubble || other.is_global)
                        && (source.is_cloaked != 0 || other.is_free)
                };
                if other.in_park && other.id != id && !other.is_moribund && eligible {
                    let Some(other_boxes) = self.ball_boxes.get(i).copied().flatten() else {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    };
                    if spherical_candidates(boxes, other_boxes) && !result.push(other.id) {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    }
                }
            }
            i += 1;
        }
        Ok(result)
    }

    fn handle_proximities(&mut self) -> Result<(), OriginalParkError> {
        // Source iterates a copy of the registry. The exact fixture spies only
        // append event arguments; arbitrary callback mutation remains external.
        let registered = self.proximity_ids;
        let mut i = 0;
        while i < registered.len() {
            let Some(id) = registered.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if self.contains(id) {
                let mut b = self.retained(id);
                if !b.is_moribund && b.has_proximity {
                    let dead = self.original_is_dead_bubble(&b);
                    if dead && b.sensor.balls.is_empty() {
                        i += 1;
                        continue;
                    }
                    if dead {
                        b.sensor.elapsed = b.sensor.period;
                    }
                    let mut inside = false;
                    if b.notification_range != 0.0 {
                        if b.mode == OriginalMode::Goto {
                            let range = b.notification_range + f64::from(b.radius);
                            inside = b.position.sub(b.goto).norm_squared() < range * range;
                        } else if matches!(
                            b.mode,
                            OriginalMode::Follow | OriginalMode::Missile | OriginalMode::Orbit
                        ) {
                            if !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let other = self.retained(b.follow_id);
                            let distance = b.position.sub(other.position).norm_squared();
                            let high = f64::from(other.radius)
                                + f64::from(b.follow_range)
                                + b.notification_range
                                + f64::from(b.radius);
                            let mut low = high - 2.0 * b.notification_range;
                            if low < 0.0 {
                                low = 0.0;
                            }
                            inside = distance < high * high && distance > low * low;
                        }
                        if inside != b.within_range {
                            b.within_range = inside;
                            self.emit_range_event(&mut b, false, inside)?;
                        }
                    }
                    if b.tracking_range >= 0.00001 {
                        if let Some(target) = b.tracking_id {
                            if !self.contains(target) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let range = b.tracking_range + f64::from(b.radius);
                            inside = b
                                .position
                                .sub(self.retained(target).position)
                                .norm_squared()
                                < range * range;
                            if inside != b.within_tracking_range {
                                b.within_tracking_range = inside;
                                self.emit_range_event(&mut b, true, inside)?;
                            }
                        }
                    }
                    if b.sensor.active {
                        b.sensor.elapsed += 1.0;
                        if !(b.sensor.elapsed < b.sensor.period) {
                            b.sensor.elapsed = 0.0;
                            self.store(b);
                            let radius = b.radius;
                            let velocity = b.velocity;
                            self.write_attribute(id, |b| b.velocity = Vec3::ZERO);
                            self.set_radius(id, b.radius + b.sensor.range);
                            let candidates =
                                self.proximity_candidates(id, b.sensor.only_interactives);
                            // Restore both fields and perform the original
                            // second radius insertion before using candidates.
                            self.write_attribute(id, |b| b.velocity = velocity);
                            self.set_radius(id, radius);
                            let candidates = candidates?;
                            let old_members = b.sensor.balls;
                            let mut retained = BoundedList::new();
                            let mut j = 0;
                            while j < old_members.len() {
                                if let Some(other) = old_members.get(j) {
                                    let mut nearby = false;
                                    let mut k = 0;
                                    while k < candidates.len() {
                                        nearby |= candidates.get(k) == Some(other);
                                        k += 1;
                                    }
                                    if other == id || nearby {
                                        if !retained.push(other) {
                                            return Err(OriginalParkError::OutsideDeclaredScope);
                                        }
                                    } else {
                                        self.emit_proximity_event(&mut b, other, false)?;
                                    }
                                }
                                j += 1;
                            }
                            b.sensor.balls = retained;
                            j = 0;
                            while j < candidates.len() {
                                if let Some(other_id) = candidates.get(j) {
                                    let other = self.retained(other_id);
                                    let mut member = false;
                                    let mut k = 0;
                                    while k < b.sensor.balls.len() {
                                        member |= b.sensor.balls.get(k) == Some(other_id);
                                        k += 1;
                                    }
                                    let distance = b.position.sub(other.position).norm_squared();
                                    // All three source operands are float, so
                                    // addition rounds in binary32 before double.
                                    let range = f64::from(b.sensor.range + other.radius + b.radius);
                                    if distance > range * range {
                                        if member {
                                            let mut kept = BoundedList::new();
                                            k = 0;
                                            while k < b.sensor.balls.len() {
                                                if let Some(old) = b.sensor.balls.get(k) {
                                                    if old != other_id && !kept.push(old) {
                                                        return Err(
                                                            OriginalParkError::OutsideDeclaredScope,
                                                        );
                                                    }
                                                }
                                                k += 1;
                                            }
                                            b.sensor.balls = kept;
                                            self.emit_proximity_event(&mut b, other_id, false)?;
                                        }
                                    } else if !member {
                                        if !b.sensor.balls.push(other_id) {
                                            return Err(OriginalParkError::OutsideDeclaredScope);
                                        }
                                        self.emit_proximity_event(&mut b, other_id, true)?;
                                    }
                                }
                                j += 1;
                            }
                        }
                    }
                    if dead {
                        b.sensor.balls = BoundedList::new();
                    }
                    self.store(b);
                }
            }
            i += 1;
        }
        Ok(())
    }

    pub fn set_mass(&mut self, id: i64, mass: f64) {
        if mass <= 0.0 || !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.mass = mass;
        Self::update_time_factor(&mut b, self.friction);
        self.store(b);
    }

    /// Exact rejected-input branch, with its caller condition checked.
    pub fn reject_nonpositive_mass(&mut self, _id: i64, mass: f64) {
        if !(mass <= 0.0) {
            self.scope_error = true;
        }
    }
    pub fn set_radius(&mut self, id: i64, radius: f32) {
        if radius < 0.0 || !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.radius = radius;
        self.store(b);
        self.boundary(OriginalBoundaryCall::InsertBoxes(id));
    }
    pub fn set_max_speed(&mut self, id: i64, speed: f32) {
        if speed < 0.0 || !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.max_velocity = speed;
        self.store(b);
        self.boundary(OriginalBoundaryCall::InsertBoxes(id));
    }
    pub fn set_agility(&mut self, id: i64, agility: f32) {
        if agility <= 0.0 || !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.agility = agility;
        Self::update_time_factor(&mut b, self.friction);
        self.store(b);
    }

    /// Source branch for the original zero-mass agility examples. The complete
    /// incoming precision and original nonpositive rejection are retained.
    pub fn set_agility_nonpositive_time_factor(&mut self, id: i64, agility: f32) {
        if agility <= 0.0 || !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        if b.mass * f64::from(agility) <= 0.0 {
            b.agility = agility;
            b.time_factor = 0.0;
            self.store(b);
        } else {
            self.scope_error = true;
        }
    }
    pub fn set_speed_fraction(&mut self, id: i64, mut fraction: f32) {
        if !fraction.is_finite() || !self.contains(id) {
            return;
        }
        if fraction < 0.0 {
            fraction = 0.0;
        } else if fraction > 1.0 {
            fraction = 1.0;
        }
        let mut b = self.retained(id);
        b.speed_fraction = fraction;
        self.store(b);
    }
    pub fn set_position(&mut self, id: i64, position: Vec3) {
        if !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.position = position;
        b.old_position = position;
        self.store(b);
        if self.is_master && self.query_bubbles.is_some() {
            if self.reinsert_query_master_ball(id).is_err() {
                self.scope_error = true;
            }
        } else {
            self.boundary(OriginalBoundaryCall::InsertBoxes(id));
        }
    }
    pub fn set_velocity(&mut self, id: i64, velocity: Vec3) {
        if !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.velocity = velocity;
        b.old_velocity = velocity;
        self.calculate_yaw_pitch_roll(&mut b, true);
        self.store(b);
    }

    /// Ballpark.cpp:4819 normalizes binary32 quaternion components, then
    /// writes both rotations and clears both angular velocities. The catalog
    /// calls use axis quaternions; platform SIMD length/order for arbitrary
    /// quaternion inputs remains a numerical correspondence obligation.
    pub fn set_ball_rotation(&mut self, id: i64, components: [f64; 4]) {
        if !self.contains(id) {
            return;
        }
        let mut q = [
            components[0] as f32,
            components[1] as f32,
            components[2] as f32,
            components[3] as f32,
        ];
        let length = (((q[0] * q[0] + q[1] * q[1]) + q[2] * q[2]) + q[3] * q[3]).sqrt();
        if length > 1e-20 {
            let mut i = 0;
            while i < 4 {
                q[i] /= length;
                i += 1;
            }
            let mut b = self.retained(id);
            b.rotation = q;
            b.old_rotation = q;
            b.angular_velocity = [0.0; 3];
            b.old_angular_velocity = [0.0; 3];
            self.store(b);
        }
    }

    /// Original Ball::CalculateYawPitchRoll, including the old-style smoothing
    /// and roll state. The ClientBall snap quaternion conversion is retained as
    /// a native numerical boundary, with its actual narrowed input angles.
    fn calculate_yaw_pitch_roll(&mut self, b: &mut OriginalParkBall, snap: bool) {
        b.old_yaw = b.yaw;
        b.old_pitch = b.pitch;
        b.old_roll = b.roll;
        b.old_roll_speed = b.roll_speed;
        let mut mass_modifier = b.mass / 1_000_000.0;
        mass_modifier = if mass_modifier != 0.0 {
            100.0 / mass_modifier
        } else {
            100.0
        };
        let velocity2 = b.velocity.norm_squared();
        let mut direction = Vec3::ZERO;
        if velocity2 == 0.0 {
            direction = b.goto.sub(b.position);
        } else {
            if b.mode == OriginalMode::Warp {
                if b.effect_stamp < 0 {
                    direction = if b.collisions.is_empty() {
                        b.goto.sub(b.position)
                    } else {
                        b.velocity
                    };
                } else if mass_modifier < 1.0 && self.current_time != b.effect_stamp {
                    if let Some(ticks) = self.current_time.checked_sub(b.effect_stamp) {
                        let factor = (ticks as f64) * 2.0;
                        let inner = if factor < 15.0 { factor } else { 15.0 };
                        let modified = mass_modifier * inner;
                        mass_modifier = if modified < 1.0 { modified } else { 1.0 };
                    } else {
                        self.scope_error = true;
                    }
                }
            } else {
                direction = b.velocity;
            }
            if b.mode != OriginalMode::Missile && !snap {
                let offset = b.goto.sub(b.position).normalize();
                direction = direction.add(offset.scale(velocity2.sqrt() * 0.3));
            }
        }
        // Kani 0.68.0 has no supported foreign atan2. This is the same actual
        // portable implementation in ordinary and proof builds, with all its
        // checks retained. Original C atan2 correspondence is still required.
        b.yaw = libm::atan2(direction.x, direction.z);
        let r = direction.length();
        b.pitch = if r > 0.0 {
            let ratio = direction.y / r;
            let lower = if -1.0 < ratio { ratio } else { -1.0 };
            -(if lower < 1.0 { lower } else { 1.0 }).asin()
        } else {
            0.0
        };
        Self::modulo_2pi(&mut b.old_yaw, &mut b.yaw);
        Self::modulo_2pi(&mut b.old_pitch, &mut b.pitch);
        // The snap branch clears roll. The ClientBall quaternion conversion
        // remains in the recorded native numerical/Blue boundary.
        if snap {
            b.roll = 0.0;
            b.old_roll = 0.0;
            b.roll_speed = 0.0;
            b.old_roll_speed = 0.0;
            b.yaw_delta = 0.0;
            if b.client_ball {
                self.boundary(OriginalBoundaryCall::ClientQuaternionSnap(
                    b.id,
                    b.yaw as f32,
                    b.pitch as f32,
                ));
            }
            return;
        }
        let reciprocal = 1.0 / (1.0 + mass_modifier * 0.5);
        b.yaw = (b.old_yaw + mass_modifier * 0.5 * b.yaw) * reciprocal;
        b.pitch = (b.old_pitch + mass_modifier * 0.5 * b.pitch) * reciprocal;
        b.yaw_delta = (b.yaw - b.old_yaw).sin().asin();
        b.roll_speed += b.yaw_delta * -3.0;
        b.roll_speed += b.old_roll_speed * -0.9 * 1.0;
        let half_pi = 1.57079632679489655;
        let upper_speed = if b.roll_speed < half_pi {
            b.roll_speed
        } else {
            half_pi
        };
        b.roll_speed = if -half_pi < upper_speed {
            upper_speed
        } else {
            -half_pi
        };
        b.roll += b.roll_speed * 1.0 * 1.0;
        b.roll += b.old_roll * -0.5 * 1.0;
        let upper_roll = if b.roll < half_pi { b.roll } else { half_pi };
        b.roll = if -half_pi < upper_roll {
            upper_roll
        } else {
            -half_pi
        };
    }

    /// Complete linear step for an empty/client park with at most one body and
    /// no static collidables, sensors, dynamical orientation or iterative mode.
    /// The caller proves these scope checks; no successful outcome is assumed.
    /// Partition::GetNearbyBalls excludes self, so this scope has no collisions.
    /// Native partition/Blue lifetime correspondence is still required.
    pub fn evolve_single_body_linear(&mut self) -> Result<(), OriginalParkError> {
        if self.count() > 1
            || self.is_master
            || self.dynamical_orientation
            || self.iterative_collision
            || !self.capsules.is_empty()
            || !self.oriented_boxes.is_empty()
        {
            self.scope_error = true;
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.evolving = true;
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(mut b)) = self.balls.get(i).copied() {
                if b.in_park
                    && (b.sensor.active || b.tracking_range != 0.0 || b.notification_range != 0.0)
                {
                    self.scope_error = true;
                    self.evolving = false;
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                if b.in_park && self.in_free_registry(b.id) && !b.is_moribund {
                    b.last_g = Vec3::ZERO;
                    b.last_c = Vec3::ZERO;
                    b.collisions = BoundedList::new();
                    match b.mode {
                        OriginalMode::Goto => {
                            b.last_g = original_goto_thrust(&b, b.goto, 1_000_000.0, 1.0, false)
                        }
                        OriginalMode::Stop => b.velocity = original_stop_velocity(b.velocity),
                        _ => {
                            self.scope_error = true;
                            self.evolving = false;
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                    }
                    if b.is_massive {
                        b.last_collision = -1.0;
                    }
                    let old_position = b.position;
                    let old_velocity = b.velocity;
                    let (p, v) = original_integrate_tick(
                        b.position,
                        b.velocity,
                        b.last_g.add(b.last_c),
                        b.mass * f64::from(b.agility),
                        1_000_000.0,
                        b.time_factor,
                    );
                    b.position = p;
                    b.velocity = v;
                    b.old_position = old_position;
                    b.old_velocity = old_velocity;
                    self.calculate_yaw_pitch_roll(&mut b, false);
                    self.store(b);
                    if !(b.position.sub(b.old_position).norm_squared() < 0.1) {
                        self.boundary(OriginalBoundaryCall::InsertBoxes(b.id));
                    }
                }
            }
            i = i.saturating_add(1);
        }
        if let Some(next) = self.current_time.checked_add(1) {
            self.current_time = next;
        } else {
            self.scope_error = true;
        }
        self.evolving = false;
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }

    /// Non-iterative catalog evolution for at most eight retained spherical
    /// bodies. All behaviour forces are calculated before either integration;
    /// integration is calculated for both before the source's swap pass.
    /// Cached partition keys mirror source InsertBoxes calls. Master fixtures
    /// retain interactive membership and the source's cached dead-bubble skip.
    /// One capsule or oriented box and proximity sensors are supported.
    /// Keep-alive registry writes and unsupported movement modes remain outside
    /// this representation's input domain.
    pub fn evolve_two_spherical_bodies(&mut self) -> Result<(), OriginalParkError> {
        if self.dynamical_orientation
            || (!self.iterative_collision && self.capsules.len() + self.oriented_boxes.len() > 1)
        {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        self.evolving = true;
        let outcome = self.evolve_two_spherical_bodies_inner();
        self.evolving = false;
        outcome
    }

    fn calculate_original_iterative_collisions(
        &mut self,
        dead: [bool; 8],
    ) -> Result<(), OriginalParkError> {
        use crate::original_iterative_collision::StaticCollider;
        let mut free = BoundedList::<usize, 8>::new();
        let mut neighbors = [BoundedList::<usize, 8>::new(); 8];
        let mut static_indices = [BoundedList::<usize, 16>::new(); 8];
        let mut statics = BoundedList::<StaticCollider, 16>::new();
        let mut j = 0;
        while j < self.capsules.len() {
            if !statics.push(StaticCollider::Capsule(
                self.capsules
                    .get(j)
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?,
            )) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            j += 1;
        }
        j = 0;
        while j < self.oriented_boxes.len() {
            if !statics.push(StaticCollider::Box(
                self.oriented_boxes
                    .get(j)
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?,
            )) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            j += 1;
        }
        let mut i = 0;
        while i < self.free_ids.len() {
            let id = self
                .free_ids
                .get(i)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            let index = self
                .index(id, true)
                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
            if !free.push(index) {
                return Err(OriginalParkError::OutsideDeclaredScope);
            }
            let b = self.retained(id);
            let boxes = self.ball_boxes[index].ok_or(OriginalParkError::OutsideDeclaredScope)?;
            j = 0;
            while j < 8 {
                if let Some(other) = self.balls[j] {
                    let direct = b.mode == OriginalMode::Missile && b.owner_id >= 0;
                    let candidate = if direct {
                        other.id == b.follow_id && other.new_bubble == b.new_bubble
                    } else {
                        let filter = other.is_massive
                            && other.is_cloaked == 0
                            && (b.mode == OriginalMode::Missile
                                || other.mode != OriginalMode::Missile)
                            && (!self.is_master
                                || other.new_bubble == b.new_bubble
                                || other.is_global)
                            && !(other.mode == OriginalMode::Field
                                && (b.harmonic == -2
                                    || other.harmonic != -1 && other.harmonic == b.harmonic
                                    || other.corporation_id != -1
                                        && other.corporation_id == b.corporation_id
                                    || other.alliance_id != -1
                                        && other.alliance_id == b.alliance_id));
                        filter
                            && self.ball_boxes[j]
                                .is_some_and(|keys| spherical_candidates(boxes, keys))
                    };
                    if other.in_park && other.id != id && !other.is_moribund && candidate {
                        let mut place = 0;
                        while place < neighbors[index].len() {
                            let previous = neighbors[index]
                                .get(place)
                                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                            let ball = self.balls[previous]
                                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                            if ball.id > other.id {
                                break;
                            }
                            place += 1;
                        }
                        if !neighbors[index].insert(place, j) {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                    }
                }
                j += 1;
            }
            if b.mode != OriginalMode::Missile || b.owner_id < 0 {
                j = 0;
                while j < statics.len() {
                    let object = statics
                        .get(j)
                        .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                    let key = match object {
                        StaticCollider::Capsule(_) => self.capsule_boxes.get(j),
                        StaticCollider::Box(_) => {
                            self.oriented_box_boxes.get(j - self.capsules.len())
                        }
                    }
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                    if footprints_touch(&boxes.keys, boxes.level, &key.owned_keys, key.level) {
                        let mut place = 0;
                        while place < static_indices[index].len() {
                            let previous = static_indices[index]
                                .get(place)
                                .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                            if statics
                                .get(previous)
                                .ok_or(OriginalParkError::OutsideDeclaredScope)?
                                .id()
                                > object.id()
                            {
                                break;
                            }
                            place += 1;
                        }
                        if !static_indices[index].insert(place, j) {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        let mut state = core::mem::take(&mut self.collision_state);
        let result = state.calculate(
            &mut self.balls,
            &free,
            dead,
            self.friction,
            &neighbors,
            &static_indices,
            &statics,
        );
        self.collision_state = state;
        result
    }

    fn evolve_two_spherical_bodies_inner(&mut self) -> Result<(), OriginalParkError> {
        let mut dead_bubbles = [false; 8];
        let mut i = 0;
        while i < self.free_ids.len() {
            let Some(id) = self.free_ids.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if self.contains(id) {
                let mut b = self.retained(id);
                if !b.is_moribund {
                    if self.original_is_dead_bubble(&b) {
                        let index = self
                            .index(id, true)
                            .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                        dead_bubbles[index] = true;
                        i += 1;
                        continue;
                    }
                    b.last_g = Vec3::ZERO;
                    b.last_c = Vec3::ZERO;
                    b.collisions = BoundedList::new();
                    match b.mode {
                        OriginalMode::Goto => {
                            b.last_g = original_goto_thrust(&b, b.goto, self.friction, 1.0, false)
                        }
                        OriginalMode::Stop => b.velocity = original_stop_velocity(b.velocity),
                        OriginalMode::Warp => {
                            if b.effect_stamp < 0 {
                                if crate::original_warp::original_aligned_for_warp(&b) {
                                    self.boundary(OriginalBoundaryCall::ActivateWarp(
                                        b.id,
                                        self.current_time,
                                    ));
                                    self.real_warp(&mut b)?;
                                } else {
                                    let Some(stamp) = b.effect_stamp.checked_sub(1) else {
                                        return Err(OriginalParkError::OutsideDeclaredScope);
                                    };
                                    b.effect_stamp = stamp;
                                    b.last_g =
                                        original_goto_thrust(&b, b.goto, 1_000_000.0, 1.0, false);
                                }
                            }
                        }
                        OriginalMode::Follow => {
                            if !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let other = self.retained(b.follow_id);
                            b.last_g = crate::original_follow_thrust(&mut b, &other);
                        }
                        OriginalMode::Missile => {
                            if !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let other = self.retained(b.follow_id);
                            b.last_g = crate::original_motion::original_missile_thrust(
                                &mut b,
                                &other,
                                self.current_time,
                            )?;
                        }
                        OriginalMode::Formation => {
                            if !self.formations_loaded || !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let other = self.retained(b.follow_id);
                            b.last_g = crate::original_rotation::original_formation_thrust(
                                &mut b, &other,
                            )?;
                        }
                        OriginalMode::Orbit => {
                            if !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let other = self.retained(b.follow_id);
                            b.last_g = if self.use_new_orbit {
                                crate::original_new_orbit_thrust(&mut b, &other, self.current_time)?
                            } else {
                                crate::original_old_orbit_thrust(&mut b, &other, self.current_time)?
                            };
                        }
                        _ => return Err(OriginalParkError::OutsideDeclaredScope),
                    }
                    self.store(b);
                }
            }
            i += 1;
        }
        if self.iterative_collision {
            self.calculate_original_iterative_collisions(dead_bubbles)?;
        }
        i = 0;
        while i < self.free_ids.len() {
            let Some(id) = self.free_ids.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if self.contains(id) {
                let mut b = self.retained(id);
                let index = self
                    .index(id, true)
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                if !b.is_moribund && !dead_bubbles[index] {
                    if !self.iterative_collision && b.is_massive {
                        b.last_collision = -1.0;
                        let Some(index) = self.index(id, true) else {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        };
                        let Some(boxes) = self.ball_boxes.get(index).copied().flatten() else {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        };
                        if b.mode == OriginalMode::Missile && b.owner_id >= 0 {
                            if !self.contains(b.follow_id) {
                                return Err(OriginalParkError::OutsideDeclaredScope);
                            }
                            let mut other = self.retained(b.follow_id);
                            if b.new_bubble == other.new_bubble {
                                crate::original_collision::sphere_potential(&mut b, &mut other, 0)?;
                            }
                        }
                        let mut j = 0;
                        while j < 8 && !(b.mode == OriginalMode::Missile && b.owner_id >= 0) {
                            if let Some(Some(mut other)) = self.balls.get(j).copied() {
                                if other.in_park
                                    && other.id != id
                                    && !other.is_moribund
                                    && other.is_massive
                                    && other.is_cloaked == 0
                                    && (b.mode == OriginalMode::Missile
                                        || other.mode != OriginalMode::Missile)
                                    && (!self.is_master
                                        || other.new_bubble == b.new_bubble
                                        || other.is_global)
                                    && !(other.is_space_junk && !b.is_space_junk)
                                    && !(other.mode == OriginalMode::Mushroom
                                        && b.id == other.owner_id)
                                    && !(b.mode == OriginalMode::Missile
                                        && (other.id == b.owner_id
                                            || (other.mode == OriginalMode::Miniball
                                                && other.owner_id == b.owner_id)))
                                    && !(other.mode == OriginalMode::Missile
                                        && (b.id == other.owner_id || b.id == other.follow_id))
                                    && !(other.mode == OriginalMode::Field
                                        && (b.harmonic == -2
                                            || (other.harmonic != -1
                                                && other.harmonic == b.harmonic)
                                            || (other.corporation_id != -1
                                                && other.corporation_id == b.corporation_id)
                                            || (other.alliance_id != -1
                                                && other.alliance_id == b.alliance_id)))
                                {
                                    let Some(other_boxes) =
                                        self.ball_boxes.get(j).copied().flatten()
                                    else {
                                        return Err(OriginalParkError::OutsideDeclaredScope);
                                    };
                                    if spherical_candidates(boxes, other_boxes) {
                                        crate::original_collision::sphere_potential(
                                            &mut b, &mut other, 0,
                                        )?;
                                    }
                                }
                            }
                            j += 1;
                        }
                        let mut j = 0;
                        while j < self.capsules.len()
                            && b.mode != OriginalMode::Missile
                            && b.mode != OriginalMode::Mushroom
                        {
                            if let Some(object) = self.capsules.get(j) {
                                let Some(keys) = self.capsule_boxes.get(j) else {
                                    return Err(OriginalParkError::OutsideDeclaredScope);
                                };
                                if footprints_touch(
                                    &boxes.keys,
                                    boxes.level,
                                    &keys.owned_keys,
                                    keys.level,
                                ) {
                                    crate::original_collision::capsule_collision(&mut b, &object)?;
                                }
                            }
                            j += 1;
                        }
                        j = 0;
                        while j < self.oriented_boxes.len()
                            && b.mode != OriginalMode::Missile
                            && b.mode != OriginalMode::Mushroom
                        {
                            if let Some(object) = self.oriented_boxes.get(j) {
                                let Some(keys) = self.oriented_box_boxes.get(j) else {
                                    return Err(OriginalParkError::OutsideDeclaredScope);
                                };
                                if footprints_touch(
                                    &boxes.keys,
                                    boxes.level,
                                    &keys.owned_keys,
                                    keys.level,
                                ) {
                                    crate::original_collision::box_collision(&mut b, &object)?;
                                }
                            }
                            j += 1;
                        }
                    }
                    let (p, v) = if b.mode == OriginalMode::Warp && b.effect_stamp >= 0 {
                        let Some(elapsed) = self.current_time.checked_sub(b.effect_stamp) else {
                            return Err(OriginalParkError::OutsideDeclaredScope);
                        };
                        let (p, v, stop) =
                            crate::original_warp::original_warp_distance(&b, elapsed as f64);
                        if stop {
                            if !self.is_master {
                                self.boundary(OriginalBoundaryCall::DeactivateWarp(
                                    b.id,
                                    self.current_time,
                                ));
                            }
                            b.is_massive = true;
                            b.sensor.active = true;
                            self.store(b);
                            self.stop_pointer(id);
                            b = self.retained(id);
                            b.position = p;
                            b.velocity = v;
                            self.store(b);
                            if b.velocity.norm_squared() < 0.1 {
                                self.boundary(OriginalBoundaryCall::InsertBoxes(id));
                            }
                            original_integrate_tick(
                                p,
                                v,
                                b.last_g.add(b.last_c),
                                b.mass * f64::from(b.agility),
                                1_000_000.0,
                                b.time_factor,
                            )
                        } else {
                            (p, v)
                        }
                    } else if let Some(properties) = self
                        .collision_state
                        .properties
                        .get(index)
                        .copied()
                        .flatten()
                    {
                        properties.position(b, self.friction, 1.0)?
                    } else {
                        original_integrate_tick(
                            b.position,
                            b.velocity,
                            b.last_g.add(b.last_c),
                            b.mass * f64::from(b.agility),
                            self.friction,
                            b.time_factor,
                        )
                    };
                    // Temporary results go into oldPos/oldVel until the swap
                    // pass. Other balls still see newPos/newVel throughout it.
                    b.old_position = p;
                    b.old_velocity = v;
                    self.store(b);
                }
            }
            i += 1;
        }
        i = 0;
        while i < self.free_ids.len() {
            let Some(id) = self.free_ids.get(i) else {
                return Err(OriginalParkError::OutsideDeclaredScope);
            };
            if self.contains(id) {
                let mut b = self.retained(id);
                let index = self
                    .index(id, true)
                    .ok_or(OriginalParkError::OutsideDeclaredScope)?;
                if !b.is_moribund && !dead_bubbles[index] {
                    core::mem::swap(&mut b.position, &mut b.old_position);
                    core::mem::swap(&mut b.velocity, &mut b.old_velocity);
                    self.calculate_yaw_pitch_roll(&mut b, false);
                    self.store(b);
                    if !(b.position.sub(b.old_position).norm_squared() < 0.1) {
                        if self.is_master {
                            self.reinsert_query_master_ball(id)?;
                        } else {
                            self.boundary(OriginalBoundaryCall::InsertBoxes(id));
                        }
                    }
                }
            }
            i += 1;
        }
        self.handle_proximities()?;
        let Some(next) = self.current_time.checked_add(1) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        self.current_time = next;
        if self.scope_error {
            Err(OriginalParkError::OutsideDeclaredScope)
        } else {
            Ok(())
        }
    }

    fn modulo_2pi(old: &mut f64, new: &mut f64) {
        let period = 6.2831853071795862;
        *new %= period;
        *old %= period;
        if (*new - *old).abs() < (*new + period - *old).abs() {
            if (*new - *old).abs() >= (*new - period - *old).abs() {
                *new -= period;
            }
        } else if (*new + period - *old).abs() < (*new - period - *old).abs() {
            *new += period;
        } else {
            *new -= period;
        }
    }
    pub fn set_free(&mut self, id: i64, flag: bool) {
        if !self.contains(id) || self.retained(id).is_free == flag {
            return;
        }
        let mut b = self.retained(id);
        b.is_free = flag;
        self.store(b);
        if flag {
            self.update_free_registry(id, true);
            self.boundary(OriginalBoundaryCall::RemoveMiniShapes(id));
            let mut b = self.retained(id);
            if let Some(t) = self.time.checked_sub(10_000_000) {
                b.new_time = t;
            } else {
                self.scope_error = true;
            }
            self.store(b);
        } else {
            self.update_free_registry(id, false);
            self.stop_pointer(id);
            self.set_velocity(id, Vec3::ZERO);
            let mut b = self.retained(id);
            b.last_g = Vec3::ZERO;
            self.store(b);
            self.boundary(OriginalBoundaryCall::InsertBoxes(id));
            self.boundary(OriginalBoundaryCall::AddMiniShapes(id));
        }
    }
    pub fn set_massive(&mut self, id: i64, flag: bool) {
        if self.contains(id) {
            let mut b = self.retained(id);
            b.is_massive = flag;
            self.store(b);
        }
    }
    pub fn set_global(&mut self, id: i64, flag: bool) {
        if self.contains(id) {
            let mut b = self.retained(id);
            b.is_global = flag;
            self.store(b);
        }
    }
    pub fn set_interactive(&mut self, id: i64, flag: bool) {
        if self.contains(id) {
            let mut b = self.retained(id);
            if b.is_interactive != flag {
                // SetBallInteractive changes the separate Python set, with
                // Decrease preceding the flag write and Increase following
                // it. No native callback observes these intermediate writes.
                if let Some(p) = &mut self.query_bubbles {
                    if p.dictionary
                        .set_interactive_membership(id, b.new_bubble, flag)
                        .is_err()
                    {
                        self.scope_error = true;
                    }
                }
                b.is_interactive = flag;
                self.store(b);
                self.boundary(OriginalBoundaryCall::UpdateInteractiveMembership(id));
            }
        }
    }
    pub fn set_boid(&mut self, id: i64, flag: bool) {
        if self.contains(id) {
            if flag {
                self.set_mode(id, OriginalMode::Boid);
            } else {
                self.stop_pointer(id);
            }
        }
    }
    pub fn set_rigid(&mut self, id: i64) {
        if self.contains(id) {
            self.stop_pointer(id);
            self.set_mode(id, OriginalMode::Rigid);
        }
    }
    pub fn set_troll(&mut self, id: i64, delay: i32) {
        if !self.contains(id) {
            return;
        }
        self.stop_pointer(id);
        self.set_free(id, true);
        self.set_interactive(id, true);
        let mut b = self.retained(id);
        if let Some(stamp) = self.current_time.checked_add(i64::from(delay.max(1))) {
            b.effect_stamp = stamp;
        } else {
            self.scope_error = true;
        }
        self.store(b);
        self.set_mode(id, OriginalMode::Troll);
    }
    pub fn set_harmonic(
        &mut self,
        id: i64,
        harmonic: i64,
        corporation: i32,
        alliance: i32,
        field: bool,
    ) {
        if !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        b.harmonic = harmonic;
        b.corporation_id = corporation;
        b.alliance_id = alliance;
        self.store(b);
        if field {
            self.stop_pointer(id);
            self.set_mode(id, OriginalMode::Field);
        } else if b.mode == OriginalMode::Field {
            self.stop_pointer(id);
        }
    }
    pub fn follow(&mut self, src: i64, dst: i64, range: f32, orbit: bool) {
        if !range.is_finite() || src == dst || !self.contains(src) || !self.contains(dst) {
            return;
        }
        let target = self.retained(dst);
        if target.is_moribund
            || target.is_cloaked != 0
            || self.retained(src).new_bubble != target.new_bubble
        {
            return;
        }
        self.stop_pointer(src);
        let mut b = self.retained(src);
        b.follow_id = dst;
        b.follow_range = range;
        self.store(b);
        self.set_mode(
            src,
            if orbit {
                OriginalMode::Orbit
            } else {
                OriginalMode::Follow
            },
        );
    }
    pub fn missile_follow(&mut self, src: i64, dst: i64, owner: i64) {
        if src == dst || src == owner || dst == owner || !self.contains(src) || !self.contains(dst)
        {
            return;
        }
        let target = self.retained(dst);
        if target.is_moribund {
            return;
        }
        self.stop_pointer(src);
        let mut b = self.retained(src);
        if b.new_bubble != target.new_bubble || target.is_cloaked != 0 {
            return;
        }
        b.follow_id = dst;
        b.follow_range = -(b.radius + target.radius);
        b.owner_id = owner;
        b.effect_stamp = self.current_time;
        b.goto = b.velocity.normalize().scale(1e16);
        b.speed_fraction = 1.0;
        self.store(b);
        self.set_mode(src, OriginalMode::Missile);
    }
    /// PyLaunchMissile's signed-byte controls and actual initial-velocity
    /// algorithm. Missing owners/targets and cloaking retain source returns.
    pub fn launch_missile(
        &mut self,
        src: i64,
        dst: i64,
        owner: i64,
        aimed: i8,
        massive: i8,
    ) -> Result<(), OriginalParkError> {
        if dst < 0 {
            return Err(OriginalParkError::RuntimeError);
        }
        let Some(owner_check) = owner.checked_abs() else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        if !self.contains(owner_check) {
            return Ok(());
        }
        let launcher = self.retained(owner_check);
        let target = if dst != owner && self.contains(dst) {
            Some(self.retained(dst))
        } else {
            None
        };
        let mut maximum = 0.0_f64;
        let velocity = if let Some(target) = target.filter(|b| aimed == 1 && b.is_cloaked == 0) {
            target.position.sub(launcher.position).normalize()
        } else {
            maximum = f64::from(launcher.max_velocity);
            let mut direction = launcher.velocity.normalize();
            if direction.norm_squared() == 0.0 {
                if self.dynamical_orientation {
                    return Err(OriginalParkError::OutsideDeclaredScope);
                }
                direction = crate::original_rotated_vector(
                    &launcher,
                    Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                );
            }
            launcher
                .velocity
                .add(direction.scale(if 150.0 < maximum { maximum } else { 150.0 }))
        };
        if massive != 0 {
            self.set_massive(src, true);
        }
        self.set_position(src, launcher.position);
        self.set_velocity(src, velocity);
        if target.is_some_and(|b| b.is_cloaked != 0) {
            return Ok(());
        }
        if target.is_none() {
            if maximum > 0.0 {
                self.goto_direction(src, velocity);
            }
        } else {
            self.missile_follow(src, dst, owner);
        }
        Ok(())
    }
    #[must_use]
    pub fn followers(&self, dst: i64) -> BoundedList<i64, 8> {
        let mut followers = BoundedList::new();
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(b)) = self.balls.get(i) {
                if b.in_park && b.mode.follows() && b.follow_id == dst {
                    followers.push(b.id);
                }
            }
            i = i.saturating_add(1);
        }
        followers
    }
    pub fn goto_point(&mut self, id: i64, point: Vec3) {
        if !self.contains(id)
            || !point.x.is_finite()
            || !point.y.is_finite()
            || !point.z.is_finite()
        {
            return;
        }
        self.stop_pointer(id);
        let mut b = self.retained(id);
        b.goto = point;
        if b.speed_fraction == 0.0 {
            b.speed_fraction = 1.0;
        }
        self.store(b);
        self.set_mode(id, OriginalMode::Goto);
    }
    pub fn goto_direction(&mut self, id: i64, direction: Vec3) {
        if !self.contains(id)
            || !direction.x.is_finite()
            || !direction.y.is_finite()
            || !direction.z.is_finite()
        {
            return;
        }
        self.goto_point(
            id,
            self.retained(id)
                .position
                .add(direction.normalize().scale(1e17)),
        );
    }
    pub fn warp_to(&mut self, id: i64, point: Vec3, min_range: f64, warp_factor: i32) {
        if !self.contains(id)
            || !point.x.is_finite()
            || !point.y.is_finite()
            || !point.z.is_finite()
        {
            return;
        }
        let b = self.retained(id);
        if b.position.sub(point).norm_squared() < 1e10 {
            self.goto_point(id, point);
            self.boundary(OriginalBoundaryCall::ExitWarp(id, true));
            return;
        }
        if b.mode == OriginalMode::Warp && point == b.goto {
            return;
        }
        self.stop_pointer(id);
        let mut b = self.retained(id);
        b.goto = point;
        b.effect_stamp = -1;
        b.follow_range = 0.0;
        b.follow_id = (if min_range < 0.0 { 0.0 } else { min_range }).to_bits() as i64;
        b.owner_id = i64::from(warp_factor.max(1));
        self.store(b);
        self.set_mode(id, OriginalMode::Warp);
    }
    fn real_warp(&mut self, ball: &mut OriginalParkBall) -> Result<(), OriginalParkError> {
        self.stop_all_followers(ball.id);
        let delta = ball.position.sub(ball.goto).normalize();
        let destination = ball
            .goto
            .add(delta.scale(f64::from_bits(ball.follow_id as u64)));
        ball.goto = destination;
        ball.effect_stamp = self.current_time;
        ball.last_collision = destination.sub(ball.position).length();
        ball.is_massive = false;
        ball.sensor.active = false;
        Ok(())
    }
    pub fn entity_warp_in(
        &mut self,
        id: i64,
        point: Vec3,
        factor: i32,
    ) -> Result<(), OriginalParkError> {
        self.warp_to(id, point, 0.0, factor);
        if !self.contains(id) || self.retained(id).mode == OriginalMode::Goto {
            return Ok(());
        }
        if self.dynamical_orientation {
            return Err(OriginalParkError::OutsideDeclaredScope);
        }
        let mut b = self.retained(id);
        self.real_warp(&mut b)?;
        b.velocity = point.normalize().scale(149_597_870_700.0);
        let Some(stamp) = self.current_time.checked_sub(5) else {
            return Err(OriginalParkError::OutsideDeclaredScope);
        };
        b.effect_stamp = if stamp > 0 { stamp } else { 0 };
        b.last_collision = b.position.sub(b.goto).length();
        self.store(b);
        if self.is_master {
            self.boundary(OriginalBoundaryCall::ActivateWarp(id, self.current_time));
        }
        Ok(())
    }
    pub fn load_original_formations(&mut self) {
        self.formations_loaded = true;
    }
    pub fn reserve_original_formation_slot(&mut self, id: i64) -> i8 {
        if !self.contains(id) || !self.formations_loaded {
            return -1;
        }
        let mut b = self.retained(id);
        let count = match b.formation_id {
            0 | 1 => Some(4),
            2 => Some(1),
            _ => None,
        };
        let (reserved, slot) = crate::reserve_formation_slot(b.formation_slots, count);
        b.formation_slots = reserved;
        self.store(b);
        slot
    }
    pub fn set_formation(&mut self, id: i64, formation: i8) {
        if !self.contains(id) || !self.formations_loaded || formation > 2 || formation < -1 {
            return;
        }
        let old = self.retained(id).formation_id;
        if old >= 0 && formation != old {
            let mut i = 0_usize;
            while i < 8 {
                if let Some(Some(f)) = self.balls.get(i).copied() {
                    if f.in_park
                        && f.follow_id == id
                        && (formation == -1
                            || (formation == 2
                                && f.mode == OriginalMode::Formation
                                && f.effect_stamp > 0))
                    {
                        self.stop_pointer(f.id);
                    }
                }
                i = i.saturating_add(1);
            }
        }
        let mut b = self.retained(id);
        b.formation_id = formation;
        if formation == -1 {
            b.formation_slots = 0;
        }
        self.store(b);
    }
    pub fn formation_follow(&mut self, src: i64, dst: i64, slot: i8) {
        if src == dst || !self.contains(src) || !self.contains(dst) || !self.formations_loaded {
            return;
        }
        let dest = self.retained(dst);
        if dest.formation_id < 0
            || dest.formation_id > 2
            || slot >= if dest.formation_id == 2 { 1 } else { 4 }
        {
            return;
        }
        self.stop_pointer(src);
        if dest.new_bubble != self.retained(src).new_bubble || dest.is_cloaked != 0 {
            return;
        }
        let mut b = self.retained(src);
        b.follow_id = dst;
        b.effect_stamp = i64::from(slot);
        self.store(b);
        self.set_mode(src, OriginalMode::Formation);
    }
    #[must_use]
    pub fn center_distance(&self, src: i64, dst: i64) -> f64 {
        self.retained(src)
            .position
            .sub(self.retained(dst).position)
            .length()
    }
    #[must_use]
    pub fn surface_distance(&self, src: i64, dst: i64) -> f64 {
        // Public Python method is Thunkers.cpp::PyGetSurfaceDist: no clamp,
        // source radius subtracted before target radius.
        self.center_distance(src, dst)
            - f64::from(self.retained(src).radius)
            - f64::from(self.retained(dst).radius)
    }

    pub fn check_visibility(&self, src: i64, dst: i64) -> i64 {
        if !self.contains(src) || !self.contains(dst) {
            return -2;
        }
        let source = self.retained(src);
        let target = self.retained(dst);
        if source.new_bubble != target.new_bubble {
            return -2;
        }
        let mut ray = target.position.sub(source.position);
        let distance = ray.length();
        ray = ray.normalize();
        let mut i = 0;
        while i < 8 {
            if let Some(Some(occluder)) = self.balls.get(i) {
                if occluder.in_park
                    && occluder.id != src
                    && occluder.id != dst
                    && occluder.new_bubble == source.new_bubble
                    && occluder.is_massive
                    && occluder.is_cloaked == 0
                {
                    let center = occluder.position.sub(source.position);
                    let v = ray.dot(center);
                    let r = f64::from(occluder.radius);
                    let discriminant = r * r - (center.dot(center) - v * v);
                    if !(discriminant < 0.0) {
                        let d = discriminant.sqrt();
                        if v - d > 0.0 && v - d < distance {
                            return occluder.id;
                        }
                    }
                }
            }
            i += 1;
        }
        0
    }

    pub fn cloak_ball(&mut self, id: i64, mode: i8, range: f32) -> Result<(), OriginalParkError> {
        if !self.contains(id) || mode <= 0 {
            return Ok(());
        }
        if self.is_master && mode == 1 {
            self.add_proximity_sensor(id, range, 2.0, 0, false, 0, 2_147_483_647)?;
        }
        // Source fixtures have no raw writes to the native followers registry.
        // Its storage/alias correspondence remains a separate obligation.
        let mut i = 0;
        while i < 8 {
            if let Some(Some(f)) = self.balls.get(i).copied() {
                if f.in_park && f.mode.follows() && f.follow_id == id {
                    self.stop_pointer(f.id);
                }
            }
            i += 1;
        }
        let mut b = self.retained(id);
        b.is_cloaked = mode;
        b.is_massive = false;
        self.store(b);
        Ok(())
    }

    pub fn uncloak_ball(&mut self, id: i64) {
        if !self.contains(id) {
            return;
        }
        let mut b = self.retained(id);
        if self.is_master && b.is_cloaked == 1 {
            self.remove_proximity_sensor(id);
            b = self.retained(id);
        }
        b.is_cloaked = 0;
        if !(b.mode == OriginalMode::Warp && b.effect_stamp >= 0) {
            b.is_massive = true;
            let mut i = 0;
            while i < self.proximity_ids.len() {
                if let Some(sensor_id) = self.proximity_ids.get(i) {
                    if self.contains(sensor_id) {
                        let mut sensor = self.retained(sensor_id);
                        if sensor.new_bubble == b.new_bubble
                            && !sensor.is_moribund
                            && sensor.has_proximity
                        {
                            let mut retained = BoundedList::new();
                            let mut j = 0;
                            while j < sensor.sensor.balls.len() {
                                if let Some(other) = sensor.sensor.balls.get(j) {
                                    if other != id && !retained.push(other) {
                                        self.scope_error = true;
                                    }
                                }
                                j += 1;
                            }
                            sensor.sensor.balls = retained;
                            self.store(sensor);
                        }
                    }
                }
                i += 1;
            }
        }
        self.store(b);
    }

    pub fn scan_cone(
        &self,
        src: i64,
        mut angle: f64,
        range: f64,
        mut direction: Vec3,
    ) -> Result<Option<BoundedList<i64, 8>>, OriginalParkError> {
        if range <= 0.0 || !self.contains(src) {
            return Ok(None);
        }
        direction = direction.normalize();
        if direction.norm_squared() < 0.5 {
            return Ok(None);
        }
        let source = self.retained(src);
        let range2 = range * range;
        angle *= 0.5;
        let cosine2 = angle.cos() * angle.cos();
        let sphere = angle > core::f64::consts::PI;
        let mut result = BoundedList::new();
        let mut i = 0;
        while i < 8 {
            if let Some(Some(b)) = self.balls.get(i) {
                if b.in_park && b.id != src && b.id >= 0 && b.is_cloaked == 0 {
                    let offset = b.position.sub(source.position);
                    let projection = direction.dot(offset);
                    let length2 = offset.dot(offset);
                    if (sphere
                        || (projection >= 0.0 && projection * projection >= cosine2 * length2))
                        && length2 <= range2
                        && !result.push(b.id)
                    {
                        return Err(OriginalParkError::OutsideDeclaredScope);
                    }
                }
            }
            i += 1;
        }
        Ok(Some(result))
    }
    #[must_use]
    pub fn accuracy(
        &self,
        src: i64,
        dst: i64,
        mut optimal: f64,
        falloff: f64,
        mut tracking: f64,
        signature: f64,
        optimal_signature: f64,
    ) -> (f64, f64, f64) {
        if src == dst
            || !self.contains(src)
            || !self.contains(dst)
            || optimal < 0.0
            || tracking == 0.0
            || signature == 0.0
            || falloff == 0.0
        {
            return (0.0, optimal, tracking);
        }
        let source = self.retained(src);
        let target = self.retained(dst);
        let radial = target.position.sub(source.position);
        let dist = radial.length();
        let short = (dist - f64::from(target.radius) - f64::from(source.radius)).max(0.0);
        if dist == 0.0 {
            return (0.0, optimal, tracking);
        }
        let combined = target.velocity.sub(source.velocity);
        let radial_speed = combined.dot(radial) / dist;
        let speed2 = (combined.norm_squared() - radial_speed * radial_speed) / (dist * dist);
        let mut exponent = optimal_signature / (tracking * signature);
        exponent *= -exponent * speed2;
        if short > optimal {
            exponent -= (short - optimal) * (short - optimal) / (falloff * falloff);
        }
        optimal = short;
        tracking = speed2.sqrt();
        (2.0_f64.powf(exponent), optimal, tracking)
    }
    pub fn start(&mut self) {
        if !self.is_running {
            // Tick registration has no partition or proximity mutation in
            // boundary(); record the same external event directly.
            self.record_boundary(OriginalBoundaryCall::RegisterTicks);
            self.is_running = true;
        }
    }
    pub fn pause(&mut self) {
        if self.is_running {
            self.record_boundary(OriginalBoundaryCall::UnregisterTicks);
            self.is_running = false;
        }
    }
    pub fn adjust_times(&mut self, delta: i64) {
        if let Some(time) = self.time.checked_add(delta) {
            self.time = time;
        } else {
            self.scope_error = true;
        }
        let mut i = 0_usize;
        while i < 8 {
            if let Some(Some(mut b)) = self.balls.get(i).copied() {
                if b.in_park {
                    let mut adjust = |t: &mut i64| {
                        if let Some(time) = t.checked_add(delta) {
                            *t = time;
                        } else {
                            self.scope_error = true;
                        }
                    };
                    if b.client_ball {
                        adjust(&mut b.pos_update_time);
                        adjust(&mut b.rot_update_time);
                    }
                    if b.new_time != 0 {
                        adjust(&mut b.new_time);
                    }
                    if b.old_time != 0 {
                        adjust(&mut b.old_time);
                    }
                    self.store(b);
                }
            }
            i = i.saturating_add(1);
        }
    }
}
