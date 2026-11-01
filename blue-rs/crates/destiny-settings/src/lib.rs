//! Native Bevy Destiny settings, integrated with the CGP-enabled Blue-rs workspace.
//!
//! C++ sources: Settings.h, Settings.cpp, Settings_Blue.cpp (CCP, 2023).
//! Bevy World owns the canonical state; Blue/Python requests are staged per world.
mod settings;

pub use settings::{
    ApplyStagedSettings, CanApplyDestinySettings, CanReadDestinySettings,
    DestinySettingsApplierComponent, DestinySettingsPlugin,
    DestinySettingsReaderComponent, GlobalSettings, ReadStagedSettings,
    SettingsBridge, SettingsConfiguration, register_destiny_settings_python,
};
