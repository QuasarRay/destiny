# Blue / Blueexposure Rust port

This tree is an additive Rust implementation of Blue and Blueexposure. The original C++ and Python sources remain the behavioral oracle during migration.

## Architecture

- `crates/blue`: Bevy 0.20 runtime kernel for Blue lifecycle, startup arguments, paths, resources, object metadata, callbacks, and timing.
- `crates/blueexposure`: PyO3 0.30 exposure registry integrated directly with Bevy reflection and `App` registration.
- `crates/blue-macros`: convention-heavy derive/attribute macros. Macro parsing and expansion logic uses `proc_macro2`; the actual procedural-macro entry points remain in the dedicated proc-macro crate because Rust does not permit a normal crate to define and consume procedural macros as though they were ordinary macros.
- `crates/blue-codegen`: Doxygen XML + source inventory reader and Genco generator. It inventories legacy tests, discovers native `#[blue_test]` functions, emits compatibility metadata, and generates Kani integration-test targets.
- `crates/bluepycore`: native Python extension module. It owns a Bevy `App` and exposes Blue-compatible lifecycle entry points through PyO3.

The integration is intentionally Bevy-centric. `BlueRuntime` exposes Bevy's `World`; `BluePlugins` fixes the default plugin order; Blue resources are Bevy resources/assets; exposed Blue classes participate in Bevy reflection instead of being hidden behind a neutral compatibility runtime.

## Verification rule

Every native Rust verification test uses `#[blue_test]`.

`#[blue_test]` generates one shared semantic body plus the ordinary Rust `#[test]` wrapper. `blue-codegen` then discovers every such function and Genco writes `crates/<crate>/tests/blue_kani_generated.rs`, containing a corresponding `#[kani::proof]` wrapper that calls the same semantic body. Kani bodies are therefore generated, not maintained manually.

Legacy test migration is catalog-driven. Doxygen/source discovery inventories the original C++ and Python tests. A legacy test is only marked mapped when `rust-port/manifests/test-map.toml` names a concrete Rust semantic test. Unmapped legacy tests remain explicit gaps; the generator never fabricates passing placeholder proofs.

The current source scan contains 556 legacy test declarations and 553 unique legacy test identities; three Python test names are duplicated in-place in the original suite. The generated inventory deduplicates by legacy identity.

## Generation

```bash
doxygen Doxyfile.rust-port
cargo blue-gen generate \
  --xml target/doxygen/xml \
  --out rust-port/generated \
  --test-map rust-port/manifests/test-map.toml \
  --root .
```

Generated outputs include:

- `rust-port/generated/inventory.json`
- `rust-port/generated/legacy_tests.rs`
- `rust-port/generated/legacy_symbols.rs`
- `rust-port/generated/kani-pairing.md`
- `rust-port/generated/kani/*.rs`
- `crates/*/tests/blue_kani_generated.rs`

Use `--deny-unmapped` only when the migration is expected to have full legacy-test coverage; during an incremental port it intentionally fails while any legacy contract is still unmapped.

## Build and verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo kani --tests -p blue -p blueexposure -p blue-codegen
```

Kani is intentionally not a normal dependency. Under `cargo kani`, Kani injects its own crate and sets `cfg(kani)`; generated proof targets are completely absent from normal builds.

The Python extension crate is `bluepycore`; build it with a PyO3-capable packaging tool such as maturin or directly through Cargo for the target Python installation.
