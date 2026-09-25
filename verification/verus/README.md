# Verus proof layer

Pinned release target: `release/0.2026.09.20.aef82ed`.

The Verus layer proves logical/refinement properties of the original-Destiny
specification without importing Bevy, Avian, Lightyear, or Replicon.

Verus and Kani intentionally have different jobs:

- **Verus** proves mathematical state/refinement properties: lifecycle guards,
  follow/orbit admissibility, visibility/proximity predicates, stop semantics,
  and abstract setter/clamp laws.
- **Kani** proves concrete executable Rust behavior, including `f64` branch
  semantics and metaprogramming replacements.

For floating-point behaviors, a Verus theorem uses an abstract value or a
predicate such as `range_is_finite`; the concrete link from Rust `f64` to
that predicate belongs to Kani. This avoids treating an abstract integer theorem
as if it were an IEEE-754 execution proof.
