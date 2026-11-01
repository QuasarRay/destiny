use bevy::ecs::reflect::ReflectResource;
use bevy::ecs::change_detection::DetectChanges;
use std::sync::{Arc, Mutex};

use bevy::prelude::{App, First, Last, Plugin, Res, ResMut, World};
use blueexposure::{blue_methods, blue_pyclass, BlueAppExt, BlueModule};
use cgp::prelude::*;
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyModuleMethods};

/// Destiny collision and orientation configuration. This is the canonical
/// resource in the Bevy World, and also a snapshot-value Python Blue class.
///
/// The C++ source documents a recommended collision iteration count >= 10,
/// but does not enforce it. This port preserves that behavior.
#[blue_pyclass(
    module = "destiny",
    name = "SettingsConfiguration",
    kind = "resource",
    doc = "Pass an instance of this to destiny.settings.Apply() to configure Destiny"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_snake_case)] // Original SettingsConfiguration field identities
pub struct SettingsConfiguration {
    /// The maximum number of collision iterations (assuming iterative collision is enabled).
    /// Recommended value >= 10; at 5 agile ships may fly through thin boxes.
    #[blue(read, write, name = "collisionMaxIterations")]
    pub m_collisionMaxIterations: usize,

    /// Boolean flag to enable/disable iterative collision.
    #[blue(read, write, name = "useIterativeCollision")]
    pub m_useIterativeCollision: bool,

    /// Boolean flag to enable/disable dynamical orientation.
    #[blue(read, write, name = "useDynamicalOrientation")]
    pub m_useDynamicalOrientation: bool,

    /// Boolean flag to disable dynamical orientation for missiles
    /// (if dynamical orientation is enabled).
    #[blue(read, write, name = "disableDynamicalOrientationForMissiles")]
    pub m_disableDynamicalOrientationForMissiles: bool,

    /// Use the new orbit behavior.
    #[blue(read, write, name = "useNewOrbit")]
    pub m_useNewOrbit: bool,
}

impl SettingsConfiguration {
    pub const DEFAULT: Self = Self {
        m_collisionMaxIterations: 20,
        m_useIterativeCollision: false,
        m_useDynamicalOrientation: false,
        m_disableDynamicalOrientationForMissiles: false,
        m_useNewOrbit: false,
    };
}

impl Default for SettingsConfiguration {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[blue_methods]
impl SettingsConfiguration {
    #[blue(new)]
    fn py_new() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug)]
struct BridgeState {
    desired: SettingsConfiguration,
    pending: bool,
}

/// Per-world, thread-safe command/snapshot bridge.
///
/// Python threads never borrow a Bevy World directly. Writes become visible
/// to Python immediately and reach Bevy at the beginning of a later update.
/// Bevy-owned changes are published to Python at the end of an update.
#[derive(bevy::prelude::Resource, Clone, Debug)]
pub struct SettingsBridge {
    state: Arc<Mutex<BridgeState>>,
}

impl Default for SettingsBridge {
    fn default() -> Self {
        Self::new(SettingsConfiguration::DEFAULT)
    }
}

impl SettingsBridge {
    pub fn new(config: SettingsConfiguration) -> Self {
        Self {
            state: Arc::new(Mutex::new(BridgeState { desired: config, pending: false })),
        }
    }

    fn stage(&self, desired: SettingsConfiguration) {
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        state.desired = desired;
        state.pending = true;
    }

    #[must_use]
    pub fn snapshot(&self) -> SettingsConfiguration {
        self.state.lock().unwrap_or_else(|err| err.into_inner()).desired
    }

    fn take_pending(&self) -> Option<SettingsConfiguration> {
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        if !state.pending {
            return None;
        }
        state.pending = false;
        Some(state.desired)
    }

    fn publish(&self, current: SettingsConfiguration) {
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        // A Python request made during Bevy's update must not be lost.
        if !state.pending {
            state.desired = current;
        }
    }
}

/// Blue's original `GlobalSettings` interface, scoped to one Bevy World.
/// The bridge is communication machinery, not a second ECS object model.
#[blue_pyclass(
    module = "destiny",
    name = "GlobalSettings",
    kind = "resource",
    doc = "Use this to configure the Destiny module"
)]
#[derive(Clone, Debug, Default)]
pub struct GlobalSettings {
    #[blue(skip)]
    #[reflect(ignore)]
    bridge: SettingsBridge,
}

impl GlobalSettings {
    /// Connect the Python-visible object to a particular Bevy World.
    pub fn from_world(world: &World) -> Option<Self> {
        world.get_resource::<SettingsBridge>().cloned().map(|bridge| Self { bridge })
    }

