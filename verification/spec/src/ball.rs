use crate::MotionMode;

pub const MAX_ORIGINAL_FORMATION_SLOTS: u8 = 16;
pub const ORIGINAL_TEN_BILLION: f64 = 10_000_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OriginalBallDefaults {
    pub id: i64,
    pub mass: f64,
    pub radius: f64,
    pub max_velocity: f64,
    pub is_free: bool,
    pub is_global: bool,
    pub is_massive: bool,
    pub is_interactive: bool,
    pub is_cloaked: bool,
    pub is_moribund: bool,
    pub harmonic: i64,
    pub corporation_id: i64,
    pub alliance_id: i64,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub agility: f64,
    pub speed_fraction: f64,
    pub mode: MotionMode,
    pub goto: [f64; 3],
    pub follow_id: i64,
    pub follow_range: f64,
    pub owner_id: i64,
    pub effect_stamp: i64,
    pub new_bubble_id: i64,
    pub old_bubble_id: i64,
    pub formation_id: i64,
    pub mini_ball_count: u16,
    pub has_ballpark: bool,
}

#[must_use]
pub const fn original_ball_defaults() -> OriginalBallDefaults {
    OriginalBallDefaults {
        id: 0,
        mass: 0.0,
        radius: 0.0,
        max_velocity: 0.0,
        is_free: false,
        is_global: false,
        is_massive: false,
        is_interactive: false,
        is_cloaked: false,
        is_moribund: false,
        harmonic: -1,
        corporation_id: -1,
        alliance_id: -1,
        position: [ORIGINAL_TEN_BILLION; 3],
        velocity: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        agility: 1.0,
        speed_fraction: 1.0,
        mode: MotionMode::Stop,
        goto: [0.0; 3],
        follow_id: 0,
        follow_range: 10.0,
        owner_id: 0,
        effect_stamp: 0,
        new_bubble_id: -1,
        old_bubble_id: -1,
        formation_id: 255,
        mini_ball_count: 0,
        has_ballpark: false,
    }
}

#[must_use]
pub const fn child_count_after_successful_add(current: u16) -> Option<u16> {
    current.checked_add(1)
}

#[must_use]
pub fn mini_capsule_radius_accepted(radius: f64) -> bool {
    radius > 0.0
}

#[must_use]
pub fn child_count_after_capsule_attempt(current: u16, radius: f64) -> Option<u16> {
    if mini_capsule_radius_accepted(radius) {
        current.checked_add(1)
    } else {
        Some(current)
    }
}

#[must_use]
pub const fn identity_rotated_vector(vector: [u64; 3]) -> [u64; 3] {
    vector
}

#[must_use]
pub fn proximity_sensor_accepted(ball_radius: f64, range: f64) -> bool {
    ball_radius + range >= 0.0
}

/// Contract for original tests that reserve slots from an initially empty
/// formation in order. reserved_prefix means slots before it are occupied.
#[must_use]
pub const fn next_sequential_formation_slot(
    reserved_prefix: u8,
    slot_count: u8,
    formation_assigned: bool,
    park_present: bool,
    formation_valid: bool,
) -> i8 {
    if !formation_assigned
        || !park_present
        || !formation_valid
        || slot_count > MAX_ORIGINAL_FORMATION_SLOTS
        || reserved_prefix >= slot_count
    {
        -1
    } else {
        reserved_prefix as i8
    }
}

/// When every valid slot is occupied except one explicitly freed slot, the
/// original first-free scan must reuse that slot.
#[must_use]
pub const fn slot_reused_after_free(
    freed_slot: u8,
    slot_count: u8,
    formation_assigned: bool,
    park_present: bool,
    formation_valid: bool,
) -> i8 {
    if !formation_assigned
        || !park_present
        || !formation_valid
        || slot_count == 0
        || slot_count > MAX_ORIGINAL_FORMATION_SLOTS
        || freed_slot >= slot_count
    {
        -1
    } else {
        freed_slot as i8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_defaults_match_test_ball_contract() {
        let d = original_ball_defaults();
        assert_eq!(d.id, 0);
        assert!(!d.is_free);
        assert!(!d.is_global);
        assert!(!d.is_massive);
        assert!(!d.is_interactive);
        assert!(!d.is_cloaked);
        assert!(!d.is_moribund);
        assert_eq!(d.position, [ORIGINAL_TEN_BILLION; 3]);
        assert_eq!(d.agility, 1.0);
        assert_eq!(d.speed_fraction, 1.0);
        assert_eq!(d.mode, MotionMode::Stop);
        assert_eq!(d.follow_range, 10.0);
        assert_eq!(d.formation_id, 255);
        assert_eq!(d.mini_ball_count, 0);
        assert!(!d.has_ballpark);
    }

    #[test]
    fn formation_sequence_and_reuse_match_original_examples() {
        for reserved in 0..4 {
            assert_eq!(
                next_sequential_formation_slot(reserved, 4, true, true, true),
                reserved as i8
            );
        }
        assert_eq!(next_sequential_formation_slot(4, 4, true, true, true), -1);
        assert_eq!(slot_reused_after_free(2, 4, true, true, true), 2);
    }
}
