# KVerus CGP migration

The immutable pre-migration snapshot is under `/kverus-before-migration` and records
the exact source commit and blob SHA of every copied KVerus file.

## Migration rules

- Preserve the existing public command and JSON surfaces unless a migration step explicitly versions them.
- Keep Verus as the default proof engine and keep Kani discovery/planning metadata at `proof_credit = false`.
- Introduce CGP only where a capability has a genuine provider boundary.
- Prefer `proc_macro2` + `synthez`/Syn for token and syntax handling; do not add new text scrapers.
- Use only Arcane-rs macros that preserve KVerus's `unsafe_code = "forbid"` boundary.
- Keep `#![forbid(unsafe_code)]`.
- Every refactor step must pass the KVerus unit tests and the upstream Kani regression unit phase for patched packages before its behavior is considered preserved.

## First slice: metadata ingestion

The metadata ingestion pipeline has compile-time CGP components for JSON parsing,
contract-clause syntax validation, and discovery normalization. Existing
`parse_metadata` and `normalize_autoharness_metadata` functions remain compatibility
facades.

`arcane-rs/delegation` was evaluated and rejected because its generated implementation
conflicts with KVerus's crate-level unsafe prohibition. The migration instead uses the
safe `arcane-rs/synthez` `ToTokens` derive over `proc_macro2::TokenStream`.

## Second slice: planning and orchestration

The next slice adds four explicit provider boundaries:

1. `ContractClauseSyntax` — parses Kani-produced clauses with Syn and returns a
   diagnostic canonical token rendering through `proc_macro2` + `synthez`.
2. `AutomationPlanner` — turns normalized Kani discovery into a machine-readable
   execution plan without granting proof credit or deciding Verus semantic support.
3. `ProjectLayout` — owns project-root resolution.
4. `PythonToolRunner` — owns Python executable selection and subprocess invocation.

The CLI's new `plan <kani-metadata.json>` command reports selected, blocked,
unsupported, and excluded candidates plus contract planning metadata. It preserves the
original Kani clause expression as authoritative; `canonical_tokens` is diagnostic
only. `main.rs` no longer owns project layout or Python process construction.

This slice intentionally does not change `verification/tools/verus_subset.py` or any
proof translation semantics.

## Third slice: end-to-end preparation

`AutomationPreparation` composes the already-validated providers instead of duplicating
their logic. The new `prepare --manifest-path <Cargo.toml> --output-dir <dir>` command:

1. invokes the existing metadata-only Kani discovery through `PythonToolRunner`;
2. reads Kani's combined metadata through the `ArtifactStore` provider;
3. reuses `MetadataParser`, `MetadataNormalizer`, and `AutomationPlanner`;
4. writes normalized discovery, the automation plan, and a preparation receipt;
5. keeps every generated planning/discovery artifact at `proof_credit = false`.

`ArtifactStore` publishes through a temporary sibling file plus rename, with a
conservative replacement fallback on platforms that reject rename-over-existing.
The Kani subprocess is captured so `prepare` itself has a single machine-readable JSON
stdout surface.

CI exercises `prepare` against all real valid, invalid, unsupported, and contract fixture
crates. The restricted Rust-to-Verus translator remains untouched.

## Fourth slice: reviewed binding preparation and typed CLI routing

`ReviewedBindingPreparation` composes `AutomationPreparation` with the existing
`verification/tools/bind_kani_verus.py bind` implementation. KVerus does not duplicate
the registry mapping or its proof-credit rules. The new
`prepare-reviewed --manifest-path <Cargo.toml> --output-dir <dir>` command:

1. performs metadata-only Kani discovery, normalization, and planning through the
   existing CGP preparation component;
2. invokes the existing reviewed Kani-to-Verus binder through `PythonToolRunner`;
3. validates the binder's public schema/status/engine/no-Kani-credit/nonempty-selected/
   empty-unresolved invariants;
