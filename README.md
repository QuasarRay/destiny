# Destiny–Bevy–Carbon compatibility v4

This project exposes a supported subset of the Destiny API over a headless
Bevy 0.19.1 / Avian 0.7.0 runtime. It ships Python, stable C, and C++17
wrappers plus Carbon-style client/server history and batching modules. An
optional Rust plugin connects the same state to Lightyear 0.29.0 and Bevy
Replicon 0.42.3.

This is a compatibility layer, not a claim that all 389 catalogued Destiny
titles have stock Bevy equivalents. The generated implementation ledger is
derived from the shipped Python objects and Rust dispatch handlers:

| Status | Titles |
|---|---:|
| Tested conformant call/result contract | 2 |
| Native implementation with a documented difference | 92 |
| Native state only; no controller effect | 2 |
| Python/Carbon facade only | 209 |
| Unsupported | 84 |

See `registry/implementation_status.json` for every title. Unsupported
controllers fail explicitly; the wrapper does not approximate orbit, follow,
warp, missile, formation, boid, mushroom, or troll behavior with unrelated
physics primitives.

## Implemented runtime subset

- One Bevy `App`/`World` per `Ballpark`, with an Avian rigid body and f64
  authoritative pose for each ball.
- Legacy 17-argument `Ballpark.AddBall`, replacement by ID, delayed removal,
  re-add cancellation, and deterministic standalone bubble assignment.
- Fixed-step evolution, pause/start, a failure-reporting Python driver, and a
  host-driven C++ `Update()` entry point.
- Destiny space damping integrated through Avian damping without pre/post
  scaling collision impulses. Collision-free motion receives an analytic
  trajectory correction.
- Bubble-aware Avian collision hooks, solver-safe numeric envelopes, live-ball
  and descriptor caps, and rollback of authoritative state/time after a
  rejected physics step.
- Static f64 compound mini-sphere, mini-capsule, and mini-box geometry.
- Bounded deterministic proximity processing, explicit overflow events,
  cloak transitions, supported spatial queries, and visibility occluder IDs.
- Strict logical-authoritative snapshot schema v3, including pending removals,
  rewind time, and future-entity removal.
- Versioned canonical Carbon values for tuples/bytes and one atomic tick batch.
- One machine-readable validation/limit contract with Python/Rust drift tests,
  deterministic property tests, and fail-closed native ABI envelopes.

`speedFraction` is validated, replicated, and snapshotted, but it has no
steering effect because the relevant Destiny controllers are unsupported.
Proximity checks use a compatibility scheduler rather than Avian `Sensor`
entities. Both facts are recorded title by title.

## Python quick start

Build a platform wheel with Rust 1.95 and CMake 3.24 or newer:

```bash
python -m pip install build
python -m build
python -m pip install dist/destiny_bevy_carbon_compat-0.4.0-*.whl
```

The wheel places the ctypes native library beside the `destiny` package.
External native libraries are loaded only when explicitly selected with
`DESTINY_BEVY_COMPAT_LIBRARY`.

```python
import destiny

destiny.use_native_backend()
park = destiny.Ballpark(isMaster=True)
ball = park.AddBall(
    1001, 10.0, 2.0, 250.0,
    True, False, True, True, False,
    0.0, 0.0, 0.0,
    5.0, 0.0, 0.0,
    0.5, 1.0,
)
park.friction = 0.2
park.Evolve()
print(ball.x, ball.vx)
park.close()
```

`destiny.use_in_memory_backend()` selects the deterministic contract double.
It is useful for facade tests but is not Bevy execution.

## C, C++, and CMake

`include/destiny_bevy_compat.h` exposes an opaque reference-counted runtime and
owned, bounded response buffers. `include/destiny_bevy_compat.hpp` supplies
RAII. `include/destiny_carbon.hpp` adds typed Destiny-named conveniences while
retaining access to the generic JSON ABI.

```cmake
add_subdirectory(path/to/destiny-bevy-carbon-compat-v4)
target_link_libraries(my_module PRIVATE destiny::bevy_compat)
```

Installed packages support both:

```cmake
find_package(carbon-destiny 0.4 CONFIG REQUIRED)
target_link_libraries(my_module PRIVATE carbon-destiny::destiny)
```

and `find_package(Destiny 0.4 CONFIG REQUIRED)`. Legacy targets `Destiny` and
`destiny` remain available.

## Carbon / Lightyear integration

`DestinyCarbonInteropPlugin` supplies:

- schema-v2 ordered-reliable recipient frames;
- authenticated Carbon-ID-to-link bindings through `CarbonClientIdentity`;
- recipient-specific sends (never broadcast-and-filter);
- redacted narrowcast frames containing only the receiving ID;
- global, bubble, and owner-personal relevance rooms;
- a bounded, monotonic replay-rejecting client inbox with mandatory rebase on
  overflow;
- observable transport and authorization counters.

The embedding product must install its complete Lightyear client/server/link
stack through `CompatRuntime::new_with_app`, authenticate each link before
inserting `CarbonClientIdentity`, maintain `CarbonClientBubbles` and
`CarbonBallOwner`, configure transport queue limits, and drain validated inbox
frames into its Carbon ticker. Unknown or duplicate identities fail closed.

Malformed, oversized, stale, duplicate, unknown-recipient, and
duplicate-binding traffic fails closed. The standalone ABI has no credentials
or product connection. It exposes a bounded inspectable outbox instead of
pretending to authenticate a peer.
This boundary is an intentional part of the API, not hidden interoperability.

## Read before release

- `COMPATIBILITY.md` defines semantic and binary boundaries.
- `registry/validation_contract.json` defines cross-language validation limits.
- `V3_REMEDIATION_STATUS.md` accounts for all 67 v2 audit findings.
- `V4_REMEDIATION_STATUS.md` records the supplied v3 audit recommendations
  and the concrete defects found by the independent v4 review.
- `VERIFICATION.md` records exactly what was and was not executed here.
- `RELEASE_GATES.md` lists mandatory native, packaging, supply-chain, and
  two-process checks.
- `SECURITY.md` defines trust and resource boundaries.
