#![forbid(unsafe_code)]

//! Bevy-independent contracts extracted from the original Destiny tests.
//!
//! This crate is intentionally engine-agnostic.  Adapters from a concrete
//! implementation may depend on Bevy/Avian, but the contracts in this crate
//! must not.

mod ball;
mod catalog;
mod model;
mod visibility;

pub use ball::{
    MAX_ORIGINAL_FORMATION_SLOTS, ORIGINAL_TEN_BILLION, OriginalBallDefaults,
    child_count_after_capsule_attempt, child_count_after_successful_add, identity_rotated_vector,
    mini_capsule_radius_accepted, next_sequential_formation_slot, original_ball_defaults,
    proximity_sensor_accepted, slot_reused_after_free,
};
pub use catalog::{ORIGINAL_TEST_FILES, OriginalTestFile};
pub use model::{
    BallState, MotionMode, ParkState, SpecError, Vec3, add_ball_is_permitted, adjust_time,
    apply_non_negative_setter, apply_positive_setter, assign_flag, assign_vec3, center_distance,
    center_distance_squared, clamp_speed_fraction, follow_allowed, membership_count_after_add,
    membership_count_after_remove, missile_follow_range, non_negative_setter_accepts,
    orbit_allowed, positive_setter_accepts, proximity_eligible, running_after_pause,
    running_after_start, stopped_mode, surface_distance, surface_distance_from_center,
    uncloak_restores_massive, visibility_occluder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    Specified,
    ImplementationMissing,
    ProofMissing,
    ProvedParity,
    KnownMismatch,
}

pub use visibility::{
    VisibilityState, cloak_transition, scan_cone_pi_over_2_x, uncloak_transition,
    visibility_candidate_blocks, visibility_result,
};
