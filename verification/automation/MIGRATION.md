# KVerus CGP migration

The immutable pre-migration snapshot is under `/kverus-before-migration` and records
the exact source commit and blob SHA of every copied KVerus file.

## Migration rules

- Preserve the existing public command and JSON surfaces unless a migration step explicitly versions them.
- Keep Verus as the default proof engine and keep Kani discovery metadata at `proof_credit = false`.
- Introduce CGP only where a capability has a genuine provider boundary.
- Prefer `proc_macro2` + `synthez`/Syn for token and syntax handling; do not add new text scrapers.
- Use arcane-rs delegation macros only for mechanical forwarding/delegation.
- Keep `#![forbid(unsafe_code)]`.
- Every refactor step must pass the KVerus unit tests and the upstream Kani regression unit phase for patched packages before its behavior is considered preserved.

## First slice

The metadata ingestion pipeline now has three compile-time CGP components:

1. `MetadataParser` — decodes Kani's one-or-many JSON envelope.
2. `ContractClauseValidator` — validates compiler-produced clause token streams using
   `proc_macro2` and `synthez::syn` without rewriting their text.
3. `MetadataNormalizer` — produces the existing KVerus discovery report and depends on
   the clause validator through CGP.

The existing `parse_metadata` and `normalize_autoharness_metadata` functions remain as
compatibility facades. The arcane-rs `delegation` macro handles the one-vs-many metadata
envelope conversion without a hand-written enum match.
