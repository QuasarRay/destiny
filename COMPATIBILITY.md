# Compatibility contract

The complete migration targets Original Destiny at
`114e89fa584f0e1a81eb0a679aabf41af8870072`.
[Latest-source migration records](verification/migration/README.md) retain the
additional partition regression and all prior Rust work. The differences below
are historical compatibility boundaries and remain open parity obligations.
They are not authorized exclusions from the complete migration.

Distance queries now use a generated adapter to the existing Rust source-order
binary64 norm, checked against the original header. `GetBoxCenter` returns an
original-style Python tuple and Boolean `isCloaked` assignments are represented
as integers at the existing ABI boundary. These scoped corrections do not
establish complete numeric, controller, private-format or platform equivalence.
Uncloaking now guards the massive-state and sensor-membership changes with the
existing source-backed proper-warp predicate. Full cloak/sensor semantics remain
separate obligations.

Generated source-owned yaw/pitch/roll and six prior/angular-rate fields now live
in Bevy ball metadata. The command-time snap and ordered post-motion update
are distinct from quaternion presentation. Formation uses actual loaded offsets
and the retained source rotation/thrust kernels; both original ten-step traces
pass directly through Rust. These scoped results leave dynamical orientation,
collision-history correspondence and complete controller domains open.

Missile thrust now preserves the source initial/late phase boundary and reads
actual world friction, duration and integer tick milliseconds. Existing source
admission/launch/field kernels are reused. All four original missile scenarios
pass through the native library and direct headless worlds; independent C++
samples check the numeric transition and non-default-angle launch/query paths.
Retained-ball rotation reads the preserved scalar angles through the same
kernel as attached balls. Full interaction, callback, target-loss and supported
configuration coverage remains open.

The active milestone is the standalone world engine. Original Blue hosting and
private representation are later adapter work; world semantics exposed through
those boundaries remain core requirements. Mixed fixtures retain linked core
and adapter obligations in the same queue. Unknown semantics block acceptance.

## Mapping, reachability, and conformance are separate

`registry/title_mapping.json` records whether a stock Bevy-family API resembles
a Destiny title. `registry/implementation_status.json` records whether the
shipped Python objects and Rust dispatch handlers actually expose it. The
latter is generated from static Python object resolution and Rust handler
extraction, with small explicit semantic annotations.

A mapping verdict is not implementation proof. A reachable wrapper is not
automatically byte-compatible or behaviorally identical. Unsupported dynamic
access raises `UnsupportedTitleError` with the mapping metadata.

`registry/validation_contract.json` is the canonical cross-language limit and
invalid-state policy. `tests/test_validation_contract.py` fails if the shipped
Python or Rust constants drift from it.

## Runtime differences

- Avian f64 `Position` and `Rotation` own their native state; the nine original
  scalar angular fields have their separate source-defined owner in
  `DestinyBallMetadata.source_angles`. Bevy `Transform` is a
  bounded f32 presentation projection; Avian-to-Transform synchronization is
  disabled and the wrapper updates it explicitly.
- `DestinyMass` keeps the f64 API value. Avian's mass component is a bounded
  f32 solver projection: positive underflow clamps to `f32::MIN_POSITIVE`,
  overflow clamps to `f32::MAX`, and exact zero retains Avian's zero sentinel.
- Space friction follows the supported exponential free-flight contract.
  Existing source collision paths coexist with Avian fallback behavior;
  complete contact-response correspondence remains unresolved.
- `DestinyBallMetadata.source_contacts` retains source `mCollisions` IDs in
  order, including duplicates, and full logical snapshots preserve that list.
  Live/free behavior admission clears it with source force state. The gradient
  kernel populates it before source angle updates; Avian contact flags do not
  select the source warp direction. Iterative location IDs remain a distinct
  projection. Moved-missile late append, owner self-collision and synchronous
  per-body `DoCollision` dispatch are still required migration work.
- The direct Rust source dispatcher captures list size once and reads current
  contact IDs and all three force bit patterns between typed host notices.
  Nested calls own independent cursors. Normally returning host failure emits
  an immediate error notice and continues. Cursors reject other worlds and
  cancel replaced owners; list shrink reports a Rust error for an unsafe source
  case. These APIs do not yet provide automatic per-missile tick dispatch or
  complete Original PyOS/error-unwinding correspondence.
- Master bubble IDs filter Destiny-to-Destiny collision pairs. Global balls
  can collide across bubbles. Client pairs ignore bubble labels, and
  pending-removal entities are filtered out.
- Static balls use compound mini geometry. Dynamic balls retain the descriptors
  but use their root sphere.
- Client evolution preserves uninitialized bubble IDs until explicit source
  initialization. Master insertion maintains source partition labels and its
  bubble counter. Source collision kernels admit all numeric bubble labels;
  client collision eligibility does not depend on their numeric values. Full
  master dead-bubble/keep-alive scheduling and partition correspondence remain
  open migration obligations.
- `ApplyImpulsiveForceAtPosition` changes a nonreplicated presentation angular
  component, never authoritative linear or angular motion.
