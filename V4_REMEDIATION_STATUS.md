# V4 aggressive review and remediation ledger

## Scope and interpretation

The supplied v3 audit contains eight short, recommendation-level documents. It
does not identify a concrete defect, file location, reproducer, severity count,
or per-finding identifier. This review therefore treated its recommendations as
themes, compared the Python contract backend, Rust/Bevy runtime, native ABI,
C++ facade, snapshot paths, and Carbon paths directly, and created concrete v4
findings below.

`LOCAL VERIFIED` means the portable Python/C/C++ gate ran in this workspace.
`SOURCE FIXED; NATIVE GATE OPEN` means Rust source and a native regression were
added, but the exact Rust tree could not be compiled here because Cargo/Rust
were unavailable. An unexecuted gate is never represented as a pass.

Snapshot schema v3 and Carbon schema v2 deliberately remain stable wire formats
inside the v4 release. Changing a release version does not silently break saved
state or deployed message compatibility.

## Supplied-audit recommendations

| ID | Audit recommendation | V4 disposition | Evidence |
|---|---|---|---|
| A3-01 | Establish one canonical validation model. | Implemented as a machine-readable contract with executable cross-language drift checks. | `registry/validation_contract.json`, `tests/test_validation_contract.py` |
| A3-02 | Add stronger/property-based backend differential tests. | Seeded recursive codec, snapshot mutation, canonical round-trip, and geometry properties added. Existing native differential remains a mandatory build gate. | `tests/test_v4_properties.py`, `tests/test_backend_differential.py` |
| A3-03 | Fuzz snapshot import/export and network codec boundaries. | Deterministic malformed-input and recursive-value fuzz regressions added without a new runtime dependency. | `tests/test_v4_properties.py` |
| A3-04 | Harden FFI lifetime, null, double-free, and poison boundaries. | Pointer/length invariants, success diagnostics/status, null/empty free behavior, retain/release, strict JSON, and poison behavior are tested. A second free of a nonempty owned buffer remains caller UB and is explicitly forbidden rather than executed. | `src/ffi.rs`, `include/destiny_bevy_compat.hpp`, `tests/test_native_contract.py` |
| A3-05 | Reduce generated/manual duplication. | Registry generation remains authoritative; validation constants now have one audited contract and a drift gate. | `tools/generate_implementation_status.py`, `registry/validation_contract.json` |
| A3-06 | Separate simulation and external API state. | Logical snapshots and tagged Carbon values retain explicit translation boundaries; no compatibility-breaking redesign was introduced. | `COMPATIBILITY.md`, `src/runtime.rs`, `python/destiny/net/_codec.py` |
| A3-07 | Add performance regression benchmarks. | Kept as a release optimization item, as the supplied audit itself marks performance P2 and says to postpone it until correctness. Hard work/size ceilings are enforced now. | `RELEASE_GATES.md`, `registry/validation_contract.json` |
| A3-08 | Avoid a large Bevy/API redesign before parity. | Followed. V4 is a hardening release with stable ABI/snapshot/Carbon schema versions. | `COMPATIBILITY.md` |

## Concrete defects absent from the supplied audit

