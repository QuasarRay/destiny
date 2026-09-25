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
    BallState, MotionMode, ParkState, SpecError, Vec3, add_ball_is_permitted,
    apply_non_negative_setter, apply_positive_setter, clamp_speed_fraction, center_distance,
    center_distance_squared, follow_allowed, missile_follow_range, orbit_allowed,
    non_negative_setter_accepts, positive_setter_accepts, proximity_eligible, stopped_mode,
    surface_distance, uncloak_restores_massive,
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
