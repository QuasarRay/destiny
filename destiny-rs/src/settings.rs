// Copyright © 2023 CCP ehf. Original Settings.h / Settings.cpp /
// Settings_Blue.cpp.
//
// One canonical SettingsConfiguration and one GlobalSettings; the five
// process-global settings retain original names and are authoritative.
// Bevy's Resource is a read-only-to-Destiny mirror at the ECS boundary.
// Atomics are a Rust thread-safety translation of C++ ordinary globals.
// Each field is read/written independently, exactly as in Settings.cpp.

use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use bevy::ecs::reflect::ReflectResource;
use bevy::prelude::{App, First, Plugin, ResMut, World};
use blueexposure::{blue_methods, blue_pyclass, BlueAppExt, BlueModule};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyAny, PyModuleMethods},
};

// The exact five original C++ default constant names and values.
#[allow(non_upper_case_globals)]
pub const g_collisionMaxIterationsDefault: usize = 20;
#[allow(non_upper_case_globals)]
pub const g_useIterativeCollisionDefault: bool = false;
#[allow(non_upper_case_globals)]
pub const g_useDynamicalOrientationDefault: bool = false;
#[allow(non_upper_case_globals)]
pub const g_disableDynamicalOrientationForMissilesDefault: bool = false;
#[allow(non_upper_case_globals)]
pub const g_useNewOrbitDefault: bool = false;

// The five original process-global variables. Rust atomics prevent undefined
// concurrent data races, while retaining one global location per C++ variable.
// Atomic relaxed loads/stores do not provide a five-field transaction; C++
// performs five independent assignments and likewise provides no atomic
// multi-field snapshot.
#[allow(non_upper_case_globals)]
pub static g_collisionMaxIterations: AtomicUsize =
    AtomicUsize::new(g_collisionMaxIterationsDefault);
#[allow(non_upper_case_globals)]
pub static g_useIterativeCollision: AtomicBool =
    AtomicBool::new(g_useIterativeCollisionDefault);
#[allow(non_upper_case_globals)]
pub static g_useDynamicalOrientation: AtomicBool =
    AtomicBool::new(g_useDynamicalOrientationDefault);
#[allow(non_upper_case_globals)]
pub static g_disableDynamicalOrientationForMissiles: AtomicBool =
    AtomicBool::new(g_disableDynamicalOrientationForMissilesDefault);
#[allow(non_upper_case_globals)]
pub static g_useNewOrbit: AtomicBool =
    AtomicBool::new(g_useNewOrbitDefault);

/// Original SettingsConfiguration, preserving all five member field names,
/// their types, order, Python/Blue names, mutability, and constructor defaults.
///
/// The Bevy Resource derive created by blue_pyclass(kind = "resource") is a
/// view into the ECS boundary; it is not the authoritative settings store.
#[blue_pyclass(
    module = "destiny",
    name = "SettingsConfiguration",
    kind = "resource",
    doc = "Pass an instance of this to destiny.settings.Configure() to apply a settings configuration"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct SettingsConfiguration {
    /// The maximum number of collision iterations (assuming iterative collision is enabled). Recommended value >= 10, at 5 agile ships are known to fly through thin boxes.
    #[blue(read, write, name = "collisionMaxIterations")]
    pub m_collisionMaxIterations: usize,

    /// Boolean flag to enable/disable iterative collision
    #[blue(read, write, name = "useIterativeCollision")]
    pub m_useIterativeCollision: bool,

    /// Boolean flag to enable/disable dynamical orientation
    #[blue(read, write, name = "useDynamicalOrientation")]
    pub m_useDynamicalOrientation: bool,

    /// Boolean flag to disable dynamical orientation for missiles (if dynamical orientation is enabled).
    #[blue(read, write, name = "disableDynamicalOrientationForMissiles")]
    pub m_disableDynamicalOrientationForMissiles: bool,

    /// Use the new orbit behavior
    #[blue(read, write, name = "useNewOrbit")]
    pub m_useNewOrbit: bool,
}

impl SettingsConfiguration {
    pub const DEFAULT: Self = Self {
        m_collisionMaxIterations: g_collisionMaxIterationsDefault,
        m_useIterativeCollision: g_useIterativeCollisionDefault,
        m_useDynamicalOrientation: g_useDynamicalOrientationDefault,
        m_disableDynamicalOrientationForMissiles:
            g_disableDynamicalOrientationForMissilesDefault,
        m_useNewOrbit: g_useNewOrbitDefault,
    };
}

impl Default for SettingsConfiguration {
    fn default() -> Self { Self::DEFAULT }
}

#[blue_methods]
impl SettingsConfiguration {
    #[blue(new)]
    fn py_new() -> Self { Self::default() }
}

/// Original C++ GlobalSettings class has no settings data fields.
///
/// Its IRoot/Blue exposure is implemented through PyO3/BlueExposure;
/// the Bevy ECS uses it only as the singleton controller of C++ globals.
#[blue_pyclass(
    module = "destiny",
    name = "GlobalSettings",
    kind = "resource",
    doc = "Use this to configure the destiny module"
)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GlobalSettings;

// C++: extern GlobalSettings* s_globalSettings = new OGlobalSettings();
// The single zero-field Rust controller has static process lifetime.
// The Python object identity is separately cached in PY_GLOBAL_SETTINGS.
#[allow(non_upper_case_globals)]
pub static s_globalSettings: GlobalSettings = GlobalSettings;

