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
