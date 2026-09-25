# Visibility parity and remaining gaps

Specification source:

- python/destiny/test/ballpark/test_visibility.py
- src/Ballpark.cpp
- src/Thunkers.cpp

## Paired Kani + Verus coverage

All tests in test_visibility.py now have paired formal markers covering:

- no occluder -> 0;
- eligible occluder -> occluder ID;
- non-massive and cloaked candidates do not occlude;
- cloak sets cloaked=true and massive=false;
- non-warp uncloak clears cloak and restores massive=true;
- warp uncloak clears cloak without restoring massiveness;
- the exact algebraic cone predicate used by the test's pi/2 full angle;
- inclusion of the +X candidate and exclusion of the tested orthogonal/backward candidates.

## Concrete Rust runtime bridge

Public compatibility-surface regressions cover all of the above except the
warp-specific uncloak case, because Warp is not implemented in the Rust
compatibility subset.

This PR also repairs a concrete mismatch: the Rust implementation previously
returned early when UncloakBall was called on an already-uncloaked ball.
Original Destiny still sets that non-warp ball massive. The Rust non-warp path
now does the same.

## Still implementation-missing

| Original behavior | Status | Reason |
| --- | --- | --- |
| uncloak while actually warping | ImplementationMissing | The current Rust runtime does not implement original Warp mode, so the non-massive-in-warp transition cannot yet be connected to a concrete implementation state. |

## Source-level behavioral differences beyond the original assertions

1. CheckVisibility in original Destiny requires source and destination to be in
   exactly the same bubble. The Rust implementation permits a global
   destination across bubbles.
2. Original occluder filtering also requires the occluder to be in exactly the
   source bubble. The Rust implementation permits global occluders across
   bubbles.
3. Original CheckVisibility returns the first intersecting ball encountered by
   its ball iterator. The Rust implementation sorts blockers by segment
   parameter and then ID, selecting the nearest deterministic blocker.
4. Original ScanCone does not filter candidates by bubble. The Rust
   implementation filters to the source bubble unless a candidate is global.
5. Original ScanCone does not explicitly reject negative angles; it halves the
   supplied angle and evaluates cosine. The Rust implementation rejects
   negative angles as invalid input.

These differences are intentionally not labeled proved parity.