    /// C++ `ApplyConfiguration`: last submitted complete configuration wins.
    pub fn apply(&self, config: SettingsConfiguration) {
        CanApplyDestinySettings::apply_destiny_settings(self, config);
    }

    /// C++ `GetConfiguration`: returns an independent configuration value.
    #[must_use]
    pub fn get(&self) -> SettingsConfiguration {
        CanReadDestinySettings::read_destiny_settings(self)
    }

    #[must_use]
    pub fn get_default(&self) -> SettingsConfiguration {
        SettingsConfiguration::DEFAULT
    }

    pub fn reset(&self) {
        self.apply(SettingsConfiguration::DEFAULT);
    }
}

// CGP selects settings operations statically. The providers below implement
// the default per-world Bevy scheduling policy, without introducing a runtime
// executor or overriding Bevy's resource model.
#[cgp_component(DestinySettingsApplier)]
pub trait CanApplyDestinySettings {
    fn apply_destiny_settings(&self, config: SettingsConfiguration);
}

#[cgp_component(DestinySettingsReader)]
pub trait CanReadDestinySettings {
    fn read_destiny_settings(&self) -> SettingsConfiguration;
}

pub struct ApplyStagedSettings;

#[cgp_provider]
impl DestinySettingsApplier<GlobalSettings> for ApplyStagedSettings {
    fn apply_destiny_settings(context: &GlobalSettings, config: SettingsConfiguration) {
        context.bridge.stage(config);
    }
}

pub struct ReadStagedSettings;

#[cgp_provider]
impl DestinySettingsReader<GlobalSettings> for ReadStagedSettings {
    fn read_destiny_settings(context: &GlobalSettings) -> SettingsConfiguration {
        context.bridge.snapshot()
    }
}

delegate_components! {
    GlobalSettings {
        DestinySettingsApplierComponent: ApplyStagedSettings,
        DestinySettingsReaderComponent: ReadStagedSettings,
    }
}

#[blue_methods]
impl GlobalSettings {
    /// Pass in a destiny.SettingsConfiguration to configure global settings.
    #[blue(name = "Apply")]
    fn py_apply(&self, config: PyRef<'_, SettingsConfiguration>) {
        self.apply(*config);
    }

    /// Get the current global settings configuration.
    #[blue(name = "Get")]
    fn py_get(&self) -> SettingsConfiguration {
        self.get()
    }

    /// Get the default global settings configuration.
    #[blue(name = "GetDefault")]
    fn py_get_default(&self) -> SettingsConfiguration {
        self.get_default()
    }

    /// Reset the current global settings configuration to default values.
    #[blue(name = "Reset")]
    fn py_reset(&self) {
        self.reset();
    }
}

/// Register both Blue classes and a world-bound `destiny.settings` Python
/// object in the host application's Python module.
///
/// In contrast with the original C++ global pointer, this registration is
/// explicit and bound to one World, allowing independent simulations.
pub fn register_destiny_settings_python(
    py: Python<'_>,
    module: &Bound<'_, PyModule>,
    world: &World,
) -> PyResult<()> {
    let settings = GlobalSettings::from_world(world)
        .ok_or_else(|| PyRuntimeError::new_err("DestinySettingsPlugin is not installed"))?;
    BlueModule::new(module)
        .class::<SettingsConfiguration>()?
        .class::<GlobalSettings>()?;
    module.add("settings", Py::new(py, settings)?)?;
    Ok(())
}

/// Installs the authoritative Bevy resource and its per-world Python bridge.
/// No locks are held while running simulation systems.
pub struct DestinySettingsPlugin;

impl Plugin for DestinySettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsConfiguration>();
        let initial = *app.world().resource::<SettingsConfiguration>();
        if !app.world().contains_resource::<SettingsBridge>() {
            app.insert_resource(SettingsBridge::new(initial));
        }
        app.register_blue_class::<SettingsConfiguration>()
            .register_blue_class::<GlobalSettings>()
            .add_systems(First, import_python_requests)
            .add_systems(Last, publish_bevy_state);
    }
}

fn import_python_requests(
    mut config: ResMut<'_, SettingsConfiguration>,
    bridge: Res<'_, SettingsBridge>,
) {
    if let Some(request) = bridge.take_pending() {
        if *config != request {
            *config = request;
        }
    }
}

fn publish_bevy_state(
    config: Res<'_, SettingsConfiguration>,
    bridge: Res<'_, SettingsBridge>,
) {
    if config.is_changed() {
        bridge.publish(*config);
    }
}