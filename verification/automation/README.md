# Shared verifier automation boundary

This crate is the executable boundary for the Verus-default automation work.

It directly compiles against the vendored Kani 0.68.0 `kani_metadata` crate and consumes Kani's own `AutoHarnessMetadata`, `AutoHarnessSkipReason`, and `ContractedFunction` representations. It does not copy those schemas, does not run CBMC on the default path, and does not claim that Kani eligibility or Kani contract metadata is itself a Verus proof.

Shared command surface:

```sh
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- doctor
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- setup
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- discover path/to/.kani-metadata.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- list path/to/.kani-metadata.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- plan path/to/.kani-metadata.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- prepare \
  --manifest-path path/to/Cargo.toml --output-dir path/to/evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- prepare-reviewed \
  --manifest-path verification/spec/Cargo.toml --output-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- verify-reviewed \
  --manifest-path verification/spec/Cargo.toml --reviewed-dir path/to/reviewed-evidence \
  --campaign-dir path/to/verus-final-campaign --verus path/to/verus \
  --config verification/automation/reviewed-verification.json
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- run-reviewed \
  --reviewed-dir path/to/reviewed-evidence --campaign-dir path/to/verus-final-campaign \
  --verus path/to/verus
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- finalize-reviewed \
  --reviewed-dir path/to/reviewed-evidence --campaign-dir path/to/verus-final-campaign
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- manifest-reviewed \
  --reviewed-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- audit-reviewed \
  --reviewed-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- status-reviewed \
  --reviewed-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- validate-reviewed \
  --reviewed-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- explain-reviewed \
  --reviewed-dir path/to/reviewed-evidence
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- explain
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- verify \
  --manifest-path path/to/Cargo.toml --discovery path/to/normalized-discovery.json \
  --function function_name --verus path/to/verus
cargo +nightly-2026-08-21 run --manifest-path verification/automation/Cargo.toml --locked -- \
  replay-kani-reference --source path/to/lib.rs --function function_name --cargo-kani path/to/cargo-kani
```

