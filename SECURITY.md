# Security and trust boundaries

This library accepts simulation state and Carbon payloads. It does not own a
product's identities, credentials, certificates, or transport policy.

## Untrusted inputs and resource limits

- C ABI requests, snapshots, canonical Carbon values, IDs, vectors, and child
  descriptors are untrusted.
- Exact schemas reject unknown fields, wrong types, duplicate snapshot fields
  and IDs, unsupported versions, invalid references, nonfinite values, and
  solver-unsafe geometry.
- Request/response, decoded snapshot, live-ball, child-shape, descriptor,
  outbox, expanded-recipient/row, client-inbox, per-client bubble-interest,
  visibility-room, proximity-work, and proximity-event caps bound library work.
  Native configuration may lower, but cannot raise, the hard ceilings in
  `registry/validation_contract.json`.
- Stream reads request bounded chunks and reject readers that ignore the bound
  before copying their result.
- Tuple/byte JSON tags are depth/node/string/binary bounded. Network data is
  never unpickled or evaluated.

Limits bound this library's queues. They do not automatically bound every
product transport buffer beneath Lightyear. Configure link/driver write-buffer
limits, bandwidth, disconnect policy, and process-level quotas separately.

## Native lifetime and failure containment

- A caller must retain its own `DbcRuntime` reference before a final release can
  race its call. Releasing or using a reference that is not live is invalid.
- Every nonnull ABI buffer must be freed exactly once with `dbc_buffer_free`.
- Exported Rust calls catch panics, poison the affected runtime, and reject
  later calls. Panic containment is not transactional recovery.
- Rejected physics steps restore exposed authoritative state and clocks, stop
  physics, and terminally disable that runtime. Solver caches are not
  serialized, so continuing would be an unprovable recovery claim.
- C++ `Ball` uses a weak shared runtime and throws after its park is destroyed.
- Automatic Python driver failures are sticky and surfaced on later calls. A
  timed-out stop retains the thread reference and refuses to start a duplicate.

## Carbon deployment obligations

Only an authenticated host may insert `CarbonClientIdentity`. Duplicate and
unknown identifiers fail closed. A host must also:

- authorize Carbon IDs for the park and maintain `CarbonClientBubbles`;
- assign `CarbonBallOwner` before owner-only cloaked replication is expected;
- install the correct Lightyear client/server/link and Replicon stack;
- configure encryption, transport-level replay protection, reliable-queue,
  bandwidth, and reconnect policy; the compatibility protocol separately
  rejects stale per-recipient batch IDs, treats only the exact last envelope
  as an idempotent retry, and rejects conflicting reuse of its ID;
- drain `CarbonClientInbox` and apply frames through the product ticker;
- acknowledge rebase only after a fresh authenticated full state succeeds,
  supplying the recipient and an advancing through-batch watermark;
- monitor `CarbonTransportMetrics` and treat overflow/rebase as an incident.
- keep Carbon IDs and bubble IDs stable and bounded; Lightyear 0.29 room IDs
  are monotonic u16 values, so this plugin fails closed before allocator
  exhaustion and reports `room_allocation_failures` rather than recycling IDs
  that an external host component might still reference.

The standalone ABI exposes a bounded local outbox and does not claim an
authenticated network connection.

## Release security gates

Run locked Rust compilation/tests, clippy, RustSec, cargo-deny, linked ABI
tests, wheel/install smoke tests, and a real two-process authorization test on
the exact archive. These remain open in this workspace; see `RELEASE_GATES.md`.
