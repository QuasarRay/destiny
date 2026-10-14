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
