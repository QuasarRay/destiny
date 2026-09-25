# V3 remediation ledger

This ledger accounts for every finding in the supplied v2 aggressive audit.
It describes the delivered v3 source, not a hypothetical production build.
“Local pass” means the Python/source-contract suite passed in this workspace.
“Native gate” means Rust/CMake execution remains mandatory because those tools
were unavailable here. “Deployment gate” means behavior must be proven with
the embedding product's authenticated Lightyear transport.

No item marked with an open gate is represented as release-verified.

## Network and Carbon — 15 findings

| ID | V2 finding | V3 disposition | Evidence / remaining gate |
|---|---|---|---|
| NET-001 | Recipient-targeted data is broadcast | **Implemented; native and deployment gates open.** The broadcast sender was replaced by an authenticated Carbon-ID-to-link binding and a per-recipient `MessageSender`; narrowcasts are split into recipient-redacted frames. | `src/network.rs`; native tests authored; two-client confidentiality gate in `RELEASE_GATES.md`. |
| NET-002 | Room relevance ignores Destiny visibility | **Implemented; native and deployment gates open.** Global, bubble, and owner-personal `Rooms` are synchronized, with cloaked and pending-removal policy. | `src/network.rs`; real disconnect/reassignment/visibility gate remains. |
| NET-003 | No native client receive/apply path | **Partially closed by an explicit integration boundary.** A bounded receiver, validation, deduplication, and overflow-to-rebase path now exist. The product still owns draining validated frames into its ticker. | `src/network.rs`, `README.md`; deployment gate remains. |
| NET-004 | Fan-out bypasses outbox bounds | **Partially closed.** Standalone outbox, pre-expansion row/recipient/byte, per-flush frame/byte, bubble-interest, and client inbox limits are enforced. A single impossible or unresolved envelope is rejected observably instead of blocking later batches. Product transport/link queues remain host-configured. | `src/network.rs`, `python/destiny/_backend.py`; slow-reader and loss deployment gate remains. |
| NET-005 | Unsupported actions are sent after skip | **Fixed; local pass.** Validation rejects an unsupported action before history mutation or transport. | `python/destiny/net/server/_actions.py`, `tests/test_hardening_contract.py`. |
| NET-006 | Bubble diffs omit noninteractive balls | **Fixed; local pass.** Membership has separate `members`, `interactives`, and `observers`; `members` includes noninteractive balls. | Python and Rust membership handlers; `tests/test_v3_regressions.py`; native gate for Rust. |
| NET-007 | Child/sensor replication claims are dead | **Fixed by removing the false claim and dead component model.** Runtime mini geometry and compatibility proximity state are documented separately. | `COMPATIBILITY.md`, `src/runtime.rs`; source-contract check rejects the old dead type. |
| NET-008 | Rollback/visibility primitives are only re-exports | **Fixed in scope.** V3 implements rooms, bounded inbox, deduplication, and rebase signaling and no longer claims a complete stock prediction/rollback pipeline. | `src/network.rs`, `README.md`. |
| NET-009 | `bubbleInteractives` exposes unassigned/inert balls | **Fixed; local pass.** Membership filters unassigned, pending-removal, and cloaked balls; interactive rows additionally require interaction. | Both backend handlers; `tests/test_v3_regressions.py`; native gate for Rust. |
| NET-010 | Three-call delivery is nontransactional | **Fixed; local pass.** One schema-v2 batch with one batch ID is validated and enqueued atomically. | `python/destiny/net/server/_parkupdatebatcher.py`, `tests/test_v3_regressions.py`. |
| NET-011 | Empty batches emit transport calls | **Fixed; local pass.** Empty sections produce no outbound batch. | Batcher implementation and network regression suite. |
| NET-012 | History getters expose mutable queues | **Fixed; local pass.** Public history access returns copies. | Client/server history implementations and tests. |
| NET-013 | `Signal` is not Carbon-compatible | **Fixed; local pass.** Bound slots are weak, dead slots are removed, and one handler failure does not abort remaining handlers. | `python/destiny/_util/signal.py`, `tests/test_v3_regressions.py`. |
| NET-014 | Default codec is not Carbon's private codec | **Accepted and precisely documented.** V3 uses bounded canonical schema-v2 JSON with tuple/bytes tags and never claims private Blue-marshal byte compatibility. | `python/destiny/net/_codec.py`, `COMPATIBILITY.md`. |
| NET-015 | Timestamp divergence amplifies failure | **Fixed; local pass.** Whole batches are validated before side effects, carry positive monotonic IDs, and stale/replayed IDs are rejected beyond the bounded dedup window; mixed timestamps stop processing after fatal desync. Full/rewind replacement explicitly starts a new batch epoch. | Client/server ticker and transport code plus failure-policy/regression tests. |

