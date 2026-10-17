use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    asset::{AssetApp, AssetPlugin},
    prelude::*,
};
use blueexposure::{BlueAppExt, BlueExposurePlugin, BlueScriptCallback};

use crate::{
    BlueClock, BlueLifecycle, BlueLifecycleState, BlueObjectMetadata, BluePaths, BlueResource,
    BlueResourceIndex, BlueStartupArgs,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum BlueSet {
    Lifecycle,
    Resources,
    Python,
    Callbacks,
}

#[derive(Default)]
pub struct BluePlugin;

impl Plugin for BluePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueLifecycle>()
            .init_resource::<BlueStartupArgs>()
            .init_resource::<BluePaths>()
            .init_resource::<BlueClock>()
            .init_resource::<BlueResourceIndex>()
            .init_asset::<BlueResource>()
            .register_asset_reflect::<BlueResource>()
            .register_type::<BlueLifecycle>()
            .register_type::<BlueLifecycleState>()
            .register_blue_class::<BlueStartupArgs>()
            .register_blue_class::<BluePaths>()
            .register_blue_class::<BlueClock>()
            .register_blue_class::<BlueObjectMetadata>()
            .register_blue_class::<BlueScriptCallback>()
            .configure_sets(
                Update,
                (
                    BlueSet::Lifecycle,
                    BlueSet::Resources,
                    BlueSet::Python,
                    BlueSet::Callbacks,
                )
                    .chain(),
            )
            .add_systems(Update, advance_blue_clock.in_set(BlueSet::Lifecycle));
    }
}

/// Opinionated headless Blue runtime. This is the default Carbon integration surface: Bevy is
/// the scheduler/ECS/asset kernel, BlueExposure owns reflection/Python registration, and Blue
/// installs its conventions after both.
pub struct BluePlugins;

impl PluginGroup for BluePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add_group(MinimalPlugins)
            .add(AssetPlugin::default())
            .add(BlueExposurePlugin)
            .add(BluePlugin)
    }
}

fn advance_blue_clock(mut clock: ResMut<'_, BlueClock>, mut lifecycle: ResMut<'_, BlueLifecycle>) {
    if lifecycle.state() == BlueLifecycleState::Starting {
        lifecycle.mark_running();
    }
    if lifecycle.state() == BlueLifecycleState::Running {
        clock.advance();
    }
}
