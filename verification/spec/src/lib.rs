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
    apply_non_negative_setter, apply_positive_setter, assign_flag, assign_vec3, center_distance,
    center_distance_squared, clamp_speed_fraction, follow_allowed, missile_follow_range,
    non_negative_setter_accepts, orbit_allowed, positive_setter_accepts, proximity_eligible,
    running_after_pause, running_after_start, stopped_mode, surface_distance,
    surface_distance_from_center, uncloak_restores_massive, visibility_occluder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    Specified,
    ImplementationMissing,
    ProofMissing,
    ProvedParity,
    KnownMismatch,
}