| ID | Defect found by v4 review | Remediation | Status / regression |
|---|---|---|---|
| V4-001 | Filtered/subset snapshots retained an ego ID for an omitted ball and could not restore themselves. | Serialize ego as zero unless selected. | LOCAL VERIFIED: `test_filtered_snapshots_clear_omitted_ego_and_restore_cleanly`; native test added. |
| V4-002 | Full/rewind snapshot import accepted a nonzero ego that did not reference an imported ball. | Validate before mutation in both backends. | LOCAL VERIFIED; native test added. |
| V4-003 | A cloaked snapshot could omit `massive_before_cloak`, making uncloak behavior unknowable. | Enforce the cloak-state invariant. | LOCAL VERIFIED; native test added. |
| V4-004 | Partial selector values such as `True`, `1.0`, or `"1"` were coerced into restore modes. | Require exact integer modes 0, 1, or 2. | LOCAL VERIFIED. |
| V4-005 | Partial-1 rewind preserved sensor members referencing future balls removed by the rewind. | Prune preserved members against the committed entity set. | LOCAL VERIFIED; native source parity fixed. |
| V4-006 | Partial-2 replacement scrubbed valid references as though the logical ball ID had been deleted. | Replace without related-state deletion. | SOURCE FIXED; NATIVE GATE OPEN. |
| V4-007 | Native partial restore could start mutating an externally inconsistent ECS world. | Preflight authoritative components before any partial mutation. | SOURCE FIXED; NATIVE GATE OPEN. |
| V4-008 | Snapshot restore renormalized already-unit quaternions and accumulated one-ULP drift. | Preserve already-normalized f64 components within a tight epsilon. | LOCAL VERIFIED canonical 64-ball round trip; native source parity fixed. |
| V4-009 | Deeply nested JSON and Unicode-surrogate failures escaped normalized protocol errors. | Normalize recursion/Unicode failures at stream, request, and codec boundaries. | LOCAL VERIFIED. |
| V4-010 | Rust accepted duplicate keys inside `serde_json::Value` subtrees even when typed outer structs were strict. | Added a recursive duplicate-rejecting JSON deserializer for options, requests, and snapshots. | SOURCE FIXED; nested-duplicate native test added. |
| V4-011 | Snapshot and Carbon binary tags accepted noncanonical base64 spellings. | Decode, re-encode, and compare before acceptance. | LOCAL VERIFIED; native source parity fixed. |
| V4-012 | Snapshot adapters used loose encoded-size allowances before base64 decoding. | Enforce the exact checked `4 * ceil(n/3)` ceiling in Rust and Python. | LOCAL VERIFIED; native gate open. |
| V4-013 | Native runtime options could raise resource caps to `usize::MAX`, bypassing advertised hard limits. | Enforce immutable upper ceilings for snapshot, child, and outbox resources. | SOURCE FIXED; cross-language drift gate passes. |
| V4-014 | Closest-point triangle math overflowed at otherwise allowed 1e100 coordinates. | Scale barycentric calculations before dot products. | LOCAL VERIFIED; native test added. |
| V4-015 | Zero-angle cones were undefined and tiny valid angles overflowed `radius / sin(angle)`. | Define zero angle as the axis capsule and use quotient-free algebra for tiny angles. | LOCAL VERIFIED; native test added. |
| V4-016 | Visibility ignored global blockers assigned to a different bubble ID. | Treat global massive balls as cross-bubble occluders. | LOCAL VERIFIED; native test added. |
| V4-017 | `ScanCone` accepted negative angles while related cone queries rejected them. | Reject negative angles consistently. | LOCAL VERIFIED; native source parity fixed. |
| V4-018 | Extreme finite friction manufactured an infinite Avian damping coefficient. | Stage exact underflow-to-zero velocity without an infinite solver coefficient. | LOCAL VERIFIED in contract backend; native test added. |
| V4-019 | Native analytic displacement used the unclamped initial velocity while Python used the speed-clamped velocity. | Use the same clamped initial velocity for native decay and displacement. | SOURCE FIXED; NATIVE GATE OPEN with regression. |
| V4-020 | The physics rollback checkpoint omitted collider, gravity, rigid-body, mass/speed, pending-removal, queue, and clock state. | Expand capture/restore and authoritative post-step validation. | SOURCE FIXED; NATIVE GATE OPEN. |
| V4-021 | A rejected native step claimed recoverability although Avian solver caches are not serialized. | Restore exposed authoritative state, pause, and terminally disable the runtime rather than continue from unproven caches. | SOURCE FIXED; NATIVE GATE OPEN. |
| V4-022 | Uncloak could retain a stale derived cloak sensor, and notification could precede a failed transition. | Scrub derived sensors and emit only after success. | LOCAL VERIFIED / SOURCE FIXED. |
| V4-023 | Scalar quaternion component setters could partially mutate before normalization failed. | Stage the complete quaternion, normalize, then commit. | LOCAL VERIFIED. |
| V4-024 | Sensor replacement could exceed child-count or combined radius limits. | Preflight the replacement and total live descriptor budget. | LOCAL VERIFIED; native source parity fixed. |
| V4-025 | Facade getters and query/event paths trusted malformed backend handles, vectors, IDs, booleans, ranges, and nonfinite numbers. | Validate query, event, handle, and property result shapes/types/ranges before exposing them or mutating caller vectors. | LOCAL VERIFIED. |
| V4-026 | Public IDs, timestamps, booleans, and source lists silently accepted floats, strings, or truthy objects. | Enforce exact signed-int and compatibility-bool domains with bounded iterables. | LOCAL VERIFIED. |
| V4-027 | A backend that allocated a runtime but returned a malformed park handle leaked facade/backend ownership. | Validate inside the construction transaction and release on failure. | LOCAL VERIFIED. |
| V4-028 | Python ignored native ABI status codes and success-time diagnostics. | Cross-check status/envelope consistency, reject invalid buffers, and free a just-created invalid runtime. | LOCAL VERIFIED through executable guard doubles; linked native gate open. |
| V4-029 | C++ request construction did not JSON-escape dynamic strings and accepted concatenated raw JSON values. | Escape strings and require exactly one raw JSON value. | LOCAL VERIFIED by compiled executable. |
| V4-030 | C++ response parsing tolerated duplicate/unknown or contradictory envelope fields. | Enforce exact success/error envelope structure. | LOCAL VERIFIED by compiled executable. |
| V4-031 | C++ buffers treated invalid pointer/length pairs as empty and could hide ABI corruption. | Add an explicit validity invariant and fail before copying. | LOCAL VERIFIED by compiled executable. |
| V4-032 | Native fanout decoded then reserialized Carbon rows, erasing tuple tags, including nested tuples. | Validate decoded values but route/redact the original tagged representation. | SOURCE FIXED; NATIVE GATE OPEN with regression. |
| V4-033 | Malformed queued network rows were accepted and failed only during later delivery. | Validate action/state/recipient structure before outbox mutation. | LOCAL VERIFIED / SOURCE FIXED. |
| V4-034 | Narrowcast expansion could clone an excessive recipient-row product. | Enforce aggregate expanded-row and byte budgets before continued cloning. | LOCAL VERIFIED; native test added. |
| V4-035 | Recreating a Python network adapter restarted batch IDs and caused replay rejection. | Allocate monotonically per backend/runtime, not per short-lived adapter. | LOCAL VERIFIED. |
| V4-036 | Client state accepted multiple or nonleading `SetState` actions. | Require at most one and only at expanded index zero. | LOCAL VERIFIED. |
| V4-037 | A same-timestamp `SetState` was appended behind queued ordinary actions and never applied. | Make the replacement supersede the old same-timestamp group. | LOCAL VERIFIED. |
| V4-038 | Client history, packaged actions, state history, and catch-up work were effectively unbounded or shallowly exposed. | Add count/byte/tick caps and deep isolation. | LOCAL VERIFIED. |
| V4-039 | Nonboolean `wait_for_bubble` values silently changed ticker control flow. | Require an exact boolean and fail closed. | LOCAL VERIFIED. |
| V4-040 | Rebase acknowledgement had no sequence watermark, allowing delayed pre-rebase frames after a fresh state. | Require an advancing per-recipient `through_batch_id` when acknowledging. | SOURCE FIXED; NATIVE GATE OPEN with regression. |
| V4-041 | Malformed matching-recipient inbound frames did not consistently force rebase. | Clear bounded inbox state, retain replay watermark, and require rebase. | SOURCE FIXED; NATIVE GATE OPEN. |
| V4-042 | Duplicate server actions still emitted a false history-addition signal. | Emit only when a row is actually appended. | LOCAL VERIFIED. |
| V4-043 | Client/server timestamps could be negative or overflow on next-tick calculation. | Enforce signed range, nonnegative state stamps, and checked increment at both boundaries. | LOCAL VERIFIED. |
| V4-044 | Bubble membership cache keyed only by tick and missed same-tick entity changes. | Include a deterministic membership fingerprint. | LOCAL VERIFIED. |
| V4-045 | Returned history objects shared nested mutable state with authoritative queues. | Deep-copy on ingress and egress. | LOCAL VERIFIED. |
| V4-046 | FFI null, empty-free, retain/release, duplicate JSON, and poisoned-runtime invariants lacked direct native tests. | Add contained Rust unit tests and executable C++ guards. | C++ LOCAL VERIFIED; Rust NATIVE GATE OPEN. |
| V4-047 | Portable `PackagedAction` JSON silently collapsed duplicate object keys before Carbon validation. | Parse packaged JSON with a duplicate-rejecting object hook. | LOCAL VERIFIED. |
| V4-048 | Native proximity backpressure events omitted the required-event count present in the contract backend. | Emit the same bounded event schema in both implementations and validate it at the facade. | SOURCE FIXED; native gate open. |
| V4-049 | Native velocity setters wrote linear velocity before discovering missing orientation components. | Preflight the complete mutation and commit velocity/rotation state together. | SOURCE FIXED; native gate open. |
| V4-050 | Facade `AddBall`/`GetBall` accepted a well-formed handle for a different requested ball ID. | Bind returned handles to the exact requested ID before constructing a facade object. | LOCAL VERIFIED. |
| V4-051 | Snapshot emission could serialize or silently prune externally corrupted live descriptors that its own importer would reject. | Validate selected live balls, aggregate descriptor bytes, and sensor references before encoding in both backends. | LOCAL VERIFIED; native regression added. |
| V4-052 | Native snapshot emission preserved sensor-member insertion order while the contract backend emitted canonical sorted membership. | Sort retained member IDs after visibility pruning. | SOURCE FIXED; native regression added. |
| V4-053 | Source-filtered native snapshot generation silently omitted a candidate whose authoritative metadata lookup failed. | Propagate the component error instead of disguising corruption as visibility filtering. | SOURCE FIXED; native gate open. |
| V4-054 | A direct native snapshot selector could exceed the configured ball-count limit before sort/dedup work began. | Bound selector length before allocation-heavy normalization. | SOURCE FIXED; native gate open. |
| V4-055 | Replacing a scheduled native ball canceled its pending removal before fallible replacement work completed. | Preflight the current components, collider plan, and orientation before mutation; cancel lifecycle state only after commit. | SOURCE FIXED; native regression added. |
| V4-056 | Delayed native removal could zero linear velocity before discovering a missing angular component. | Stage metadata and commit all motion, collider, lifecycle-component, and queue state together after preflight. | SOURCE FIXED; native regression added. |
| V4-057 | Native ball removal scrubbed sensor owners incrementally, so a corrupt later owner could leave an aborted removal partially applied. | Clone and validate every owner first, then commit all scrubbed metadata. | SOURCE FIXED; native regression added. |
| V4-058 | Removing an absent native ball ran related-state cleanup before proving the target existed. | Resolve the target entity before staging any sensor or event cleanup. | SOURCE FIXED; native gate open. |
| V4-059 | Snapshot import and facade capture accepted negative authoritative tick stamps even though live evolution begins at zero and only increments. | Reject negative `current_time` on import and reject negative backend capture/property results. | LOCAL VERIFIED; native regression/source parity added. |
| V4-060 | The facade did not verify that an atomic capture label matched the timestamp embedded in the returned snapshot. | Strictly parse the bounded snapshot envelope, reject duplicate fields, and compare the embedded and outer ticks before returning bytes. | LOCAL VERIFIED. |
| V4-061 | Carbon batch IDs advanced before submit acknowledgement, so a commit-then-response-loss retry used a new ID and could duplicate a tick. | Hold the same batch ID across uncertain failures and advance only after acknowledgement. | LOCAL VERIFIED. |
| V4-062 | Backends rejected even an exact retry of the last accepted Carbon envelope, preventing recovery with the stable idempotency key. | Acknowledge an exact last-envelope retry without requeueing and reject conflicting reuse of that ID. | LOCAL VERIFIED; native regression/source parity added. |
| V4-063 | Python retry fingerprints treated JSON object insertion order as content, unlike JSON semantics and native value equality. | Fingerprint canonical key-sorted JSON so reordered but equal objects reuse the same idempotency key safely. | LOCAL VERIFIED. |
| V4-064 | The client state boundary accepted arbitrary iterators and could consume an unbounded or nonterminating source before enforcing its action cap. | Require a concrete list/tuple and enforce the top-level count before expansion. | LOCAL VERIFIED. |

## Remaining release gates, not known source defects

- Compile, format, clippy, and run all Rust tests on the exact v4 archive.
- Run linked C/C++ ABI, install/relocation, wheel, RustSec, cargo-deny, and
  two-process authenticated Lightyear scenarios on their required platforms.
- Expand randomized native/Python differential coverage from the shipped
  scenario to every supported operation before a production compatibility
  claim.

These are explicitly recorded in `RELEASE_GATES.md` and `VERIFICATION.md`.