## State, snapshots, and rewind — 9 findings

| ID | V2 finding | V3 disposition | Evidence / remaining gate |
|---|---|---|---|
| STA-001 | Invalid `SetState` destroys the park | **Fixed; local pass.** Input is bounded, parsed, and validated before a non-destructive commit; failure preserves live state and invalidates the ticker. | `python/destiny/net/client/_baseticker.py`, `tests/test_v3_regressions.py`. |
| STA-002 | Rewind cannot restore time | **Fixed; local pass.** Snapshot v3 records logical time and partial mode 1 restores it. | Backend snapshot handlers and rewind regression. |
| STA-003 | Rewind retains future entities | **Fixed; local pass.** Partial mode 1 removes entities absent from the checkpoint. | Backend snapshot handlers and rewind regression. |
| STA-004 | Automatic driver races Carbon history | **Mitigated and locally tested.** Ticker-owned scheduling stops the driver first; forced evolution rejects an active driver and driver failure is sticky. Embedders must retain one scheduler owner. | Facade/ticker driver code and v3 driver regressions. |
| STA-005 | Pending removal is absent from snapshots | **Fixed; local pass.** Pending-removal deadlines are authoritative snapshot-v3 data. | Python/Rust snapshot schema; pending-removal regression; native gate for Rust. |
| STA-006 | Snapshots omit physics-transient state | **Accepted and disclosed boundary.** Schema v3 is logical-authoritative, not an Avian cache/manifold/sleeping-island image. | `COMPATIBILITY.md`. |
| STA-007 | “Atomic restore” excludes commit failure | **Fixed for recoverable validation/commit failures; native gate open.** Native evolution checkpoints authoritative components and clocks; a Rust panic poisons the runtime instead of claiming rollback. | `src/runtime.rs`, `COMPATIBILITY.md`; Rust tests must execute. |
| STA-008 | Sensor schema is inexact/nonreferential | **Fixed; local pass.** Strict keys, duplicate detection, exact references, count limits, and child validation precede mutation. | `python/destiny/_backend.py` and strict snapshot tests. |
| STA-009 | Mixed timestamps report fatal but continue | **Fixed; local pass.** The ticker returns immediately after fatal desynchronization. | Client ticker and failure-policy test. |

## Runtime and physics — 15 findings

