# Destiny-rs migration

The current migration target is Original Destiny `main` at
`114e89fa584f0e1a81eb0a679aabf41af8870072`, pinned from the newest remote
source on 16 October 2026. Existing Rust work was preserved while updating
`destiny-rs-dev` to `8bf4fad`. The implementation remains incomplete.

[Migration records and reproducible commands](verification/migration/README.md)
retain all 532 original test definitions, the twelve original configuration
presets, generated/reused provenance, concrete failures and separate assurance
categories. A complete behavioral-feature denominator and full required proof
and release matrix are still unresolved. The canonical queue remains under
`.aegis/destiny/work-queue.json`.

The local migration adds exact source-order distance queries through a
generated adapter to existing Rust math, and source-backed Python tuple/input
representation corrections. A recorded syntax transformation reuses the
existing proper-warp guard when uncloaking, preserving its mass state.
Generated source angular state and formation wiring now run in the actual
Bevy world. Nine binary64 angle/history fields and roll inputs survive snapshot
restore, including exact nonfinite bit patterns. Both original ten-step
formation traces run as direct Rust tests without Python or Blue.
Source stop motion now preserves velocities above the maximum-velocity
parameter. `DestinyMaxVelocity` owns that parameter separately from Avian's
hard speed limit, and source collision computation precedes Avian damping
preparation. Independent unchanged C++ stop/integration samples check actual
world position and velocity bits in both collision configurations.
Aligning warp now receives the original goto thrust; the ten-step original
position and mode assertions pass through the headless Rust runtime.
Missile steering now uses the actual world clock, friction and duration. The
four original launch/20-step/collider scenarios and independent C++ numeric
cases run in the real Bevy world. Rotational agility has source f32 storage,
bit-preserving snapshots and retained-generation access. Stationary launch and
retained-object rotation share the existing source scalar-angle kernel,
checked with the pinned Carbon Math/DirectXMath dependencies.
Master bubble liveness now preserves separate keep-alive registration and
group membership, cached motion-phase skips and the final dead-bubble sensor
check/clear. Typed membership APIs share the existing tracker and full logical
snapshots. Independent unchanged C++ methods use real CPython sets only in the
validation oracle. Full collision history and all source/configuration/proof
gates remain open.
Source controller/collision forces now persist in each existing ball metadata
component. Live/free control resets them; dead/global neighbours retain them
for collision prediction, and snapshots preserve their binary64 bits. Three
unchanged-C++ scenarios and actual headless checks cover force flow and replay.
Source contact IDs now persist in the existing metadata owner and full logical
snapshots, preserving order and duplicates. Gradient contacts reach source warp
orientation before publication; iterative location IDs remain a separate
projection until the later missile phase. The existing C++ oracle checks 130
ordered gradient IDs and both source contact states for warp angles. Six direct
headless contact checks cover these paths, snapshot defaults and admission
reset/retention. Complete location histories, missile collision callback timing
and mandatory proofs remain open. See the
[force records](verification/migration/force_source_evidence.json) and
[contact records](verification/migration/contact_source_evidence.json).
The source dispatcher is also available through the typed Rust API:
`begin_source_collision_pass`, `next_source_collision` and
`dispatch_source_collisions`. An operation captures its contact count once,
rereads IDs/force bits between callbacks, and permits nested host mutation.
A false observer result emits an immediate host-error notice and continues.
Six unchanged-C++ cases cover 319 contact notices and three error notices.
Automatic moved-missile publication/FFI scheduling must still be connected to
this same dispatcher; it is not implemented by the direct API alone.
[Generated architecture views](docs/architecture/overview.md)
and [decisions](docs/adr/adr-001.md) describe their actual scope and open gaps.

The active milestone is the complete standalone Rust/Bevy world engine.
Original Blue/Blueexposure/exefile hosting, legacy wheels and authenticated
transport integration follow core acceptance. Their world-state and transition
semantics remain core requirements. Build and exercise the current headless
implementation with `cargo test --no-default-features --locked --offline`.
`python verification/tools/update_migration.py acceptance --scope core`
and `--scope full` retain separate incomplete reports from the same canonical
inventory. The present core report rejects unknown semantics, incomplete
inventory and missing required evidence; no production-ready engine is claimed.

## Historical compatibility v4 boundary

The earlier subset description below explains the v4 compatibility design.
Its accepted differences remain migration gaps under the complete parity
contract; the original title catalog and wrapper reachability are not the
full product denominator. Current execution results and blockers are recorded
in the migration records linked above.

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

See `registry/implementation_status.json` for the historical title ledger.
The current migration records above track the newer source mechanics and
their remaining correspondence gaps. Unsupported operations fail explicitly.

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

`speedFraction` is validated, replicated, snapshotted and consumed by the
implemented source steering paths; complete controller domains remain open.
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
