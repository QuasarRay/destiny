# Formal verification status

Specification source: original Destiny `main` at
`ef738406b61ad06bae2f402857e464735ab106a2`.

## Invariants

- The specification layer contains no Bevy, Avian, Lightyear, or Replicon types.
- Original tests are catalogued by source file and test name.
- Kani and Verus proof source markers are validated against that catalog.
- A behavior is not called equivalent merely because it has a catalog entry.
- Concrete runtime parity requires an implementation adapter/regression in
  addition to the abstract proof.
- Repeated implementation/proof forms are candidates for macros only after a
  Kani regression establishes equivalence to the handwritten form.

## Current verified families

The stack currently contains paired Kani/Verus specifications for:

- add-during-evolve rejection;
- positive-only and non-negative property setter guards;
- speed-fraction clamping;
- follow self/moribund rejection;
- orbit self/non-finite/cloaked/cross-bubble rejection;
- visibility occluder eligibility;
- proximity eligibility;
- uncloaking massiveness predicate;
- stop-mode normalization;
- missile follow-range construction;
- original Ball default-state constants;
- mini-ball/mini-capsule child-count and capsule-radius rules;
- identity-vector rotation and proximity-sensor acceptance;
- formation-slot first-free ordering, exhaustion, and freed-slot reuse.

The concrete Rust runtime is additionally wired to the formally specified
setter guard policy and contains original-test regressions for ignored
mass/radius/max-speed/max-angular-speed/agility requests, time/distance and
lifecycle membership behavior, plus public-surface Ball regressions for
mini-ball insertion, mini-capsule insertion/rejection, identity rotation, and
proximity-sensor state.

## Explicitly incomplete

Full original-Destiny equivalence is **not** yet established. The original suite
contains 531 catalog entries (509 unique source/test keys). Major remaining
areas include exact Goto/Follow/FormationFollow trajectories, Warp, old/new
Orbit dynamics, missile dynamics, iterative/simple collision behavior,
mini-shape collision behavior, formation runtime state, boxes/bubbles, callbacks, stream serialization,
lifecycle edge cases, network client/server history and ticker semantics, and
the C++ geometry/collision suite.

Some of those behaviors are not implemented in `destiny-rs-dev` at all; for
example the current runtime rejects `use_new_orbit` because Orbit is not
implemented. Those obligations remain visible rather than being discharged by
assumption.

Run:

```bash
python verification/tools/proof_coverage.py
python verification/tools/pattern_scan.py --json
```

The `--strict-both` coverage mode is intentionally available for the final
completion gate, but is not enabled until every catalog obligation has a real
Kani and Verus proof.
