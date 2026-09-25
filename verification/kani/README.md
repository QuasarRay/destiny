# Kani proof layer

Pinned target: Kani 0.68.0.

This crate model-checks the concrete executable specification and
metaprogramming replacements. It intentionally does not claim full Destiny
parity yet: `destiny-rs-dev` still has original-Destiny operations that are
unsupported (for example Orbit/new-orbit).

Current harnesses prove:

- add-during-evolve gating for all modeled park states;
- non-negative setter no-op/acceptance behavior for all finite `f64` inputs;
- speed-fraction clamping for all finite `f64` inputs;
- metaverification equivalence of reusable setter, delegation, and saturating
  counter patterns against handwritten reference forms.

Every additional original test remains an explicit catalog obligation until its
concrete implementation adapter and Kani harness exist.