4. writes `verus-binding.json` and `reviewed-preparation-receipt.json`;
5. explicitly records `verus_campaign_checked = false` and `proof_credit = false`.

This is preparation for the existing reviewed Verus concurrency campaign, not a new
translation theorem and not proof completion.

The CLI surface is also now modeled by the CGP `CommandCatalog` and
`CommandRouter` components. `main.rs` receives a typed `AutomationCommand` instead
of parsing flag strings itself. Static command names are converted to Rust-style
identifiers and replayed through `proc_macro2` plus the safe `synthez::ToTokens`
wrapper, giving the command catalog the same unsafe-free macro boundary as contract
token diagnostics. `arcane-rs/delegation` remains intentionally excluded.

## Fifth slice: reviewed campaign finalization and shared JSON tool boundary

`ReviewedCampaignFinalization` composes the existing
`verification/tools/bind_kani_verus.py check-campaign` path instead of duplicating its
receipt validation semantics. The new
`finalize-reviewed --reviewed-dir <dir> --campaign-dir <dir>` command:

1. reuses `verus-binding.json` produced by `prepare-reviewed`;
2. invokes the existing campaign binder through the CGP JSON subprocess runner;
3. requires the subprocess JSON and persisted `verus-receipt-binding.json` to be
   structurally identical;
4. validates the binder's public schema/status/engine/Kani-credit/scoped-Verus invariants;
5. writes `reviewed-finalization-receipt.json` with the checked receipt count and
   contract metadata counts.

This command does not run Verus and cannot manufacture proof receipts. Its
`verus_checked_scoped = true` value is accepted only when the existing binder has
validated a campaign directory containing the required fresh `CHECKED_SCOPED` receipts.

Two orchestration concerns are now reusable CGP providers:

- `JsonPythonToolRunner` owns captured Python subprocess execution, nonzero-exit
  diagnostics, and JSON stdout decoding;
- `JsonArtifactStore` owns JSON artifact loading on top of the existing atomic
  `ArtifactStore`.

The command catalog gains `finalize-reviewed`, so its identifier also passes through
the existing `proc_macro2` + safe `arcane-rs/synthez` token replay boundary. No
Rust-to-Verus translation semantics changed in this slice.

## Sixth slice: reviewed evidence manifest and reusable receipt validators

`ReviewedEvidenceAssembly` adds
`manifest-reviewed --reviewed-dir <dir>`. It consumes only artifacts already produced
by `prepare-reviewed` and `finalize-reviewed`:

- `reviewed-preparation-receipt.json`;
- `verus-binding.json`;
- `verus-receipt-binding.json`;
- `reviewed-finalization-receipt.json`.

The assembler cross-checks engine, proof-credit markers, selected/obligation counts,
contract inventory/clause counts, campaign receipt counts, translation-correctness
classification, and the reviewed/finalized status chain. It emits
`reviewed-evidence-manifest.json` with
`proof_credit_source = existing_verus_checked_scoped_receipts_only`. The manifest
does not itself prove anything and cannot upgrade Kani metadata into proof credit.

The earlier inline binder checks are now reusable CGP providers:

- `ReviewedBindingValidator` owns the public `BOUND_FOR_VERUS` invariants;
- `CampaignBindingValidator` owns the public
  `KANI_SELECTED_VERUS_CHECKED_SCOPED` invariants.

`prepare-reviewed`, `finalize-reviewed`, and `manifest-reviewed` all reuse these
validators rather than duplicating receipt policy.

A new `TokenIdentity` CGP provider owns safe static-identifier canonicalization.
Both CLI command identities and evidence-artifact kinds are normalized through the
same `proc_macro2` + `synthez::ToTokens` path. This keeps the Arcane-derived macro
surface safe and avoids introducing `arcane-rs/delegation` or unsafe code.

No Rust-to-Verus lowering semantics, Kani contract semantics, reviewed mappings, or
proof campaign acceptance rules changed in this slice.
