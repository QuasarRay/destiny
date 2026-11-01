# Settings: original C++ ↔ Rust/Bevy correspondence

**Sources:** `src/Settings.h`, `src/Settings.cpp`, `src/Settings_Blue.cpp`.
**Destination:** `destiny-rs/src/settings.rs`.

## Original state

The source has two types: `SettingsConfiguration` (five fields) and `GlobalSettings` (four member methods). It also defines five process-global variables, five default constants, and one process-global `s_globalSettings` pointer. `ApplyConfiguration` copies five values directly into the globals; `GetConfiguration` constructs a fresh snapshot; `GetDefaultConfiguration` uses the constants; `ResetConfiguration` immediately resets all five values. The Blue module exposes global `settings` and names the methods `Apply`, `Get`, `GetDefault`, and `Reset`.

## Rust representation

- One canonical `SettingsConfiguration` value type with all five original `m_*` names and original types (`size_t` → `usize`, `bool` → `bool`). The Blue wrapper exposes the original read/write names.
- One canonical `GlobalSettings` type with original-named `ApplyConfiguration`, `GetConfiguration`, `GetDefaultConfiguration`, and `ResetConfiguration` methods.
- Five statics named `g_*` and five matching `g_*Default` constants, plus the process-wide `s_globalSettings` controller.
- Original globals are modeled with `AtomicUsize`/`AtomicBool` rather than unsynchronized C++ mutable scalars to prevent data races in safe Rust. These are independent relaxed atomics, matching the C++ five-store sequence in single-threaded observations. C++ unsynchronized concurrent access is undefined behavior, so full concurrency equivalence is neither possible nor claimed.
- One process-global Python singleton object for `destiny.settings`, including across multiple module registrations in the same interpreter. The C++ reference-counted IRoot representation is replaced by PyO3.
- The Bevy `SettingsConfiguration` resource is **only an ECS boundary snapshot**, synchronized from globals in the `First` schedule. It is not a separate authoritative per-world settings state. Direct ECS mutation is overwritten at the next update; the source does not provide independent per-world settings.
- The previous `SettingsBridge`, CGP setting providers and delayed-write systems have been removed: these did not exist in original Settings.cpp and changed its immediately visible behavior.

## Validation

- `python3 -m unittest discover -s tools/tests -p 'test_*.py'` verifies original class/global/field/method names against the C++ headers.
- `cargo test -p destiny-rs --test settings` checks defaults, immediate global writes, independent snapshots, process-wide sharing, Bevy projection and Python exposure.
- `python3 tools/run_settings_differential.py` compiles the **actual original `src/Settings.cpp`** with only the external CCP Blue/Python ABI mocked, and compares direct global state transitions to a native Rust example. The Blue Python method mapping is independently tested using PyO3.

## Remaining exactness boundaries

Python `PyObject*`/`IRoot` lifetime and automatic memory management are represented by safe PyO3 objects rather than literal C++ pointers. The original C++ is not thread safe; Rust's atomic globals define safe behavior for concurrent accesses that C++ leaves undefined. Bevy resource synchronization happens on scheduled updates, whereas C++ code reads the process globals immediately (Rust methods and direct atomic loads also remain immediate). Differential tests cover finite scenarios, but do not prove universal runtime equivalence for every possible Python runtime or concurrent execution.

**Status:** All original Settings declarations have explicit Rust counterparts and original single-threaded global semantics are implemented. Full universal E2E equivalence—including the external Blue ABI and concurrency—is not formally established.
