// Matches the cases emitted by tools/cpp_settings_oracle.cpp. The C++
// arithmetic and side effects come from the ACTUAL src/Settings.cpp.
use destiny_rs::{SettingsConfiguration, s_globalSettings};

fn output(label: &str, c: SettingsConfiguration) {
    println!("{} {} {} {} {} {}",
        label, c.m_collisionMaxIterations,
        c.m_useIterativeCollision as u8, c.m_useDynamicalOrientation as u8,
        c.m_disableDynamicalOrientationForMissiles as u8, c.m_useNewOrbit as u8);
}
fn main() {
    s_globalSettings.ResetConfiguration();
    output("initial", s_globalSettings.GetConfiguration());
    let desired = SettingsConfiguration {
        m_collisionMaxIterations: 5,
        m_useIterativeCollision: true,
        m_useDynamicalOrientation: true,
        m_disableDynamicalOrientationForMissiles: true,
        m_useNewOrbit: true,
    };
    s_globalSettings.ApplyConfiguration(&desired);
    output("immediate", s_globalSettings.GetConfiguration());
    let mut snapshot = s_globalSettings.GetConfiguration();
    let other = s_globalSettings.GetConfiguration();
    assert_eq!(snapshot, other);
    output("get", snapshot);
    snapshot.m_collisionMaxIterations = 999;
    output("snapshot_independence", s_globalSettings.GetConfiguration());
    output("get_default", s_globalSettings.GetDefaultConfiguration());
    // Wrong-type Python argument is exercised by the Python Blue test, not
    // by this native Rust example, because the Rust method is strongly typed.
    output("invalid_argument_unchanged", s_globalSettings.GetConfiguration());
    s_globalSettings.ResetConfiguration();
    output("reset", s_globalSettings.GetConfiguration());
}
