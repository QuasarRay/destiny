use bevy::prelude::*;
use destiny_settings::{DestinySettingsPlugin, SettingsConfiguration};

fn show_configuration(settings: Res<'_, SettingsConfiguration>) {
    if settings.is_changed() {
        println!("Collision iterations: {}", settings.collision_max_iterations);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(DestinySettingsPlugin)
        .add_systems(Update, show_configuration)
        .run();
}
