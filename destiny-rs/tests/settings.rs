use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::Ordering;
use bevy::prelude::*;
use blueexposure::{BlueClass, BlueRegistry};
use destiny_rs::{
    DestinySettingsPlugin, GlobalSettings, SettingsConfiguration, s_globalSettings,
    g_collisionMaxIterations, g_useIterativeCollision, g_useDynamicalOrientation,
    g_disableDynamicalOrientationForMissiles, g_useNewOrbit,
    register_destiny_settings_python,
};

// The original settings are process-global; these tests must serialize their
// mutations rather than relying on the old per-World isolation.
static TEST_LOCK: Mutex<()> = Mutex::new(());
fn locked() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
    s_globalSettings.ResetConfiguration();
    guard
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(DestinySettingsPlugin);
    app
}
fn custom() -> SettingsConfiguration {
    SettingsConfiguration {
        m_collisionMaxIterations: 5,
        m_useIterativeCollision: true,
        m_useDynamicalOrientation: true,
        m_disableDynamicalOrientationForMissiles: true,
        m_useNewOrbit: true,
    }
}

#[test]
fn source_configuration_defaults_are_preserved() {
    let _guard = locked();
    assert_eq!(SettingsConfiguration::default(), SettingsConfiguration::DEFAULT);
    assert_eq!(s_globalSettings.GetConfiguration(), SettingsConfiguration::DEFAULT);
    assert_eq!(s_globalSettings.GetDefaultConfiguration(), SettingsConfiguration::DEFAULT);
    assert_eq!(SettingsConfiguration::DEFAULT.m_collisionMaxIterations, 20);
    assert!(std::mem::size_of::<SettingsConfiguration>() >=
        std::mem::size_of::<usize>() + 4);
}

#[test]
fn four_original_cplusplus_methods_act_immediately_with_no_bevy_update() {
    let _guard = locked();
    let settings = &s_globalSettings;
    let desired = custom(); // 5 is discouraged, not rejected.
    settings.ApplyConfiguration(&desired);
    assert_eq!(settings.GetConfiguration(), desired);
    assert_eq!(settings.GetDefaultConfiguration(), SettingsConfiguration::DEFAULT);
    assert_eq!(g_collisionMaxIterations.load(Ordering::Relaxed), 5);
    assert!(g_useIterativeCollision.load(Ordering::Relaxed));
    assert!(g_useDynamicalOrientation.load(Ordering::Relaxed));
    assert!(g_disableDynamicalOrientationForMissiles.load(Ordering::Relaxed));
    assert!(g_useNewOrbit.load(Ordering::Relaxed));
    settings.ResetConfiguration();
    assert_eq!(settings.GetConfiguration(), SettingsConfiguration::DEFAULT);
}

#[test]
fn get_configuration_returns_independent_new_value_not_live_reference() {
    let _guard = locked();
    s_globalSettings.ApplyConfiguration(&custom());
    let mut snapshot = s_globalSettings.GetConfiguration();
    snapshot.m_collisionMaxIterations = 901;
    snapshot.m_useNewOrbit = false;
    assert_eq!(s_globalSettings.GetConfiguration(), custom());
    s_globalSettings.ResetConfiguration();
}

#[test]
fn every_bevy_world_observes_same_global_configuration() {
    let _guard = locked();
    let mut a = app();
    let mut b = app();
    s_globalSettings.ApplyConfiguration(&custom());
    // Unlike the old staged bridge, globals change before either update.
    assert_eq!(s_globalSettings.GetConfiguration(), custom());
    a.update();
    b.update();
    assert_eq!(*a.world().resource::<SettingsConfiguration>(), custom());
    assert_eq!(*b.world().resource::<SettingsConfiguration>(), custom());
    s_globalSettings.ResetConfiguration();
    a.update();
    b.update();
    assert_eq!(*a.world().resource::<SettingsConfiguration>(), SettingsConfiguration::DEFAULT);
    assert_eq!(*b.world().resource::<SettingsConfiguration>(), SettingsConfiguration::DEFAULT);
}

