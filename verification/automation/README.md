# Shared verifier automation boundary

This crate is the first executable boundary for the Verus-default automation work.

It directly compiles against the vendored Kani 0.68.0 `kani_metadata` crate and consumes Kani's own `AutoHarnessMetadata` / `AutoHarnessSkipReason` representation. It does not copy the schema, does not run CBMC, and does not claim that a Kani-eligible function is Verus-supported.

Current commands:

```sh
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- doctor
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- discover path/to/.kani-metadata.json
```

`discover` is classification only and always emits `proof_credit: false`. The next implementation milestone is to invoke the vendored Kani compiler in autoharness metadata/list mode, with CBMC verification disabled, then map only the declared integer/Boolean subset to generated Verus obligations and the existing source-bound evidence controller.