`setup` reuses the existing vendor-integrity and Kani patch bootstrap scripts; it does not install host toolchains or silently fetch external tools. The CLI command catalog and argument routing are now CGP providers rather than a duplicated usage string plus stringly-typed parser in `main.rs`; command identifiers are normalized through `proc_macro2` and the safe `arcane-rs/synthez` token replay wrapper. Captured JSON subprocess handling and JSON artifact decoding are also CGP providers. `JsonReceiptDecoder` adds a typed serde boundary shared by reviewed binding, campaign execution, campaign binding, manifest construction, and integrity validation, removing repeated `serde_json::from_value` and field-by-field receipt decoding. Reviewed binding/campaign receipt validation is likewise factored into CGP validators and reused by evidence assembly. `ArtifactDigest` owns SHA-256 byte hashing, while `EvidenceArtifactPolicy` owns the typed artifact-kind/filename/status/proof-credit mapping shared by manifest construction and auditing. `ReviewedLifecycleInspection` and `ReviewedLifecycleTransition` make the reviewed pipeline an explicit state machine instead of inferring legality from downstream failures. Static command, lifecycle-stage, and evidence-artifact identities share one `TokenIdentity` provider backed by `proc_macro2` and safe `arcane-rs/synthez` token replay. Python executable selection, project-root resolution, and tool invocation are now CGP providers rather than hard-coded CLI concerns. `discover` and `list` are classification only and always emit `proof_credit: false`. Their normalized report preserves Kani's compiler-produced `contracted_functions` inventory, including function identity, source file, contract harness identities, and the parsed `requires`/`ensures`/`modifies` clauses exported by the pinned Kani metadata extension. `plan` turns the same compiler-produced discovery into a machine-readable pre-verification plan grouped into selected, blocked, unsupported, and excluded candidates. `prepare` composes the existing metadata-only Kani discovery with normalization and planning, captures the discovery subprocess output, and writes `kani-metadata.json`, `normalized-discovery.json`, `automation-plan.json`, and `preparation-receipt.json` as one CGP-owned preparation pipeline. `prepare-reviewed` extends that same CGP pipeline with the existing reviewed Kani-to-Verus binder, validates only its public receipt invariants, and emits `verus-binding.json` plus `reviewed-preparation-receipt.json`; it still grants no proof credit because the Verus campaign has not run yet. `run-reviewed` is the CGP-owned execution boundary for the existing `run_verus_campaign.py` proof loop: it requires a `reviewed-prepared` lifecycle, supplies explicit campaign/verifier paths, requires a persisted machine-readable runner receipt, validates scoped Verus/audit/case-acceptance invariants, and writes `kverus-reviewed-campaign-execution.json` under the campaign directory. The orchestration receipt itself keeps `proof_credit: false`; credit remains in the existing `CHECKED_SCOPED` Verus receipts. `finalize-reviewed` consumes that reviewed directory plus an existing Verus campaign directory, delegates to the existing `check-campaign` binder path, verifies that stdout and the persisted campaign receipt are identical, and emits `reviewed-finalization-receipt.json`. The final receipt credits only the already-checked scoped Verus campaign receipts and explicitly keeps `kani_proof_credit: false`. `manifest-reviewed` then cross-checks the reviewed preparation receipt, reviewed binding, campaign binding, and finalization receipt and emits schema-2 `reviewed-evidence-manifest.json`; it does not create proof credit, records `existing_verus_checked_scoped_receipts_only` as the sole credit source, and stores a SHA-256 digest of the exact bytes of every reviewed artifact. `audit-reviewed` independently rebuilds the expected manifest from the current artifacts, requires exact structural equality with the stored manifest, recomputes every digest, and emits `reviewed-evidence-audit.json`. Appending even semantically irrelevant whitespace to a reviewed artifact is therefore detected as evidence drift. `status-reviewed` reports one typed lifecycle stage—`reviewed-empty`, `reviewed-prepared`, `reviewed-finalized`, `reviewed-manifested`, `reviewed-audited`, or `reviewed-inconsistent`—plus legal next actions. It is intentionally presence-only. `validate-reviewed` is the complementary read-only integrity check: it decodes and cross-checks every receipt implied by the current stage, reuses the existing reviewed/campaign validators, rebuilds the content-addressed manifest when present, and verifies an audit receipt's manifest digest when audited. A directory can therefore remain `reviewed-manifested` by presence while `validate-reviewed` rejects byte-level artifact drift. `run-reviewed` is legal only from `reviewed-prepared` and intentionally does not advance that reviewed lifecycle because campaign invocation is not itself reviewed proof binding. `prepare-reviewed`, `run-reviewed`, `finalize-reviewed`, `manifest-reviewed`, and `audit-reviewed` enforce the transition policy before acting, preventing a real campaign from silently reusing synthetic or downstream-complete reviewed artifacts. Contract rows retain the original Kani expression and add a diagnostic-only canonical token rendering produced through `proc_macro2` plus the safe `arcane-rs/synthez` token pipeline; the original compiler-emitted expression remains authoritative and the plan always has `proof_credit: false`. `verify` delegates to the existing restricted Verus subset implementation and keeps Verus as the default proof engine. `replay-kani-reference` is deliberately explicit: it delegates to the separate Kani/CBMC source-replay lane and cannot grant Verus proof credit. Set `DESTINY_PYTHON` only when the platform's Python executable is not the default `python3` (or `python` on Windows).


## Kani-selected reviewed Verus entries

`registry.json` is the declared semantic bridge for actual Destiny specification functions. It admits only cases whose current Rust semantics are Boolean logic, integer equality, or value selection without arithmetic overflow, casts, pointers, allocation, concurrency, or floating point. Existing `f64`, checked-arithmetic and bounded-`i128` functions remain explicitly unsupported.

`verification/tools/bind_kani_verus.py bind` consumes the normalized metadata from the single Kani discovery path. Every discovered autoharness candidate is classified as selected, intentionally excluded, unsupported, or blocked. A required supported candidate that is absent or blocked makes the binding incomplete.

The binder also validates and carries the normalized Kani contract inventory. It still grants no proof credit. The generated-obligation lane may consume those Kani-parsed clauses only through the restricted contract subset below; the reviewed Destiny registry mappings continue to point at existing reviewed Verus entries. Duplicate or malformed contract identities/clauses are rejected rather than silently merged.

The ordinary reviewed Verus concurrency campaign still runs once. Afterward, `check-campaign` binds the Kani-selected obligation IDs to those fresh `CHECKED_SCOPED` receipts. This is a reviewed correspondence boundary, not a general Rust-to-Verus translation proof and not Kani proof credit.


## Restricted generated-obligation fixture slice

