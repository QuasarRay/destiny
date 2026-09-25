#![forbid(unsafe_code)]

//! Bevy-independent contracts extracted from the original Destiny tests.
//!
//! This crate is intentionally engine-agnostic.  Adapters from a concrete
//! implementation may depend on Bevy/Avian, but the contracts in this crate
//! must not.

mod catalog;
mod model;

pub use catalog::{ORIGINAL_TEST_FILES, OriginalTestFile};
pub use model::{
    BallState, MotionMode, ParkState, SpecError, Vec3, add_ball_is_permitted, adjust_time,
    assign_flag, assign_vec3,
    apply_non_negative_setter, apply_positive_setter, clamp_speed_fraction, center_distance,
    center_distance_squared, follow_allowed, missile_follow_range, orbit_allowed,
    membership_count_after_add, membership_count_after_remove, non_negative_setter_accepts,
    positive_setter_accepts, proximity_eligible, stopped_mode,
    running_after_pause, running_after_start, surface_distance,
    surface_distance_from_center, uncloak_restores_massive,
    visibility_occluder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    Specified,
    ImplementationMissing,
    ProofMissing,
    ProvedParity,
    KnownMismatch,
}
