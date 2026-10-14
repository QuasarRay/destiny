# Shared verifier automation boundary

This crate is the first executable boundary for the Verus-default automation work.

It directly compiles against the vendored Kani 0.68.0 `kani_metadata` crate and consumes Kani's own `AutoHarnessMetadata` / `AutoHarnessSkipReason` representation. It does not copy the schema, does not run CBMC, and does not claim that a Kani-eligible function is Verus-supported.

Current commands:

```sh
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- doctor
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- discover path/to/.kani-metadata.json
```

`discover` is classification only and always emits `proof_credit: false`. The next implementation milestone is to invoke the vendored Kani compiler in autoharness metadata/list mode, with CBMC verification disabled, then map only the declared integer/Boolean subset to generated Verus obligations and the existing source-bound evidence controller.


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
