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


## Kani-assisted Verus automation

The default automation engine is Verus. Kani 0.68.0 is reused as a discovery
frontend for the initial supported subset: its autoharness compiler path decides
which executable-specification functions are eligible and emits
`AutoHarnessMetadata`; list mode is codegen-only and does not run CBMC.
Selected functions are then bound to existing reviewed Verus entries and
checked through the maintained Destiny controller.

The initial subset is intentionally restricted to semantics-preserving Boolean
logic, integer equality, and direct value selection. Machine arithmetic,
floating point, casts, pointers, allocation, unsafe code, general loops,
recursion, and concurrency are not translated by this bridge. Existing Kani
obligations for those domains remain required.

First use:

```sh
python3 -B verification/tools/bootstrap_verifiers.py --init
python3 -B verification/tools/bootstrap_verifiers.py --check

# With a pinned cargo-kani 0.68.0 already set up:
python3 -B verification/tools/verus_kani_bridge.py discover \
  --cargo-kani "$(command -v cargo-kani)" \
  --output .aegis/destiny/verus-kani-automation/discovery.json

# Inspect support and semantic exclusions:
python3 -B verification/tools/verus_kani_bridge.py explain

# After materializing the maintained .agents controller and a pinned Verus binary:
python3 -B verification/tools/run_verus_campaign.py \
  --verus /path/to/verus \
  --automation-discovery .aegis/destiny/verus-kani-automation/discovery.json
```

Explicit Kani/CBMC comparison is available only through
`verus_kani_bridge.py verify --engine kani`; it cannot satisfy a required
Verus result. `replay` currently reports `UNSUPPORTED` because no validated
Verus-model-to-source counterexample decoder exists yet.