`verification/tools/verus_subset.py` reuses the single normalized Kani compiler discovery artifact and generates Verus obligations only for the grammar declared in `capabilities.json`. It does not invoke Kani itself. Branching is preserved structurally; accepted literal `for _ in A..B` loops are exhaustively unrolled with the exact range recorded in evidence.

The generated slice currently supports:

- independent Boolean and fixed-width integer parameters;
- structural `if/else`;
- literal bounded loops up to 32 iterations;
- source-scoped `kani::assume` with separate Verus-checked non-vacuity obligations;
- selected primitive monomorphic generic instances reported by Kani;
- `+`, `-`, and `*` only inside executable branch conditions, where Verus must prove fixed-width overflow/underflow safety;
- Kani-parsed non-arithmetic `requires` predicates over the supported scalar parameters;
- Kani-parsed unit-return `ensures` closures of the form `|_| predicate`, plus supported scalar-return result-reference closures such as `|result: &u8| *result == x`, lowered to a named Verus result value;
- one unsigned fixed-width `+`, `-`, or `*` with a nonnegative integer literal inside an `ensures` comparison; KVerus emits the exact definedness guard (for example `x <= 254` for `u8 x + 1`) into the Verus postcondition and evidence;
- a separate Verus existential satisfiability obligation for every accepted `requires` set, preventing contradictory preconditions from making a contract proof vacuous.

It deliberately rejects ghost arithmetic in assertions or assumptions, target-width integers, floating point, pointers/references, allocation, unsafe code, dynamic loops/recursion, and general concurrent Rust source translation. The reviewed registry/binder may select an existing Verus concurrency proof for a Kani-discovered specification function, but that is distinct from translating arbitrary concurrent Rust into Verus.

A Verus proof failure remains a proof failure. Concrete source replay is kept as a separate explicit Kani/CBMC reference lane and is not required or invoked by the default Verus fixture path.


## Restricted function-contract semantic bridge

Pinned Kani exposes contracted functions through `kani_metadata::KaniMetadata::contracted_functions`. The local pinned Kani patch now extends that upstream metadata path at Kani's own proc-macro parsing boundary: after Kani parses a contract attribute into `syn::Expr` / `ExprClosure`, it preserves the parsed clause kind and token representation in `ContractedFunction`. KVerus therefore does not derive contract semantics by scraping the user's source attributes.

The normalized contract rows contain:

- crate identity;
- fully qualified contracted function identity as emitted by Kani;
- source file identity;
- Kani contract-harness identities;
- Kani-parsed clause kind and expression records;
- explicit upstream provenance.

The Kani-to-Verus binder validates those rows and carries them with `proof_credit: false` and `contract_semantics_status: compiler_parsed_clauses_available`. `verification/tools/verus_subset.py` then implements the restricted semantic slice: non-arithmetic Boolean/comparison `requires`, unit-return `ensures` closures whose result binder is `_`, and supported scalar-return `ensures` closures where Kani's result reference is dereferenced and mapped to a same-typed named Verus return value. `ensures` may additionally contain exactly one `u8/u16/u32/u64` scalar/literal `+`, `-`, or `*` operation on one side of a comparison; KVerus emits its Rust-definedness bound into the Verus postcondition and records that guard in evidence. Arithmetic in `requires` remains rejected because Kani evaluates the expression before `kani::assume`, so guarding it would change overflow behavior. Contract preconditions must also pass a separately generated Verus existential satisfiability proof. `modifies`, `old`/history, general references/calls, signed or nested contract arithmetic, non-scalar returns, and richer result structures remain explicitly unsupported. A successful result credits the generated Verus obligation, never the metadata itself.


## Explicit concrete replay reference lane

Pinned Kani 0.68 records in its source that automatic harnesses do not support concrete playback. Therefore replay is intentionally **not** part of the Verus-default autoharness path.

`verification/tools/replay_kani_fixture.py` creates a temporary manual Kani harness that imports the original failing fixture with Rust's `#[path]` attribute. Kani/CBMC generates the concrete witness and Kani playback executes that harness. A result is called `reproduced_source_failure` only when playback fails at the original source assertion line and the original source remains byte-for-byte unchanged. This lane is explicitly labeled `kani_cbmc_reference` and grants no Verus proof credit.


## Derived reviewed-state metadata and composed explanation

The reviewed-state model now keeps lifecycle-stage names/ranks/recommendations, legal action
names/stage sets, and reviewed evidence-artifact metadata on their enum variants and derives
the repetitive accessors with `derive-deftly`. `comprende` builds command descriptors,
planner partitions, lifecycle action lists, and evidence specifications from those typed
sources. CGP remains the component boundary; the macros remove table-shaped boilerplate
without changing verifier semantics.