#[test]
fn bevy_resource_is_boundary_mirror_not_another_settings_authority() {
    let _guard = locked();
    let mut a = app();
    a.world_mut().resource_mut::<SettingsConfiguration>().m_useNewOrbit = true;
    assert_eq!(s_globalSettings.GetConfiguration(), SettingsConfiguration::DEFAULT);
    a.update();
    assert!(!a.world().resource::<SettingsConfiguration>().m_useNewOrbit);
}

#[test]
fn plugin_initializes_from_current_process_globals() {
    let _guard = locked();
    s_globalSettings.ApplyConfiguration(&custom());
    let a = app();
    assert_eq!(*a.world().resource::<SettingsConfiguration>(), custom());
    s_globalSettings.ResetConfiguration();
}

#[test]
fn blue_metadata_keeps_cplusplus_names_and_readwrite_attributes() {
    let _guard = locked();
    let a = app();
    let registry = a.world().resource::<BlueRegistry>();
    let metadata = registry.class("destiny.SettingsConfiguration").unwrap();
    assert_eq!(
        metadata.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        vec!["collisionMaxIterations", "useIterativeCollision",
             "useDynamicalOrientation", "disableDynamicalOrientationForMissiles", "useNewOrbit"]
    );
    assert!(metadata.fields.iter().all(|f| f.readable && f.writable));
    assert_eq!(SettingsConfiguration::BLUE_MODULE, "destiny");
    assert!(registry.class("destiny.GlobalSettings").is_some());
}

#[test]
fn python_blue_methods_and_invalid_argument_match_cpp_contract() {
    let _guard = locked();
    let a = app();
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
        use pyo3::types::PyAnyMethods;
        let module = pyo3::types::PyModule::new(py, "destiny")?;
        register_destiny_settings_python(py, &module, a.world())?;
        let config = module.getattr("SettingsConfiguration")?.call0()?;
        config.setattr("collisionMaxIterations", 9)?;
        config.setattr("useNewOrbit", true)?;
        let settings = module.getattr("settings")?;
        settings.call_method1("Apply", (config,))?;
        let snapshot = settings.call_method0("Get")?;
        assert_eq!(snapshot.getattr("collisionMaxIterations")?.extract::<usize>()?, 9);
        assert_eq!(snapshot.getattr("useNewOrbit")?.extract::<bool>()?, true);
        let default = settings.call_method0("GetDefault")?;
        assert_eq!(default.getattr("collisionMaxIterations")?.extract::<usize>()?, 20);
        assert!(!default.getattr("useNewOrbit")?.extract::<bool>()?);
        let error = settings.call_method1("Apply", (42,)).unwrap_err();
        assert!(error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        assert!(error.to_string().contains("Argument 1 is not a SettingsConfiguration"));
        settings.call_method0("Reset")?;
        assert_eq!(settings.call_method0("Get")?.getattr("collisionMaxIterations")?.extract::<usize>()?, 20);
        Ok(())
    }).unwrap();
}

#[test]
fn python_module_registration_reuses_one_process_global_settings_object() {
    let _guard = locked();
    let a = app();
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
        use pyo3::types::PyAnyMethods;
        let first = pyo3::types::PyModule::new(py, "destiny_a")?;
        let second = pyo3::types::PyModule::new(py, "destiny_b")?;
        register_destiny_settings_python(py, &first, a.world())?;
        register_destiny_settings_python(py, &second, a.world())?;
        assert!(first.getattr("settings")?.is(&second.getattr("settings")?));
        Ok(())
    }).unwrap();
}

#[test]
fn original_global_object_is_available_without_bevy_runtime() {
    let _guard = locked();
    let singleton: &GlobalSettings = &s_globalSettings;
    singleton.ApplyConfiguration(&custom());
    assert_eq!(singleton.GetConfiguration(), custom());
}

#[test]
fn embedded_blue_runtime_observes_global_settings() {
    let _guard = locked();
    let mut runtime = blue::BlueRuntime::new();
    runtime.app_mut().add_plugins(DestinySettingsPlugin);
    s_globalSettings.ApplyConfiguration(&custom());
    runtime.update();
    assert_eq!(*runtime.world().resource::<SettingsConfiguration>(), custom());
}
