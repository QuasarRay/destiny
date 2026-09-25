//! Destiny-named adapters backed by Bevy 0.19, Avian 0.7, and optional
//! Lightyear/Replicon Carbon networking components.

mod carbon_codec;
mod ffi;
mod generated_registry;
mod runtime;

#[cfg(feature = "carbon-network")]
pub mod network;

pub use ffi::{DbcBuffer, DbcRuntime};
pub use generated_registry::{GENERATED_TITLE_RECORDS, GeneratedTitleRecord};
pub use runtime::{
    BallSnapshot, CarbonNetworkOutbox, CompatError, CompatOptions, CompatRequest, CompatRuntime,
    DestinyBallId, DestinyBallMetadata, DestinyMass, DestinyPendingRemoval,
    DestinyPresentationAngularVelocity,
};
