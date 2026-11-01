# Validation record

Source-level consistency checks performed on the generated workspace:

- every workspace member path exists;
- all Cargo TOML files parse as TOML;
- 11 `#[blue_test]` functions were discovered;
- 11 generated `#[kani::proof]` wrappers were materialized;
- every generated wrapper resolves to its expected shared `__blue_test_body_*` function;
- no legacy mapping targets a missing native semantic test;
- no hand-maintained Kani proof body is present outside the Genco generator/output path.

A Rust toolchain was not available in the execution environment used to prepare this archive (`cargo`, `rustc`, and `rustfmt` were absent), and outbound package-manager/network access was unavailable. Therefore this record does **not** claim a successful `cargo test`, `cargo clippy`, or `cargo kani` run.

Run on a Rust-enabled host:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo blue-gen generate --root .
cargo kani --tests -p blue -p blueexposure -p blue-codegen
```
