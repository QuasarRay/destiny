# Verus/Kani automation baseline

Branch baseline: `destiny-rs-dev` at `f5bb8edafb27d35165ceafeb39be8b0498e553e9`.

The existing Destiny verification workspace keeps concrete Kani obligations and Verus refinement obligations distinct. This integration must not relabel either class as discharged. Existing counted Verus execution continues through `verification/tools/run_verus_campaign.py` and the maintained Aegis controller; Kani remains the concrete/reference backend until a specific obligation is migrated with equivalent or stronger evidence.

Pinned upstream source:

- Verus `aef82eda71838deef4a8cd0260fda6c9504be9b9` (release `0.2026.09.20.aef82ed`, Rust 1.98.1).
- Kani `0d2328a93f0e0ff66132d6bfa1a7d884877cf862` (Kani 0.68.0, `nightly-2026-08-21`).

The first implementation boundary is deliberately narrow: the shared automation crate consumes Kani's own `kani_metadata::AutoHarnessMetadata` and `AutoHarnessSkipReason` types and normalizes discovery results for a Verus-default planner. This does **not** execute Kani, CBMC, or Verus, and therefore produces no proof credit. Compiler-side autoharness generation is the next dependency-ordered milestone.

Reproduction:

```sh
git submodule update --init third_party/verus third_party/kani
python3 -B verification/tools/check_vendor_integrity.py
rustup toolchain install nightly-2026-08-21 --profile minimal
cargo +nightly-2026-08-21 test --manifest-path verification/automation/Cargo.toml --locked
```

The Kani nested Charon/firecracker/performance submodules are not required for this metadata-only boundary and are intentionally not initialized by CI. Any later Kani compiler feature that needs one must first promote it to a required pin in `verification/vendor/upstreams.json`.


## Staged compiler-discovery patch

The next dependency-ordered slice is recorded in
`verification/vendor/patches/kani/0001-autoharness-metadata-only.patch`.
It adds `cargo_metadata_only` in Kani's driver and routes
`autoharness --list` through that function. The path still executes Kani's
real compiler and automatic-harness generation, but stops before
`Project::try_new` links GOTO artifacts.

`verification/tools/bootstrap_kani_vendor.py` validates the exact base,
patch digest and postimage Git blobs. `run_kani_autoharness_discovery.py`
puts failing shims ahead of CBMC/GOTO executables; any accidental backend
invocation is therefore an error rather than silently becoming proof work.
