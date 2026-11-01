use bevy::prelude::*;
use destiny_rs::{DestinySettingsPlugin, SettingsConfiguration};

fn show_configuration(settings: Res<'_, SettingsConfiguration>) {
    if settings.is_changed() {
        println!("Collision iterations: {}", settings.m_collisionMaxIterations);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(DestinySettingsPlugin)
        .add_systems(Update, show_configuration)
        .run();
}
