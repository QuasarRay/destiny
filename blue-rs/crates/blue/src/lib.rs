mod clock;
mod lifecycle;
mod object;
mod paths;
mod plugin;
mod resource;
mod runtime;
mod startup;

pub use blueexposure;
pub use clock::BlueClock;
pub use lifecycle::{BlueLifecycle, BlueLifecycleState};
pub use object::BlueObjectMetadata;
pub use paths::{BluePathError, BluePaths};
pub use plugin::{BluePlugin, BluePlugins, BlueSet};
pub use resource::{BlueResource, BlueResourceIndex};
pub use runtime::BlueRuntime;
pub use startup::BlueStartupArgs;

pub mod prelude {
    pub use blueexposure::prelude::*;

    pub use crate::{
        BlueClock, BlueLifecycle, BlueLifecycleState, BlueObjectMetadata, BluePathError, BluePaths, BluePlugin,
        BluePlugins, BlueResource, BlueResourceIndex, BlueRuntime, BlueSet, BlueStartupArgs,
    };
}
