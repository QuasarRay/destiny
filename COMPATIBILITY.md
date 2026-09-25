# Compatibility contract

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

- Avian f64 `Position` and `Rotation` are authoritative. Bevy `Transform` is a
  bounded f32 presentation projection; Avian-to-Transform synchronization is
  disabled and the wrapper updates it explicitly.
- `DestinyMass` keeps the f64 API value. Avian's mass component is a bounded
  f32 solver projection: positive underflow clamps to `f32::MIN_POSITIVE`,
  overflow clamps to `f32::MAX`, and exact zero retains Avian's zero sentinel.
- Space friction follows the supported exponential free-flight contract.
  Contact material response remains Avian behavior.
- Bubble IDs filter Destiny-to-Destiny collision pairs. Global balls can
  collide across bubbles. Pending-removal entities are filtered out.
- Static balls use compound mini geometry. Dynamic balls retain the descriptors
  but use their root sphere.
- The standalone ABI assigns still-unassigned balls to compatibility bubble 0
  after their first evolution. An embedded product can assign real bubbles.
- `ApplyImpulsiveForceAtPosition` changes a nonreplicated presentation angular
  component, never authoritative linear or angular motion.
- `speedFraction` is state-only. No unsupported steering controller consumes it.
- Proximity sensors are a deterministic bounded compatibility scheduler, not
  Avian `Sensor` colliders or Avian contact events.

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
