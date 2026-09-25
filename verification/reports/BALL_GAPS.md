# Original Ball contract parity and gaps

Specification source: original Destiny main, especially:

- python/destiny/test/test_ball.py
- src/Ball.cpp

## Paired Kani + Verus specifications

The following original tests now have both Kani and Verus proof markers and
engine-independent contracts:

- test_can_construct
- test_default_values
- test_can_add_miniball
- test_can_add_minicapsule
- test_can_not_add_minicapsule_with_negative_radius
- test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation
- test_add_proximity_sensor
- test_reserve_formation_slot_returns_negative_one_when_ball_is_not_set_up_properly
- test_reserve_formation_slot
- test_slots_are_reserved_in_incremental_order
- test_reserving_too_many_formation_slots_fails
- test_free_formation_slot

## Concrete Rust runtime bridge

Public compatibility-surface regressions now cover:

- identity rotation of an unrotated ball;
- successful mini-ball insertion;
- successful positive-radius mini-capsule insertion;
- transactional rejection of non-positive mini-capsule radius;
- successful proximity-sensor insertion and serialized state.

The regression inspects the serialized Destiny snapshot rather than Bevy ECS
components, so the parity assertion is not tied to Bevy representation details.

## Still implementation-missing

| Original behavior | Status | Reason |
| --- | --- | --- |
| standalone Ball construction/default object | ProofMissing | The original Ball default state is specified and proved, but the Rust core primarily represents balls inside CompatRuntime. A concrete retained standalone-Ball adapter still needs equivalence coverage. |
| ReserveFormationSlot / FreeFormationSlot | ImplementationMissing | The original first-free slot semantics are formally specified and proved, but the Rust runtime has no formation-slot API/state yet. |
| LoadFormations / SetBallFormation bridge | ImplementationMissing | Required before the formation-slot proofs can be connected to the new implementation. |

## Source-level behavioral mismatches beyond the current tests

These are not discharged by the happy-path tests and must remain visible for
full implementation equivalence:

1. Original PyAddMiniBall performs no positive-radius guard before AddMiniBall;
   the Rust mini-sphere path currently rejects radius <= 0.
2. Original PyAddMiniCapsule rejects radius <= 0, but otherwise forwards the
   endpoints; the Rust path additionally rejects coincident endpoints.
3. Original AddProximitySensor accepts the period value supplied by the Python
   wrapper; the Rust path additionally requires period > 0.

These stricter Rust checks may be desirable hardening, but they are behavioral
differences from the original and therefore cannot be called identical until
the compatibility policy explicitly resolves them.
