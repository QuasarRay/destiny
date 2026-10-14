# Verus-default / Kani-assisted automation slice

This directory is an intentionally narrow vertical slice of the requested
Verus/Kani integration. It does not claim general Kani compatibility,
translation soundness, or Destiny parity.

## Architecture

The default flow is:

1. cargo kani autoharness -Z autoharness --list --format json performs Kani
   compiler discovery and automatic-harness eligibility analysis.
2. Upstream Kani defines --list as --only-codegen; this path does not accept a
   Kani/CBMC proof result.
3. verification/tools/verus_kani.py structurally parses only the subset in
   capabilities.json.
4. It lowers source assert! sites and control flow to generated Verus source
   under .aegis/destiny/verus-kani/<identity>/.
5. The pinned Verus executable checks one selected generated entry. Success is
   accepted only through Aegis's existing strict checked_output parser.

Kani remains a separate explicit verification backend elsewhere in Destiny.
This tool never treats Kani verification as fallback for failed Verus.

## First use

Materialize and validate the immutable sources:

    python verification/tools/vendor_verifiers.py --materialize
    python verification/tools/vendor_verifiers.py --verify-only

After provisioning the pinned Rust toolchains and Cargo dependencies, an
offline source build is:

    verification/tools/build_vendored_verifiers.sh --offline

With Kani 0.68.0 and the pinned Verus binary on the host:

    python verification/tools/verus_kani.py doctor --verus /path/to/verus
    python verification/tools/verus_kani.py discover \
      --manifest-path verification/automation/fixtures/valid/Cargo.toml
    python verification/tools/verus_kani.py verify \
      --manifest-path verification/automation/fixtures/valid/Cargo.toml \
      --function branch_valid --verus /path/to/verus
    python verification/tools/verus_kani.py explain

verify defaults to engine Verus. The Kani proof engine is intentionally not a
hidden fallback; existing project Kani lanes remain the explicit reference
backend.

## Supported semantic bridge

The initial bridge accepts free, non-generic, unit-returning functions whose
parameters are independent bool or fixed-width signed/unsigned integer values.
Statements are assert!, if/else, and literal for _ in A..B ranges. Expressions
are Boolean connectives and scalar comparisons. Literal loops are exactly
unrolled, with a maximum of 32 iterations.

Arithmetic, assumptions/contracts, function calls, references, pointers,
slices, allocation, ADTs, floats, unsafe code, concurrency, dynamic loops and
recursion are rejected. Mathematical integers/reals are not silently
substituted for machine integers/floats.

The bridge itself is a trusted correspondence boundary. Parser acceptance,
generated-source hashes, source hashes, Kani/Verus pins, explicit bounds and
assumptions are recorded in evidence. A future extension must add a regression
and preservation argument before advertising more syntax.

## Failure and replay semantics

checked_valid means only that the exact generated obligation completed under
Verus and passed the strict counted-output check. A proof_failure is not
automatically a concrete source bug. replay therefore refuses to fabricate a
source trace unless a separately validated concrete witness exists.

The fixture crates exercise a valid branch/bounded-loop case, an invalid
universal assertion, and unsupported floating-point syntax.
