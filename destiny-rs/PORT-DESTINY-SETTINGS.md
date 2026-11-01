# Settings port notes

Source input: CCP 2023 Settings(1).h, Settings(1).cpp, Settings_Blue(1).cpp.

| C++ | Rust |
|---|---|
| `g_collisionMaxIterationsDefault` | `SettingsConfiguration::DEFAULT.collision_max_iterations = 20` |
| four remaining defaults | `false` in `SettingsConfiguration::DEFAULT` |
| five `extern` globals | `SettingsConfiguration` Bevy Resource per World |
| `GlobalSettings::ApplyConfiguration` | `GlobalSettings::apply`, CGP `CanApplyDestinySettings` |
| `GetConfiguration` | `get`, CGP `CanReadDestinySettings` |
| `GetDefaultConfiguration` | `get_default` |
| `ResetConfiguration` | `reset` |
| `BLUE_CLASS`, `EXPOSE_TO_BLUE` | `#[blue_pyclass]`, `#[blue_methods]` |
| `MAP_ATTRIBUTE` | `#[blue(read, write, name = ...)]` with Blue field docs |
| `BLUE_REGISTER_GLOBAL_AS_MODULE_OBJECT("settings", ...)` | `register_destiny_settings_python(py, module, world)` |

Explicit compatibility difference: the original C++ process-global settings
are now per Bevy World. Python `Apply()` immediately updates its bridge snapshot,
then the ECS sees the change at `First` in the next `App::update()`; direct
Bevy changes publish at `Last`. Python concurrency is safe at the bridge but
neither free-threaded PyO3 module support nor synchronous cross-thread ECS
mutation is implied.
