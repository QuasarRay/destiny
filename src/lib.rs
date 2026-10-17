// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
//! Destiny-named adapters backed by Bevy 0.19, Avian 0.7, and optional
//! Lightyear/Replicon Carbon networking components.

mod accuracy;
mod ball_defaults;
mod ball_lifecycle;
mod bubbles;
mod carbon_codec;
mod collision_identity;
mod evolution_phase;
mod ffi;
mod ffi_lifecycle;
mod formation;
mod generated_registry;
mod harmonic;
mod iterative_collision;
mod missile;
mod mushroom;
mod noniterative_collision;
mod notification_batch;
mod operation_gate;
mod proximity;
mod runtime;
mod warp_entry;

#[cfg(feature = "carbon-network")]
pub mod network;

pub use ffi::{DbcBuffer, DbcRuntime};
pub use generated_registry::{GENERATED_TITLE_RECORDS, GeneratedTitleRecord};
pub use runtime::{
    BallSnapshot, CarbonNetworkOutbox, CompatError, CompatOptions, CompatRequest, CompatRuntime,
    DestinyBallId, DestinyBallMetadata, DestinyMass, DestinyPendingRemoval,
    DestinyPresentationAngularVelocity, MotionSnapshot, RangeNotificationSnapshot,
};

mod generated_geometry;

mod generated_angular;
pub use generated_angular::{
    Angles as DestinyAngularState, RollParameters as DestinyRollParameters,
};

pub use runtime::DestinyMaxVelocity;

mod generated_missile;

mod generated_partition;

mod generated_liveness;
pub use generated_liveness::LivenessState as DestinyLivenessState;

mod generated_forces;
pub use generated_forces::Forces as DestinyForceState;

mod generated_contacts;
pub use generated_contacts::{
    CollisionEvent as DestinyCollisionEvent, CollisionNotice as DestinyCollisionNotice,
    CollisionPass as DestinyCollisionPass,
};

mod generated_kinematics;
pub use generated_kinematics::Pose as DestinyOldPose;

#[cfg(test)]
#[path = "../verification/kani/src/iterative_sphere_production_contracts.rs"]
mod concrete_iterative_kernel_fixtures;
#[cfg(test)]
#[path = "../verification/kani/src/noniterative_production_contracts.rs"]
mod concrete_noniterative_kernel_fixtures;
