# Unified Cargo workspace

All Rust crates are workspace members of the root `Cargo.toml` and use the
single root `Cargo.lock` and `rust-toolchain.toml`.

```sh
cargo check --workspace --all-targets
cargo test --workspace
cargo build -p destiny-rs
cargo build -p blue
cargo test -p destiny-rs
cargo blue-gen --help
```

The gameplay code and Blue-exposed Destiny Settings implementation live in
`destiny-rs/src/`; `destiny-rs/tests/` restores the partition and settings
regression suites. Blue's reusable crates remain in `blue-rs/crates/`.
Use `destiny_rs::settings` and `destiny_rs::partition` to import the Rust
modules. The two previously maintained copies of Settings are reconciled:
the moved implementation was preserved, with its missing Bevy reflection and
change-detection imports restored from the validated copy.

The root manifest pins Bevy, CGP, PyO3 and all shared dependencies, and defines
workspace-wide package metadata and lint rules. Individual crates keep only
their name, unique flags, and `.workspace = true` dependencies.

Cargo searches parent directories for the manifest, so `cargo` inside
`blue-rs` and `destiny-rs` also resolves to this root workspace.

Do not apply old `destiny-settings` patches: it is now one module inside
`destiny-rs`. Save any unpushed local edits before pulling these changes.