impl GlobalSettings {
    /// Original GlobalSettings::ApplyConfiguration: five sequential assignments.
    /// The change is immediately visible via GetConfiguration and the globals;
    /// there is no queued command and no intervening Bevy update required.
    #[allow(non_snake_case)]
    pub fn ApplyConfiguration(&self, config: &SettingsConfiguration) {
        g_collisionMaxIterations.store(config.m_collisionMaxIterations, Ordering::Relaxed);
        g_useIterativeCollision.store(config.m_useIterativeCollision, Ordering::Relaxed);
        g_useDynamicalOrientation.store(config.m_useDynamicalOrientation, Ordering::Relaxed);
        g_disableDynamicalOrientationForMissiles.store(
            config.m_disableDynamicalOrientationForMissiles, Ordering::Relaxed
        );
        g_useNewOrbit.store(config.m_useNewOrbit, Ordering::Relaxed);
    }

    /// Original GlobalSettings::GetConfiguration: independent value snapshot
    /// freshly populated from the five original process-global settings.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn GetConfiguration(&self) -> SettingsConfiguration {
        SettingsConfiguration {
            m_collisionMaxIterations: g_collisionMaxIterations.load(Ordering::Relaxed),
            m_useIterativeCollision: g_useIterativeCollision.load(Ordering::Relaxed),
            m_useDynamicalOrientation: g_useDynamicalOrientation.load(Ordering::Relaxed),
            m_disableDynamicalOrientationForMissiles:
                g_disableDynamicalOrientationForMissiles.load(Ordering::Relaxed),
            m_useNewOrbit: g_useNewOrbit.load(Ordering::Relaxed),
        }
    }

    /// Original GlobalSettings::GetDefaultConfiguration reads the five C++
    /// defaults, independently of the current globals.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn GetDefaultConfiguration(&self) -> SettingsConfiguration {
        SettingsConfiguration::DEFAULT
    }

    /// Original GlobalSettings::ResetConfiguration: reset all five globals
    /// immediately and in declaration order.
    #[allow(non_snake_case)]
    pub fn ResetConfiguration(&self) {
        g_collisionMaxIterations.store(g_collisionMaxIterationsDefault, Ordering::Relaxed);
        g_useIterativeCollision.store(g_useIterativeCollisionDefault, Ordering::Relaxed);
        g_useDynamicalOrientation.store(g_useDynamicalOrientationDefault, Ordering::Relaxed);
        g_disableDynamicalOrientationForMissiles.store(
            g_disableDynamicalOrientationForMissilesDefault, Ordering::Relaxed
        );
        g_useNewOrbit.store(g_useNewOrbitDefault, Ordering::Relaxed);
    }
}

// Original Settings_Blue.cpp maps the four C++ method names to these exact
// Python names, without introducing different authoritative methods.
#[blue_methods]
impl GlobalSettings {
    /// Pass in an instance of destiny.SettingsConfiguration to configure global settings.
    #[blue(name = "Apply")]
    fn py_apply(&self, config: &Bound<'_, PyAny>) -> PyResult<()> {
        let config = config.extract::<PyRef<'_, SettingsConfiguration>>()
            .map_err(|_| PyTypeError::new_err(
                "Argument 1 is not a SettingsConfiguration"
            ))?;
        self.ApplyConfiguration(&config);
        Ok(())
    }

    /// Get the current global settings configuration.
    #[blue(name = "Get")]
    fn py_get(&self) -> SettingsConfiguration {
        self.GetConfiguration()
    }

    /// Get the default global settings configuration.
    #[blue(name = "GetDefault")]
    fn py_get_default(&self) -> SettingsConfiguration {
        self.GetDefaultConfiguration()
    }

    /// Reset the current global settings configuration to the default values.
    #[blue(name = "Reset")]
    fn py_reset(&self) {
        self.ResetConfiguration();
    }
}

// Unlike the old implementation, this cache is process-global. Registering
// in two modules exposes the SAME Python settings object as the C++ global
// s_globalSettings, rather than a distinct per-Bevy-World controller.
static PY_GLOBAL_SETTINGS: OnceLock<Py<GlobalSettings>> = OnceLock::new();

/// Blue module registration as in Settings_Blue.cpp. The World parameter is
/// retained solely as the host-integration boundary; settings remain global.
pub fn register_destiny_settings_python(
    py: Python<'_>,
    module: &Bound<'_, pyo3::types::PyModule>,
    _world: &World,
) -> PyResult<()> {
    BlueModule::new(module)
        .class::<SettingsConfiguration>()?
        .class::<GlobalSettings>()?;
    if PY_GLOBAL_SETTINGS.get().is_none() {
        let fresh = Py::new(py, s_globalSettings)?;
        let _ = PY_GLOBAL_SETTINGS.set(fresh);
    }
    let shared = PY_GLOBAL_SETTINGS.get()
        .expect("settings Python singleton initialized above")
        .clone_ref(py);
    module.add("settings", shared)?;
    Ok(())
}

/// The ONLY Bevy-specific settings machinery. The original five globals
/// remain authoritative and are imported into each World as a *snapshot*.
/// Bevy resource changes do not write globals back or create per-world
/// independent settings, since neither exists in the original C++ source.
pub struct DestinySettingsPlugin;

impl Plugin for DestinySettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsConfiguration>()
            .register_blue_class::<SettingsConfiguration>()
            .register_blue_class::<GlobalSettings>()
            .add_systems(First, synchronize_global_settings_to_bevy);
        // Plugin installation must immediately match the original globals.
        let current = s_globalSettings.GetConfiguration();
        *app.world_mut().resource_mut::<SettingsConfiguration>() = current;
    }
}

fn synchronize_global_settings_to_bevy(
    mut snapshot: ResMut<'_, SettingsConfiguration>,
) {
    let current = s_globalSettings.GetConfiguration();
    if *snapshot != current {
        *snapshot = current;
    }
}
