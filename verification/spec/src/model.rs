#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[must_use]
    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    #[must_use]
    pub fn norm_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MotionMode {
    #[default]
    Stop,
    GotoDirection,
    GotoPoint,
    Follow,
    FormationFollow,
    Orbit,
    Missile,
    Warp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallState {
    pub id: i64,
    pub mass: f64,
    pub radius: f64,
    pub max_velocity: f64,
    pub agility: f64,
    pub speed_fraction: f64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_cloaked: bool,
    pub bubble_id: i64,
    pub mode: MotionMode,
}

impl Default for BallState {
    fn default() -> Self {
        Self {
            id: 0,
            mass: 0.0,
            radius: 0.0,
            max_velocity: 0.0,
            agility: 0.0,
            speed_fraction: 1.0,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            is_free: true,
            is_global: false,
            is_massive: true,
            is_interactive: true,
            is_cloaked: false,
            bubble_id: 0,
            mode: MotionMode::Stop,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParkState {
    pub evolving: bool,
    pub running: bool,
    pub time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecError {
    AddDuringEvolve,
    InvalidBallId,
    BallNotFound,
    SelfTarget,
    MoribundTarget,
    InvalidRange,
    CrossBubbleTarget,
}

/// Contract extracted from
/// `test_balls_can_not_be_added_during_evolve`.
#[must_use]
pub const fn add_ball_is_permitted(park: &ParkState) -> bool {
    !park.evolving
}

/// Original Destiny radius/max-speed policy: negative values are ignored,
/// while zero is accepted.
#[must_use]
pub fn apply_non_negative_setter(current: f64, requested: f64) -> f64 {
    if requested >= 0.0 { requested } else { current }
}

/// Original Destiny mass/agility policy: zero and negative values are ignored.
#[must_use]
pub fn apply_positive_setter(current: f64, requested: f64) -> f64 {
    if requested > 0.0 { requested } else { current }
}

#[must_use]
pub const fn non_negative_setter_accepts(requested: f64) -> bool {
    requested >= 0.0
}

#[must_use]
pub const fn positive_setter_accepts(requested: f64) -> bool {
    requested > 0.0
}

/// Contract extracted from the speed-fraction setter tests.
#[must_use]
pub fn clamp_speed_fraction(requested: f64) -> f64 {
    if requested < 0.0 {
        0.0
    } else if requested > 1.0 {
        1.0
    } else {
        requested
    }
}

#[must_use]
pub fn center_distance_squared(left: &BallState, right: &BallState) -> f64 {
    left.position.sub(right.position).norm_squared()
}

#[must_use]
pub fn center_distance(left: &BallState, right: &BallState) -> f64 {
    center_distance_squared(left, right).sqrt()
}

/// The original tests specify surface distance as center distance minus the
/// combined radii; no engine collider representation participates in this
/// contract.
#[must_use]
pub fn surface_distance(left: &BallState, right: &BallState) -> f64 {
    center_distance(left, right) - left.radius - right.radius
}


#[must_use]
pub const fn follow_allowed(src_id: i64, dst_id: i64, target_moribund: bool) -> bool {
    src_id != dst_id && !target_moribund
}

#[must_use]
pub fn orbit_allowed(
    src_id: i64,
    dst_id: i64,
    range: f64,
    target_cloaked: bool,
    same_bubble: bool,
) -> bool {
    src_id != dst_id && range.is_finite() && !target_cloaked && same_bubble
}

#[must_use]
pub const fn visibility_occluder(is_massive: bool, is_cloaked: bool) -> bool {
    is_massive && !is_cloaked
}

#[must_use]
pub const fn proximity_eligible(
    owner_is_free: bool,
    target_is_interactive: bool,
    only_interactives: bool,
) -> bool {
    owner_is_free && (!only_interactives || target_is_interactive)
}

#[must_use]
pub const fn uncloak_restores_massive(in_warp: bool) -> bool {
    !in_warp
}

#[must_use]
pub const fn stopped_mode(_: MotionMode) -> MotionMode {
    MotionMode::Stop
}

#[must_use]
pub fn missile_follow_range(src_radius: f64, dst_radius: f64) -> f64 {
    -(src_radius + dst_radius)
}


#[must_use]
pub fn adjust_time(current: i64, delta: i64) -> Option<i64> {
    current.checked_add(delta)
}

#[must_use]
pub const fn running_after_start() -> bool {
    true
}

#[must_use]
pub const fn running_after_pause() -> bool {
    false
}

#[must_use]
pub const fn assign_flag(requested: bool) -> bool {
    requested
}

#[must_use]
pub const fn assign_vec3(requested: Vec3) -> Vec3 {
    requested
}

#[must_use]
pub fn surface_distance_from_center(
    center_distance: f64,
    left_radius: f64,
    right_radius: f64,
) -> f64 {
    center_distance - left_radius - right_radius
}


#[must_use]
pub fn membership_count_after_add(current: usize) -> Option<usize> {
    current.checked_add(1)
}

#[must_use]
pub fn membership_count_after_remove(current: usize, was_present: bool) -> usize {
    if was_present && current > 0 {
        current - 1
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_is_rejected_while_evolving() {
        assert!(!add_ball_is_permitted(&ParkState {
            evolving: true,
            ..ParkState::default()
        }));
    }

    #[test]
    fn negative_non_negative_setter_request_is_noop() {
        assert_eq!(apply_non_negative_setter(7.0, -1.0), 7.0);
    }

    #[test]
    fn non_positive_positive_setter_request_is_noop() {
        assert_eq!(apply_positive_setter(7.0, -1.0), 7.0);
        assert_eq!(apply_positive_setter(7.0, 0.0), 7.0);
        assert_eq!(apply_positive_setter(7.0, 2.0), 2.0);
    }

    #[test]
    fn speed_fraction_is_clamped_to_closed_unit_interval() {
        assert_eq!(clamp_speed_fraction(-1.0), 0.0);
        assert_eq!(clamp_speed_fraction(2.0), 1.0);
        assert_eq!(clamp_speed_fraction(0.25), 0.25);
    }
}