| ID | V2 finding | V3 disposition | Evidence / remaining gate |
|---|---|---|---|
| PHY-001 | Analytic damping corrupts collisions | **Implemented; native gate open.** Damping is applied through Avian; the analytic correction is restricted to collision-free motion and no longer rescales post-contact velocity. | `src/runtime.rs`; native collision tests authored. |
| PHY-002 | Bubble separation does not isolate physics | **Implemented; native gate open.** Collision hooks reject non-global cross-bubble, pending, and otherwise ineligible pairs. | `src/runtime.rs`; collision tests must execute. |
| PHY-003 | Native evolution lacks preflight/rollback | **Implemented; native gate open.** Complete damping/numeric preflight occurs before mutation; checkpoints cover authoritative components and fixed/physics/substep clocks. | `src/runtime.rs`; rollback tests authored. |
| PHY-004 | Avian overwrites safe visual projection | **Implemented; native gate open.** Automatic transform synchronization is disabled and v3 performs a bounded manual f32 projection. | `src/runtime.rs`; Rust compile/runtime gate remains. |
| PHY-005 | Finite values can exceed solver-safe range | **Fixed in both backends; local Python pass and native gate.** Coordinate, velocity, radius, mass, agility, and rotation envelopes are enforced. | Backend/runtime validators and extreme-numeric tests. |
| PHY-006 | Live entity count is unbounded | **Fixed in both backends; local Python pass and native gate.** Adds, restore, and replacement enforce the live-ball cap. | `MAX_LIVE_BALLS` / native equivalent. |
| PHY-007 | Proximity work is unbounded quadratic | **Fixed; local pass and native gate.** Deterministic per-tick work and event budgets bound processing. | Both backends; hardening tests. |
| PHY-008 | Proximity transitions are silently lost | **Fixed in source; native gate open.** Overflow emits an explicit event and preserves state needed for recovery instead of silently committing a partial transition set. | Backend proximity processors. |
| PHY-009 | Cloak sensor survives mode changes | **Fixed; local pass and native gate.** Cloak and sensor state are independent; replacement/uncloak clears stale state. | v3 cloak/sensor regression. |
| PHY-010 | Cloak policy differs by query family | **Fixed; local pass and native gate.** Supported spatial queries share the same visibility predicate. | v3 query regression and backend handlers. |
| PHY-011 | Cone/sphere intersection differs | **Fixed; local pass; native differential gate open.** The ported sphere/cone policy handles wide-cone counterexamples. | `tests/test_v3_regressions.py`, optional backend differential test. |
| PHY-012 | Degenerate triangle policy diverges | **Fixed; local pass and native gate.** Both backends reject degenerate triangles consistently. | v3 geometry regression. |
| PHY-013 | `speedFraction` has no behavior | **Accepted and labelled state-only.** It is stored, replicated, and snapshotted, but no unsupported steering behavior is invented. | Status ledger and `COMPATIBILITY.md`. |
| PHY-014 | “Visual-only” impulse changes authority | **Fixed in source; native gate open.** The call mutates a nonreplicated presentation component only. | `src/runtime.rs`, implementation annotations. |
| PHY-015 | Proximity “sensors” are not Avian sensors | **Accepted and disclosed boundary.** They are explicitly a bounded deterministic compatibility scheduler. | `README.md`, `COMPATIBILITY.md`. |

## Python API, backend, and ABI — 14 findings

| ID | V2 finding | V3 disposition | Evidence / remaining gate |
|---|---|---|---|
| API-001 | Driver failure is silent | **Fixed; local pass.** Background exceptions become sticky park errors surfaced by later calls and close. | Driver implementation and v3 regression. |
| API-002 | Timed-out stop permits two drivers | **Fixed; local pass.** A timed-out generation remains registered and blocks replacement until it actually exits. | Generation-specific driver events and regression. |
| API-003 | Stream input is unbounded | **Fixed; local pass.** Incremental bounded readers reject oversized/non-byte chunks before accumulation. | Facade/codec stream helpers and hardening tests. |
| API-004 | Response limits apply after construction | **Fixed in source; native gate open.** JSON/snapshot writers and FFI responses enforce caps while constructing/copying; every Python snapshot response path validates encoded and decoded bounds before a caller stream write. | `src/ffi.rs`, `src/runtime.rs`, `python/destiny/_facade.py`, regression suite. |
| API-005 | `isCloaked=0` differs by backend | **Fixed; local pass and native differential gate.** Exact integer/boolean conversion policy is shared. | Backend tests and differential scenario. |
| API-006 | Velocity/orientation semantics diverge | **Fixed; local pass and native differential gate.** Calls use one validated vector/quaternion convention. | Backend handlers and differential test. |
| API-007 | `SetBallFree(false)` differs for static balls | **Fixed; local pass and native differential gate.** Idempotent static behavior is aligned. | Backend handlers and contract tests. |
| API-008 | `GetRotatedVector` can be nonfinite | **Fixed; local pass.** Inputs and calculated outputs are solver-safe validated. | Extreme-numeric regression. |
| API-009 | Boolean integer width differs | **Fixed; local pass and native gate.** ABI booleans use validated signed 64-bit representation. | Python/native converters. |
| API-010 | Registry falsely claims park `GetBoxCenter` | **Fixed; local pass.** Both module and park entry points exist, match, and are the two conformant rows. | Reachability generator and v3 regression. |
| API-011 | C++ distance cannot represent null | **Fixed at compile-contract level; native link gate open.** The typed wrapper returns `std::optional<double>`. | Header compile test and linked C++ gate. |
| API-012 | Typed C++ facade covers a small subset | **Accepted and documented.** The generic C ABI covers all dispatchable titles; the typed C++ layer intentionally remains a convenience subset. | Headers and `COMPATIBILITY.md`. |
| API-013 | Native discovery trusts ancestor paths | **Fixed; local pass.** Discovery accepts only an explicit environment path or wheel/package-adjacent library. | `python/destiny/_backend.py`; wheel gate remains. |
| API-014 | Bubble membership is an N+1 ABI path | **Fixed; local pass and native gate.** One lock-consistent backend call returns all membership classes. | Python/Rust membership handlers and v3 regression. |

