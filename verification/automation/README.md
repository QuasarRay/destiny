# Shared verifier automation boundary

This crate is the first executable boundary for the Verus-default automation work.

It directly compiles against the vendored Kani 0.68.0 `kani_metadata` crate and consumes Kani's own `AutoHarnessMetadata` / `AutoHarnessSkipReason` representation. It does not copy the schema, does not run CBMC, and does not claim that a Kani-eligible function is Verus-supported.

Shared command surface:

```sh
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- doctor
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- setup
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- discover path/to/.kani-metadata.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- list path/to/.kani-metadata.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- explain
cargo +1.98.1 run --manifest-path verification/automation/Cargo.toml --locked -- verify \
  --manifest-path path/to/Cargo.toml --discovery path/to/normalized-discovery.json \
  --function function_name --verus path/to/verus
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- \
  replay-kani-reference --source path/to/lib.rs --function function_name --cargo-kani path/to/cargo-kani
```

`setup` reuses the existing vendor-integrity and Kani patch bootstrap scripts; it does not install host toolchains or silently fetch external tools. `discover` and `list` are classification only and always emit `proof_credit: false`. `verify` delegates to the existing restricted Verus subset implementation and keeps Verus as the default proof engine. `replay-kani-reference` is deliberately explicit: it delegates to the separate Kani/CBMC source-replay lane and cannot grant Verus proof credit. Set `DESTINY_PYTHON` only when the platform's Python executable is not the default `python3` (or `python` on Windows).


## Kani-selected reviewed Verus entries

`registry.json` is the declared semantic bridge for actual Destiny specification
functions. It admits only cases whose current Rust semantics are Boolean logic,
integer equality, or value selection without arithmetic overflow, casts,
pointers, allocation, concurrency, or floating point. Existing `f64`,
checked-arithmetic and bounded-`i128` functions remain explicitly unsupported.

`verification/tools/bind_kani_verus.py bind` consumes the normalized metadata
from the single Kani discovery path. Every discovered candidate is classified as
selected, intentionally excluded, unsupported, or blocked. A required supported
candidate that is absent or blocked makes the binding incomplete.

The ordinary reviewed Verus campaign still runs once. Afterward,
`check-campaign` binds the Kani-selected obligation IDs to those fresh
`CHECKED_SCOPED` receipts. This is a reviewed correspondence boundary, not a
general Rust-to-Verus translation proof and not Kani proof credit.


## Restricted generated-obligation fixture slice

`verification/tools/verus_subset.py` reuses the single normalized Kani compiler
discovery artifact and generates Verus obligations only for the grammar declared
in `capabilities.json`. It does not invoke Kani itself. Branching is preserved
structurally; accepted literal `for _ in A..B` loops are exhaustively unrolled
with the exact range recorded in evidence.

The fixture corpus contains a universally valid branch, an exhaustively bounded
literal loop, two independent symbolic scalar inputs, a deliberately invalid
universal assertion, and an unsupported floating-point function. Arithmetic,
target-width integers and assumptions are rejected by regression tests rather
than silently modeled with mathematical integers or extra premises.

A Verus proof failure remains a proof failure. Concrete source replay is kept as
a separate explicit Kani/CBMC reference lane and is not required or invoked by
the default Verus fixture path.


## Explicit concrete replay reference lane

Pinned Kani 0.68 records in its source that automatic harnesses do not support
concrete playback. Therefore replay is intentionally **not** part of the
Verus-default autoharness path.

`verification/tools/replay_kani_fixture.py` creates a temporary manual Kani
harness that imports the original failing fixture with Rust's `#[path]`
attribute. Kani/CBMC generates the concrete witness and Kani playback executes
that harness. A result is called `reproduced_source_failure` only when playback
fails at the original source assertion line and the original source remains
byte-for-byte unchanged. This lane is explicitly labeled
`kani_cbmc_reference` and grants no Verus proof credit.