- Implemented source goto/follow/orbit/formation/missile thrust consumes `speedFraction`.
  Complete controller domains and interactions remain parity obligations.
- `DestinyMaxVelocity` owns the source maximum-velocity parameter, including
  getters/setters, snapshots, checkpoint restore and optional replication.
  Source thrust reads it; balls may retain greater actual speed during stop
  decay. The runtime no longer installs Avian `MaxLinearSpeed` for this field.
  API storage remains f64 and source motion narrows to f32; full source API
  rounding and domain correspondence remain open.
- Proximity sensors are a deterministic bounded compatibility scheduler, not
  Avian `Sensor` colliders or Avian contact events.
- `RotationalAgility` has one f32 metadata owner, including retained detached
  generations. Raw property writes preserve zero, negative and nonfinite state;
  the source dynamic constructor has its separate nonpositive-to-one rule.
  Complete dynamical-orientation worlds and dynamic snapshot restore remain
  explicitly unsupported.

## Query and lifecycle policy

Pending-removal, cloaked, unassigned, and cross-bubble entities are excluded
where the supported Destiny query contract requires it. Global balls remain
visible across assigned bubbles. Pair distances return null/`None` for missing
balls. Degenerate triangles are rejected consistently, and cone intersection
uses the ported sphere/cone policy.

`GetBubbleMembership` is a single lock-consistent backend operation. Its
`members` rows include noninteractive balls; `interactives` and `observers`
remain separate. Pending, cloaked, and unassigned balls are omitted.

## Snapshot schema v3

Streams contain strict UTF-8 JSON identified by
`destiny-bevy-compat-state-v3`; they are not Destiny private binary packets.
Unknown/duplicate fields, duplicate IDs, invalid references, nonfinite or
solver-unsafe values, unsupported settings, and count/byte overflows fail
before recoverable live mutation.
The nine source angular fields use u64 bit patterns, so snapshots preserve
their nonfinite values, NaN payloads and signed zero. Missing angular fields in
older snapshots default to zero for compatibility; full historical-format
reconstruction and future-state equivalence remain open.
Rotational agility retains numeric JSON compatibility for finite values and
uses `{ "source_f32_bits_v1": <u32> }` for nonfinite values. Old numeric fields
remain readable. The codec preserves signed zero and NaN payloads in the
accepted ordinary-orientation snapshot path. These are logical state records;
Original Blue conversion, stream layout and ABI remain separate obligations.

- Full mode `0` replaces park metadata, lifecycle, and the entity set.
- Partial mode `1` is rewind mode: it restores time/lifecycle and removes
  entities absent from the checkpoint while preserving child descriptors for
  entities present on both sides.
- Partial mode `2` is a same-tick merge: it merges incoming entities, child
  descriptors, and their pending-removal lifecycle without rewinding park
  time or deleting unrelated entities.
- Pending removals are authoritative snapshot data.
- Network/proximity queues are ephemeral and cleared by full/rewind restore.

The schema is explicitly `logical-authoritative`: it does not serialize Avian
broad-phase caches, contact manifolds, sleeping islands, or other solver
transients. A rejected native step restores exposed authoritative components
and clocks, pauses physics, and terminally disables that runtime; it never
claims that uncheckpointed solver caches are safe to reuse. A Rust panic also
poisons the runtime.

## Carbon wire boundary

The portable protocol is `destiny-carbon-update` schema v2. Tuples and bytes
use reserved canonical JSON tags with depth/node/string/binary limits. Input is
never unpickled. This is not byte-compatible with private Carbon/Blue marshal;
a product needing that format must provide an isolated audited decoder.

Only `singlecast`, `narrowcast`, and atomic `batch` envelopes are accepted.
Positive batch IDs must increase monotonically within a runtime epoch; full or
rewind replacement starts a new epoch. A sender retains its current ID until
submission is acknowledged. The backend acknowledges an exact retry of the
last accepted envelope without queueing it twice, but rejects reuse of that ID
with different content. Each Rust delivery frame contains one
authenticated recipient ID. Narrowcasts are split and redacted per link. The
plugin synchronizes global, bubble, and personal relevance rooms, validates
the complete bounded canonical value tree and every row recipient on receive,
and rejects duplicate or stale batches even after the bounded dedup window.
Inbox overflow requires an authenticated full-state rebase. A host acknowledges
it with `acknowledge_rebase(recipient_id, through_batch_id)` so no delayed frame
from before that watermark can be accepted after replacement. Oversized,
malformed, unknown-recipient, or duplicate-binding envelopes are dropped
before the first send and counted rather than permanently blocking the global
outbox.

The product still owns authentication, authorization, certificates, link
lifecycle, underlying transport backpressure/replay policy, reconnection, and
the final call from a drained inbox into its ticker.

## Explicit non-equivalents

Orbit/follow/warp/missile/formation/boid/troll/mushroom controllers, private
Blue vtables, private stream bytes, and standalone runtime lifecycle for
`Capsule`/`OrientedBox` are not implemented. Module-level geometry objects are
immutable descriptors; runtime mini geometry is created through `Ball.AddMini*`.
