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

## Seventh slice: content-addressed reviewed evidence and independent audit

The reviewed evidence manifest is explicitly versioned to schema 2. Every declared
reviewed artifact now carries:

- a typed artifact kind selected by `EvidenceArtifactPolicy`;
- its canonical token identity from `TokenIdentity`;
- the exact repository/runtime path;
- expected semantic status and proof-credit provenance;
- `digest_algorithm = sha256`;
- the SHA-256 digest of the exact stored bytes.

`ArtifactDigest` is a CGP capability implemented by the safe RustCrypto `sha2`
interface. `EvidenceArtifactPolicy` centralizes the reviewed artifact set so the
assembler and auditor cannot silently drift to different inventories.

The new `audit-reviewed --reviewed-dir <dir>` command does not trust the stored
manifest as an oracle. It rebuilds an expected schema-2 manifest from the current
preparation/binding/finalization artifacts using the existing semantic validators,
recomputes every byte digest, and requires structural equality with the stored
`reviewed-evidence-manifest.json`. On success it emits
`reviewed-evidence-audit.json`, including the SHA-256 digest of the manifest itself.

The automation regression lane includes a negative test that appends whitespace to a
reviewed JSON artifact after manifest creation and requires `audit-reviewed` to fail,
then restores the artifact and requires a successful audit. This demonstrates
byte-level drift detection independently of JSON semantic equivalence.

This is tamper-evident/content-addressed evidence, not a digital signature: an actor
capable of replacing both the evidence and its manifest can still create a new
self-consistent bundle. No proof semantics or proof-credit rules change.

## Eighth slice: typed reviewed lifecycle and provenance-safe handoff

Reviewed evidence is now governed by an explicit CGP lifecycle rather than by the
incidental presence of whatever files a command happens to need.

`ReviewedLifecycleInspection` classifies the exact reviewed artifact inventory as one
of:

- `reviewed-empty`;
- `reviewed-prepared`;
- `reviewed-finalized`;
- `reviewed-manifested`;
- `reviewed-audited`;
- `reviewed-inconsistent`.

The stage identity is canonicalized through the existing `TokenIdentity` provider,
therefore through the same `proc_macro2` + safe `synthez::ToTokens` boundary used
for commands and evidence-artifact identities. The new
`status-reviewed --reviewed-dir <dir>` command reports the stage, canonical token,
present/missing lifecycle artifacts, legal next actions, and `proof_credit = false`.

`ReviewedLifecycleTransition` now guards the mutating reviewed commands:

- `prepare-reviewed`: only empty or already-prepared state;
- `finalize-reviewed`: only prepared or already-finalized state;
- `manifest-reviewed`: only finalized or already-manifested state;
- `audit-reviewed`: only manifested or already-audited state.

This preserves idempotent reruns at the current stage while rejecting rollback-like
writes after downstream evidence exists. Any partial/non-prefix artifact set becomes
`reviewed-inconsistent` and is rejected rather than guessed around.

The workflow also fixes a previously latent provenance problem: synthetic
finalization/audit regression evidence is no longer written into the same directory
uploaded to the real Verus job. The real `kani-verus-automation-discovery` handoff
now remains strictly `reviewed-prepared`; synthetic lifecycle regression runs in a
separate directory/artifact. The real Verus finalization step asserts the prepared
stage before consuming fresh campaign receipts.

No verifier semantics, Kani metadata semantics, reviewed function mappings, or proof
credit rules changed.


## Ninth slice: CGP-owned reviewed Verus campaign execution

The reviewed Verus campaign is no longer launched directly from workflow shell code.
The new `run-reviewed --reviewed-dir <dir> --campaign-dir <dir> --verus <path>`
command introduces two provider boundaries:

- `ReviewedCampaignExecution` owns the reviewed execution precondition, invocation of
  the existing `verification/tools/run_verus_campaign.py` proof loop, and the KVerus
  execution receipt;
- `CampaignExecutionValidator` owns the machine-readable runner-receipt invariants.

The Python runner keeps its proof semantics unchanged. It still binds the declared plan,
executes each reviewed Verus entry once, rejects any non-Verus method, requires every
per-obligation result to be `CHECKED_SCOPED`, performs the existing final audit, and
retains the existing case-acceptance exit behavior. This slice only adds explicit
`--campaign-dir` and `--receipt` plumbing so the Rust orchestration layer can consume
the result deterministically. Failures now persist an `INCOMPLETE` runner receipt for
diagnosis instead of existing only in mixed stdout/stderr.

A successful KVerus execution receipt requires:

- schema 1 and engine `verus`;
- `status = CHECKED_SCOPED`;
- `kani_proof_credit = false`;
- `verus_checked_scoped = true`;
- a nonempty deduplicated reviewed-entry count;
- `formal_audit = CHECKED_SCOPED`;
- `case_acceptance_exit = 0`.

The KVerus wrapper writes
`<campaign-dir>/kverus-reviewed-campaign-execution.json`, with
`proof_credit = false` and
`proof_credit_source = existing_verus_checked_scoped_receipts_only`. This artifact is
execution/orchestration evidence, not part of the reviewed binding lifecycle and not a
new proof source. The reviewed directory intentionally remains `reviewed-prepared`
until `finalize-reviewed` binds actual campaign receipts.

The CLI parser is also simplified: one generic required-named-flag parser now backs the
one-, two-, and three-argument reviewed commands, replacing command-specific flag
parsers. The new `run-reviewed` command identity and its
`reviewed-campaign-execution` receipt identity continue through the shared
`proc_macro2` + safe `arcane-rs/synthez::ToTokens` `TokenIdentity` provider.

CI tests the new runner receipt protocol without requiring Verus by deliberately
supplying a missing plan and requiring a persisted `INCOMPLETE` receipt plus nonzero
exit. The real Verus job now invokes `run-reviewed`; it no longer calls
`run_verus_campaign.py` directly.
