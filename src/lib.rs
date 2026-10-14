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
