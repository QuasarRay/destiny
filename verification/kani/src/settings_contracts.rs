// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny tests.
// Concrete scenarios from test_configure.py at ef738406b61ad06bae2f402857e464735ab106a2.
// Each proof starts with the original initial global settings and executes Reset
// from tearDown. Rust values model copies made by Get/GetDefault, not aliases.
use destiny_original_spec::{OriginalGlobalSettings, OriginalSettingsConfiguration};

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_initial_settings_configuration_values_match_default_values
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn initial_settings_configuration_values_match_default_values() {
    let mut settings = OriginalGlobalSettings::default();
    let config = OriginalSettingsConfiguration::default();
    let default_config = settings.get_default();
    // original-assertion: python/destiny/test/test_configure.py:14
    assert!(default_config.collision_max_iterations == config.collision_max_iterations);
    // original-assertion: python/destiny/test/test_configure.py:15
    assert!(default_config.use_iterative_collision == config.use_iterative_collision);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_configure
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn configure() {
    let mut settings = OriginalGlobalSettings::default();
    let mut config = settings.get_default();
    config.collision_max_iterations = 123;
    settings.apply(config);
    let applied_config = settings.get();
    // original-assertion: python/destiny/test/test_configure.py:22
    assert!(123 == applied_config.collision_max_iterations);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_reset
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn reset() {
    let mut settings = OriginalGlobalSettings::default();
    let mut config = settings.get_default();
    config.collision_max_iterations = 123;
    settings.apply(config);
    settings.reset();
    let applied_config = settings.get();
    let default_config = settings.get_default();
    // original-assertion: python/destiny/test/test_configure.py:31
    assert!(default_config.collision_max_iterations == applied_config.collision_max_iterations);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_enable_dynamical_orientation
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn enable_dynamical_orientation() {
    let mut settings = OriginalGlobalSettings::default();
    let mut config = settings.get_default();
    // original-assertion: python/destiny/test/test_configure.py:35
    assert!(!config.use_dynamical_orientation);
    config.use_dynamical_orientation = true;
    settings.apply(config);
    let applied_config = settings.get();
    // original-assertion: python/destiny/test/test_configure.py:39
    assert!(applied_config.use_dynamical_orientation);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_disable_dynamical_orientation_for_missiles
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn disable_dynamical_orientation_for_missiles() {
    let mut settings = OriginalGlobalSettings::default();
    let mut config = settings.get_default();
    // original-assertion: python/destiny/test/test_configure.py:43
    assert!(!config.disable_dynamical_orientation_for_missiles);
    config.disable_dynamical_orientation_for_missiles = true;
    settings.apply(config);
    let applied_config = settings.get();
    // original-assertion: python/destiny/test/test_configure.py:47
    assert!(applied_config.disable_dynamical_orientation_for_missiles);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}

// original-case: python/destiny/test/test_configure.py::TestConfigure::test_enable_new_orbit
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn enable_new_orbit() {
    let mut settings = OriginalGlobalSettings::default();
    let mut config = settings.get_default();
    // original-assertion: python/destiny/test/test_configure.py:51
    assert!(!config.use_new_orbit);
    config.use_new_orbit = true;
    settings.apply(config);
    let applied_config = settings.get();
    // original-assertion: python/destiny/test/test_configure.py:55
    assert!(applied_config.use_new_orbit);
    settings.reset();
    #[cfg(kani)]
    kani::cover!(true, "complete original settings scenario and teardown");
}
