# Destiny formal verification workspace

This directory treats the original Destiny implementation on `main` as the
behavioral specification source. It deliberately does **not** use Bevy, Avian,
Lightyear, or Replicon types in the specification layer.

## Verification rule

A behavior is only called *proved parity* when all of the following hold:

1. the original Destiny test/behavior has a machine-readable specification here;
2. the Rust implementation is connected to that specification through a
   representation adapter that does not expose engine internals;
3. a Kani proof establishes the concrete Rust safety/behavior obligation;
4. a Verus proof establishes the corresponding logical invariant/refinement
   obligation where Verus can soundly model the value domain;
5. a differential test checks the original implementation against the same
   specification for behaviors that cannot be linked directly into either
   verifier.

Anything else is classified as `Specified`, `ImplementationMissing`,
`ProofMissing`, or `KnownMismatch`. The verification code must never turn an
unsupported Destiny operation into an assumption.

## Layout

- `spec/`: Bevy-independent executable specification and original-test catalog.
- `meta/`: reusable proc/declarative/derive/attribute/delegation infrastructure.
- `kani/`: concrete model-checking harnesses.
- `verus/`: deductive proofs and refinement lemmas.
- `reports/`: generated proof-obligation and repetition reports.

The original test catalog is intentionally exhaustive with respect to the test
files present on the original `main` tree at commit
`ef738406b61ad06bae2f402857e464735ab106a2`.