## Build, packaging, dependencies, and tests — 14 findings

| ID | V2 finding | V3 disposition | Evidence / remaining gate |
|---|---|---|---|
| BLD-001 | Exact final Rust tree was never compiled | **Open release gate.** V3 pins Rust 1.95 and provides fmt/check/test/clippy commands, but none ran here. | `rust-toolchain.toml`, CI workflow, `RELEASE_GATES.md`. |
| BLD-002 | No real Carbon/Lightyear deployment test | **Open deployment gate.** A precise two-client acceptance matrix is supplied; no claim of execution is made. | `RELEASE_GATES.md`. |
| BLD-003 | Native collision behavior is untested | **Tests authored; execution gate open.** Native unit coverage targets cross-bubble filtering, damping/contact behavior, and rollback. | `src/runtime.rs`; Cargo gate required. |
| BLD-004 | Install/wheel/ABI behavior is unverified | **CI configured; execution gate open.** Matrix jobs build, install, relocate, and clean-import. | `.github/workflows/ci.yml`, `RELEASE_GATES.md`. |
| BLD-005 | Token tests accept dead integration | **Improved, not used as native proof.** Semantic Rust tests and an optional differential test were added; source-token checks remain only supplementary. | Native tests plus `tests/test_backend_differential.py`; native run required. |
| BLD-006 | C/C++ tests do not link or execute | **Linked tests authored; execution gate open.** Local strict header compilation passes, while CMake/ABI execution awaits a native build. | `tests/native`, CMake `ctest` definitions. |
| BLD-007 | Legacy CMake config has wrong prefix depth | **Fixed in source; relocated-install gate open.** The legacy config is generated/installed at its correct package location. | CMake config and consumer project. |
| BLD-008 | CMake tracking is incomplete | **Fixed in source.** Recursive Rust inputs use `CONFIGURE_DEPENDS` and feed the native custom command dependencies. | `CMakeLists.txt`. |
| BLD-009 | No adversarial native/differential suite | **Partially closed.** A native-vs-memory scenario and adversarial regressions exist; native execution, fuzzing, and Miri remain open hardening work. | `tests/test_backend_differential.py`, release gates. |
| BLD-010 | Dependency advisories unaudited | **Open supply-chain gate.** CI runs `cargo audit` and `cargo deny`; neither was available locally. | `deny.toml`, CI workflow, `RELEASE_GATES.md`. |
| BLD-011 | Dependency surface is large/duplicated | **Accepted risk with controls.** Direct versions are exact and were lockfile-checked; `cargo deny` duplicate warnings require release review. | `Cargo.toml`, `Cargo.lock`, `deny.toml`. |
| BLD-012 | No CI release pipeline | **Fixed in source.** Linux/macOS/Windows jobs cover Rust quality, linked ABI, install/consumer, wheel, and supply-chain checks. | `.github/workflows/ci.yml`; only actual runs count as evidence. |
| BLD-013 | Binary notices unresolved | **Partially closed; legal gate open.** Direct-license notices are supplied, but the complete transitive inventory must be generated and reviewed for each binary. | `THIRD_PARTY_NOTICES.md`, release gate 4. |
| BLD-014 | Status generation is hard-coded | **Fixed; local pass.** Static Python reachability and Rust dispatch extraction regenerate all 389 rows; semantic differences live in a separate annotation file. | Generator, annotations, JSON ledger, reproducibility test. |

## Overall disposition

All 67 findings are accounted for. The three critical defects have concrete v3
source repairs and local regressions where their Python paths can execute.
Several native, packaging, supply-chain, and real-network claims deliberately
remain open gates. V3 is therefore an integration candidate, not evidence of a
production-qualified native release until every gate in `RELEASE_GATES.md`
passes against the exact archive.
