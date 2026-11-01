use pyo3::types::PyAnyMethods;
use bevy::prelude::*;
use blueexposure::{BlueClass, BlueRegistry};
use destiny_rs::{
    DestinySettingsPlugin, GlobalSettings, SettingsConfiguration,
    register_destiny_settings_python,
};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(DestinySettingsPlugin);
    app
}

#[test]
fn original_defaults_are_preserved() {
    let config = SettingsConfiguration::default();
    assert_eq!(config.collision_max_iterations, 20);
    assert!(!config.use_iterative_collision);
    assert!(!config.use_dynamical_orientation);
    assert!(!config.disable_dynamical_orientation_for_missiles);
    assert!(!config.use_new_orbit);
}

#[test]
fn apply_get_getdefault_reset() {
    let mut app = app();
    let settings = GlobalSettings::from_world(app.world()).unwrap();
    let desired = SettingsConfiguration {
        collision_max_iterations: 5, // documented as discouraged, NOT invalid
        use_iterative_collision: true,
        use_dynamical_orientation: true,
        disable_dynamical_orientation_for_missiles: true,
        use_new_orbit: true,
    };
    settings.apply(desired);
    assert_eq!(settings.get(), desired);
    assert_eq!(*app.world().resource::<SettingsConfiguration>(), SettingsConfiguration::DEFAULT);
    app.update();
    assert_eq!(*app.world().resource::<SettingsConfiguration>(), desired);
    settings.reset();
    assert_eq!(settings.get(), SettingsConfiguration::DEFAULT);
    app.update();
    assert_eq!(*app.world().resource::<SettingsConfiguration>(), SettingsConfiguration::DEFAULT);
    assert_eq!(settings.get_default(), SettingsConfiguration::DEFAULT);
}

#[test]
fn direct_bevy_changes_publish_to_blue() {
    let mut app = app();
    let settings = GlobalSettings::from_world(app.world()).unwrap();
    app.world_mut().resource_mut::<SettingsConfiguration>().use_new_orbit = true;
    app.update();
    assert!(settings.get().use_new_orbit);
}

#[test]
fn two_worlds_have_independent_configuration() {
    let mut a = app();
    let mut b = app();
    let alpha = GlobalSettings::from_world(a.world()).unwrap();
    let beta = GlobalSettings::from_world(b.world()).unwrap();
    alpha.apply(SettingsConfiguration { use_new_orbit: true, ..Default::default() });
    a.update();
    b.update();
    assert!(a.world().resource::<SettingsConfiguration>().use_new_orbit);
    assert!(!b.world().resource::<SettingsConfiguration>().use_new_orbit);
    assert!(!beta.get().use_new_orbit);
}

#[test]
fn preconfigured_bevy_resource_is_used() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SettingsConfiguration { collision_max_iterations: 71, ..Default::default() });
    app.add_plugins(DestinySettingsPlugin);
    assert_eq!(GlobalSettings::from_world(app.world()).unwrap().get().collision_max_iterations, 71);
}

#[test]
fn blue_metadata_and_legacy_python_field_names_match() {
    let app = app();
    let registry = app.world().resource::<BlueRegistry>();
    let metadata = registry.class("destiny.SettingsConfiguration").unwrap();
    assert_eq!(metadata.fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), vec![
        "collisionMaxIterations", "useIterativeCollision", "useDynamicalOrientation",
        "disableDynamicalOrientationForMissiles", "useNewOrbit",
    ]);
    assert!(metadata.fields.iter().all(|f| f.readable && f.writable));
    assert!(metadata.fields[0].doc.contains("Recommended value >= 10"));
    assert_eq!(SettingsConfiguration::BLUE_MODULE, "destiny");
    assert!(registry.class("destiny.GlobalSettings").is_some());
}

#[test]
fn concurrent_writers_submit_without_borrowing_world() {
    let mut app = app();
    let settings = GlobalSettings::from_world(app.world()).unwrap();
    let handles: Vec<_> = (0..8).map(|n| {
        let settings = settings.clone();
        std::thread::spawn(move || {
            for _ in 0..50 {
                // The bridge itself is deliberately not an ECS World handle.
                settings.apply(SettingsConfiguration { collision_max_iterations: n, ..Default::default() });
                let _ = settings.get();
            }
        })
    }).collect();
    for handle in handles { handle.join().unwrap(); }
    let snapshot = settings.get();
    app.update();
    assert_eq!(*app.world().resource::<SettingsConfiguration>(), snapshot);
}

#[test]
fn python_blue_module_round_trip() {
    let mut app = app();
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| -> pyo3::PyResult<()> {
        let module = pyo3::types::PyModule::new(py, "destiny")?;
        register_destiny_settings_python(py, &module, app.world())?;
        let config = module.getattr("SettingsConfiguration")?.call0()?;
        config.setattr("collisionMaxIterations", 9)?;
        config.setattr("useNewOrbit", true)?;
        let settings = module.getattr("settings")?;
        settings.call_method1("Apply", (config,))?;
        let snapshot = settings.call_method0("Get")?;
        assert_eq!(snapshot.getattr("collisionMaxIterations")?.extract::<usize>()?, 9);
        assert_eq!(snapshot.getattr("useNewOrbit")?.extract::<bool>()?, true);
        settings.call_method0("Reset")?;
        assert_eq!(settings.call_method0("Get")?.getattr("collisionMaxIterations")?.extract::<usize>()?, 20);
        Ok(())
    }).unwrap();
    app.update();
    assert_eq!(*app.world().resource::<SettingsConfiguration>(), SettingsConfiguration::DEFAULT);
}

#[test]
fn cgp_component_providers_are_wired_to_settings_context() {
    use destiny_settings::{CanApplyDestinySettings, CanReadDestinySettings};
    fn assert_contract<T: CanApplyDestinySettings + CanReadDestinySettings>() {}
    assert_contract::<GlobalSettings>();
}

#[test]
fn settings_plugin_integrates_with_embedded_blue_runtime() {
    let mut runtime = blue::BlueRuntime::new();
    runtime.app_mut().add_plugins(DestinySettingsPlugin);
    let controller = GlobalSettings::from_world(runtime.world()).unwrap();
    controller.apply(SettingsConfiguration {
        use_new_orbit: true,
        ..Default::default()
    });
    runtime.update();
    assert!(runtime.world().resource::<SettingsConfiguration>().use_new_orbit);
}