`explain-reviewed --reviewed-dir <dir>` is a read-only CGP composition of
`ReviewedLifecycleInspection` and `ReviewedIntegrityValidation`. It reports the full
lifecycle and integrity records together with one smallest useful recommended next action.
An invalid/inconsistent directory has no recommended transition and exposes its integrity
violations as blockers. A valid audited directory reports `complete: true`. The command
never grants proof credit and does not mutate evidence.


## One-command reviewed Verus workflow

`verify-reviewed` is the low-friction reviewed verification entry point. It composes the
existing CGP preparation, Verus execution, campaign finalization, content-addressed
manifest, audit, and integrity-validation components rather than reimplementing them.
A fresh directory is prepared automatically. A `reviewed-prepared` directory may be
reused only after semantic integrity validation and only when its nested preparation
receipt is bound to the exact requested manifest. Finalized, manifested, audited, or
inconsistent directories are rejected so the command cannot silently reuse an older
Verus proof.

The workflow also requires fresh campaign evidence. Existing runner, KVerus execution,
binding, or audit receipts in the campaign directory cause a fail-closed error before
Verus runs. Successful output is a compact `KVERUS_REVIEWED_WORKFLOW_COMPLETE` receipt
with the starting stage, executed/reused steps, selected and checked counts, manifest
digest, final audited stage, and proof-credit provenance. The orchestration receipt itself
has `proof_credit: false`; formal credit remains solely in the exact fresh
`CHECKED_SCOPED` Verus receipts that the finalization and audit bind.

The workflow-step inventory is generated with `derive-deftly`, and `comprende` builds
the executed/reused step lists. CGP remains responsible for behavior and component
composition.


## Unified reviewed verification configuration

`verification/automation/reviewed-verification.json` is the single optional configuration
source for `verify-reviewed`. Schema 1 contains `selection` and `limits` objects.

Selection is carried end to end rather than interpreted as UI-only filtering:

- `packages` forwards repeated Cargo package selection to Kani autoharness discovery;
- `features`, `all_features`, and `no_default_features` forward Cargo feature selection;
- `target` is one of `default`, `lib`, `bins`, or `bin`; `bin` requires
  `target_name`;
- `functions` accepts exact supported reviewed registry identities or their Kani metadata
  names. Each value is lowered to an escaped suffix-exact Kani `--include-pattern`,
  because pinned Kani filters autoharness candidates using a crate-qualified function name.
  The binder independently resolves every requested value to exactly one supported registry
  mapping and rejects unknown, ambiguous, duplicated, or alias-duplicated selections.

The selected function set therefore constrains Kani discovery, the Kani-to-Verus reviewed
binding, and the generated Verus campaign plan together. The campaign binds and audits that
filtered plan, so a partial configured run is never credited as though the full registry
had executed. An empty function list preserves the existing full reviewed registry.

The upstream selector behavior is taken from pinned Kani
`kani-driver/src/args/autoharness_args.rs::CommonAutoharnessArgs`,
`kani-driver/src/args/cargo.rs::{CargoCommonArgs,CargoTargetArgs}`, and
`kani-compiler/src/kani_middle/codegen_units.rs::autoharness_filtered_out`; KVerus
adapts those existing selectors instead of implementing a parallel discovery language.

Resource controls are deliberately fail-closed:

- `timeout_seconds` must be 1 through 120. The outer campaign process enforces the
  requested wall-clock limit while the maintained Destiny controller retains its existing
  120-second scoped reservation and durable attempt accounting;
- `memory_mib` is optional. When set, the campaign applies an address-space limit to
  each controller/verifier subprocess tree on platforms exposing `resource.RLIMIT_AS`;
- `max_concurrency` is currently required to be `1`, matching the maintained
  controller's sequential verification lock. Higher values are rejected rather than
  silently ignored.

The CGP `ReviewedVerificationConfigValidation`,
`ReviewedVerificationConfigStore`, and `DiscoverySelectionArguments` components own
validation, loading, and lowering. `derive-deftly` generates the Cargo-target metadata
used by that lowering, while `comprende` constructs repeated package, feature, function,
and obligation argument inventories. The exact effective selection and limits are retained
in preparation/execution/final workflow evidence. They do not grant proof credit.
