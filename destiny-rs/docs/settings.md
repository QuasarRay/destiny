# Destiny Settings — Bevy 0.20.1 + CGP Blue-rs

Ports CCP's `Settings.h`, `Settings.cpp`, `Settings_Blue.cpp` into the unified Destiny-rs crate. Maintains **native Bevy ECS Resource** `SettingsConfiguration`
and **Blue/PyO3** class metadata; no recreated vector, quaternion, ECS or Python
interoperability framework.

## Architecture

- **Authoritative state:** one `SettingsConfiguration` in each Bevy World.
- **CGP provider selection:** `CanApplyDestinySettings` and `CanReadDestinySettings`
  delegate to `ApplyStagedSettings` and `ReadStagedSettings`.
- **Blue exposure:** `#[blue_pyclass]`, `#[blue_methods]` and `BlueModule` from the
  current Blue workspace. `BlueClassInfo` metadata contains original field names
  and descriptions; macro rename support is a backwards-compatible Blue change.
- **Python boundary:** `GlobalSettings` holds a cloned per-world `SettingsBridge`
  (short-lived `Mutex`) for thread-safe staging. `First` commits requests; `Last`
  publishes native Bevy state. Bevy owns the canonical resource.

No global `OnceLock`, no implicit Python GIL as a Rust-side synchronization
mechanism, no assumption that a Bevy `World` is safe to share between threads.
The existing Blue `_bluepycore` module still uses `gil_used(true)` and its
`Runtime` is `unsendable`: this port alone does **not** make the entire runtime
free-threaded or asynchronous. Python async callers may call short nonblocking
`Apply`/`Get` methods; ECS changes are visible only at update boundaries.

## Using it

```rust
use blue::BlueRuntime;
use destiny_settings::{DestinySettingsPlugin, SettingsConfiguration};
let mut runtime = BlueRuntime::new();
runtime.app_mut().add_plugins(DestinySettingsPlugin);
assert_eq!(runtime.world().resource::<SettingsConfiguration>().collision_max_iterations, 20);
```

For Python, the host **must register the object against its actual Bevy World**:

```rust
use destiny_settings::register_destiny_settings_python;
use pyo3::{prelude::*, types::PyModule};
Python::attach(|py| -> PyResult<()> {
    let module = PyModule::new(py, "destiny")?;
    register_destiny_settings_python(py, &module, runtime.world())?;
    // Mount/register `module` in the host interpreter; do not create a second
    // unrelated Python global instance.
    Ok(())
})?;
```

Python API:

```python
config = destiny.SettingsConfiguration()
config.collisionMaxIterations = 5  # legal, but discouraged in source docs
destiny.settings.Apply(config)
assert destiny.settings.Get().collisionMaxIterations == 5
# After App.update(): the Bevy SettingsConfiguration resource is updated.
destiny.settings.Reset()
```

## Verification

```bash
cargo test -p destiny-rs
cargo test --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

**Not executed** in the packaging environment (Rust toolchain unavailable).
Configuration values and APIs are source-ported; proof of C++/Rust behavioral
parity and complete Blue/PyO3 integration is pending compilation and tests.
