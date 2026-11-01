# Blue-rs: CGP + derive-deftly + Bevy 0.20.1

This is an **incremental architectural refactor**, not a complete rewrite of every subsystem.

## Source-level ownership

- **Bevy 0.20.1** remains authoritative for `App`, `World`, ECS resources,
  components, reflection, scheduling, and assets. No shadow ECS is introduced.
- **CGP 0.8.0-alpha** creates statically selectable behavior providers:
  `CanTickBlueRuntime` / `BlueRuntimeTicker`,
  `CanAddBlueResource` / `BlueResourceAdder`, and
  `CanWriteBlueMetadata` / `BlueMetadataWriter`.
- **derive-deftly 1.14.0** provides one reusable `BlueMetadataShape` template
  to emit the Rust schema for all Blue metadata structs and the registry.
- **PyO3 0.30.1**, the existing `blue_*` procedural macros, and **Genco** remain.
  Their responsibilities are not duplicated by the new derive template.

## CGP injection points

`BlueRuntime::update()` and `BlueRuntime::add_resource()` delegate to CGP-selected providers. The default
provider calls `App::update()`. Consumers still invoke the original API.
`BlueRegistry::register_*()` delegates metadata upserts to the statically
selected provider, preserving the qualified-name overwrite behavior.

The code uses the *actual* CGP 0.8 macro idioms (`#[cgp_component]`,
`#[cgp_provider]`, `delegate_components!`) from the supplied source snapshot.
For a new provider, implement the generated provider trait and wire it to a
new context type (a second `BlueRegistry` impl is not possible in Rust).

## Deliberate compatibility constraints

- Do not replace Bevy `DVec3`, `DQuat`, ECS, or reflection with wrappers.
- Preserve the current exposed names (`BlueRuntime`, `BlueRegistry`,
  `BluePlugin`, `BlueExposure`, and existing Python metadata).
- `PyBlueRuntime` is *still* `unsendable` and the module still declares
  `gil_used(true)`. This refactor **does not** claim free-threaded Python
  compatibility. Reworking interpreter/world scheduling needs separate design.
- The Blue macro proc-macro system still owns PyO3 and Bevy class derives.
  Derive-deftly here only generates strongly typed structural metadata.
- The uploaded CGP book is a DRAFT and contains macro syntax predating
  some 0.8 capabilities; source-verified 0.8 syntax takes precedence.

## Build matrix to run with a Rust toolchain

```
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo tree -d
```

No `cargo` / `rustc` executable was available in the assembly environment;
**these checks have not run** and the generated artifacts are not certified
as compiling. Start by validating the CGP 0.8 macro expansions and
`derive-deftly` template compatibility.

## Next refactor slices

1. Extract the `blue-macros` metadata naming rules into shared declarative
   schema; generate both Blue runtime descriptors and Python registration
   from it (keep a single source of truth).
2. Introduce CGP-selected Python dispatch and asynchronous command queues,
   with per-world ownership and no hidden `Arc<Mutex<World>>`.
3. Keep reflected object IDs and persistence contracts stable while migrating
   callback/runtime policies behind CGP components.
4. Add compile-time provider checks, fixture parity and Kani harnesses.
5. Run compilation, Rust/Python integration tests, and differential tests
   before replacing the production Blue path.

## Compatibility matrix

| Contract | Status | Required parity check |
|---|---|---|
| `BlueRuntime` / `World` ownership | Unchanged | Existing runtime tests |
| `BlueRegistry` and qualified names | Unchanged | Existing registration tests, CGP writer tests |
| `BlueClock`, lifecycle, assets | Unchanged API, CGP provider behind tick/assets | Bevy application test |
| `#[blue_pyclass]` / `#[blue_methods]` | Retained | Existing macro tests |
| Python runtime threading | Not migrated | Free-threaded Python tests still required |
| Bevy version | `0.20.1` throughout | Cargo check and lockfile resolution |
| CGP wiring | `0.8.0-alpha` | `cgp_metadata` tests |
| derive-deftly schema | `1.14.0` | `cgp_metadata` tests |

The original C++ Blue implementation, Python test suite, and Rust Kani harnesses
were not provided as complete runnable baselines in this request. Existing
source files were preserved, but behavioral E2E equivalence is **not** established.
