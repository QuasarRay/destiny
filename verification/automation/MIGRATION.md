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


## Tenth slice: read-only reviewed integrity validation and typed receipt decoding

The lifecycle state machine intentionally classifies reviewed directories by artifact
presence. That keeps `status-reviewed` cheap and makes transition legality independent
of JSON parser behavior, but presence alone is not evidence integrity. The new
`validate-reviewed --reviewed-dir <dir>` command adds a separate read-only semantic
validation layer.

`ReviewedIntegrityValidation` is a CGP component that composes the existing lifecycle,
reviewed-binding validator, campaign-binding validator, content-addressed manifest
builder, digest provider, and token-identity provider. It validates only the artifacts
implied by the current lifecycle stage:

1. `reviewed-prepared`: decode the preparation receipt, validate the reviewed binding,
   and cross-check path/status/count correspondence.
2. `reviewed-finalized`: additionally validate the campaign binding and finalization
   receipt, including translation/count correspondence.
3. `reviewed-manifested`: rebuild the expected content-addressed manifest and require
   structural equality with the stored manifest, so byte-level drift in an underlying
   reviewed artifact is detected even when its parsed JSON meaning is unchanged.
4. `reviewed-audited`: additionally decode the audit receipt and require its manifest
   SHA-256 digest, verified-artifact count, engine/status, and proof-credit-source
   invariants to match the current evidence.

The command returns a typed `ReviewedIntegrityReport` containing the presence lifecycle
stage, proc-macro-canonical stage/validation identities, validated artifact identities,
and any violations. It mutates no evidence and always reports `proof_credit = false`.
The CLI exits nonzero when `valid = false`, while still printing the machine-readable
report.

This slice also adds the generic CGP `JsonReceiptDecoder`. Existing reviewed-binding,
campaign-execution, and campaign-binding validators now share that typed
`serde::de::DeserializeOwned` decoding boundary. Reviewed preparation/finalization
validation was factored into reusable typed helpers, replacing the previous ad-hoc
`serde_json::Value` field extractors in manifest construction.

All new command/artifact identities continue through `TokenIdentity`, backed by
`proc_macro2` and the safe `arcane-rs/synthez::ToTokens` replay wrapper. No unsafe
Arcane macro is introduced, and no proof-generation or Rust-to-Verus translation
semantics change.


## Eleventh slice: derived state metadata and composed reviewed explanation

This slice removes three handwritten metadata tables from the reviewed path. Lifecycle
stages, lifecycle actions, and evidence-artifact kinds now carry their own declarative
metadata and use `derive-deftly` to generate names, ranks, transition sets, and artifact
properties. `comprende` replaces repeated filter/push/map/collect scaffolding in the
automation planner, command catalog, lifecycle action derivation, and evidence policy.
CGP still owns all behavior-bearing boundaries; the macros only derive deterministic
metadata and collections.

The new `explain-reviewed` operation composes the existing presence lifecycle and
semantic integrity validator. It is read-only, grants no proof credit, recommends the
smallest legal next reviewed transition only when integrity is valid, and reports audited
valid evidence as complete. This gives callers one machine-readable decision surface
without weakening or bypassing `status-reviewed`, `validate-reviewed`, or any verifier
gate.


## Twelfth slice: one-command reviewed Verus verification

The reviewed path now has `verify-reviewed`, a CGP composition that advances an empty or
semantically valid prepared reviewed directory through real Verus campaign execution,
campaign binding, evidence manifesting, content-addressed audit, and final integrity
validation. It does not add a second verifier path or duplicate the existing stage
implementations.

Prepared-state reuse is deliberately narrow. The nested `KVERUS_PREPARED` receipt is now
typed and validated for engine, pinned Kani commit, no-credit status, output/evidence
paths, and contract-count correspondence. `verify-reviewed` additionally requires the
requested manifest to match that nested receipt exactly. Completed reviewed stages are
rejected so a previous proof cannot be presented as a fresh workflow execution.

`run-reviewed` is also hardened to reject pre-existing campaign runner/execution,
binding, or audit receipts. This closes the stale-receipt boundary before Verus execution.
The successful workflow receipt records derived step identities, exact manifest digest,
final audited stage, and the existing scoped-Verus proof-credit source while retaining
`proof_credit = false` for orchestration itself.

`derive-deftly` owns the workflow-step identity table; `comprende` constructs the
executed/reused step inventories; CGP composes all stateful operations.


## Thirteenth slice: unified selection and resource configuration

`verify-reviewed` now accepts `--config <json>` and uses one typed schema for Cargo
package/feature/target selection, reviewed function selection, proof timeout, optional
memory limit, and concurrency. The checked-in
`verification/automation/reviewed-verification.json` preserves the previous full-scope
behavior and is exercised by CI.

The configuration is not a shell-only adapter. CGP components validate and load it,
then thread the same selection through `AutomationPreparation`,
`ReviewedBindingPreparation`, and `ReviewedVerification`. Pinned Kani's own Cargo and
autoharness selectors are reused for discovery. Function selection is independently
resolved against the reviewed registry, persisted in `verus-binding.json`, and checked
against the nested preparation receipt. The Verus runner builds a filtered campaign plan
from exactly those bound controller obligation IDs before invoking the maintained
controller. Unknown or empty effective proof inventories fail closed.

The campaign receipt records the exact obligation list and effective resource limits.
Timeouts are reported as `TIMEOUT`, not generic proof success or an inferred source bug.
The configurable timeout cannot exceed the maintained controller's 120-second scoped
limit. Optional memory limiting uses the host's process address-space limit where
available. Verification remains serialized: `max_concurrency != 1` is rejected because
the maintained Destiny controller owns a sequential proof lock and durable attempt budget.

`derive-deftly` generates the target-kind argument metadata instead of duplicating
target switch tables. `comprende` generates repeated package, feature, function, and
obligation argument inventories. CGP remains the behavior-bearing boundary for config
validation/loading, discovery lowering, preparation, binding, execution, and final
workflow composition.
