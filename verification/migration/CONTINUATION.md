# Destiny-rs migration continuation

Status: **INCOMPLETE**. The standalone world-engine Goal remains **active**
with no explicit token budget. This turn is **PROGRESS**: a generated, typed
source collision dispatcher now executes over actual runtime metadata and real
gradient output. Independent original C++ reentrant traces pass. Source
inspection also identified the required old/new pose state that must precede
automatic per-body missile publication. No new agent/chat/daemon/automation;
no live process handles remain at this checkpoint.

## Current checkpoint: source collision dispatcher

The Goal contract was fully reread:
`/home/astral/.codex/attachments/9d617859-4f8f-4cc4-8be6-cf6862cf40ef/pasted-text-1.txt`.
Complete all world mechanics and required assurance before Original Blue /
Blueexposure / exefile / legacy packaging or transport. World semantics behind
those boundaries and mandatory core proofs remain core work.

Source authority remains `114e89fa584f0e1a81eb0a679aabf41af8870072`.
Current upstream and fork main were rechecked and match. Remote Rust branch
and visible `destiny-rs-dev` still match HEAD
`8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268`. All **531** older identities plus
the newer partition regression remain: **532 definitions, 458 Python / 74 C++**.
The source choice is resolved; do not reopen it due to old controller metadata.
All migration work is local, uncommitted and unpublished.

Production implementation through existing typed IR/Genco/syn generation:

- New `src/generated_contacts.rs` declares CollisionEvent, CollisionNotice and
  operation-owned CollisionPass, exported as DestinyCollision* types. Executable
  rules live in contact_rules.json and generated Runtime methods, reading the
  existing source_contacts / source_forces metadata owner.
- `begin_source_collision_pass` captures count and WorldId/Entity/generation.
  `next_source_collision` rereads the actual next ID and all LastC bits, then
  yields one typed event. Appends beyond captured count are ignored; nested
  passes capture independent sizes and observe live mutations.
- `dispatch_source_collisions` uses that same incremental cursor for a direct
  synchronous Rust host. False contact result emits immediate HostError, then
  continues. Event/error hooks may mutate the world or reenter the same
  dispatcher; no metadata borrow crosses the callback.
- Foreign-world cursor use is rejected, replaced active owners cancel, and
  an out-of-bounds read after list shrink becomes an explicit Rust error after
  prior effects. These active-object API guards do not close defined retained /
  detached Original-owner dispatch or original lifetime correspondence.
- This is the dispatcher primitive for later Evolve/FFI scheduling. Automatic
  moved-missile dispatch is **not wired yet**. No whole-world deferred event
  batch, second dispatcher, new queue or alternate collision engine was added.

Independent evidence and current validation:

- Existing stop checker now also extracts unchanged Ball.cpp:468-488 with real
  Vector3d and dynamic vector fields, plus an explicit normally returning host
  event/error hook. Exact channel/method/Lddd descriptor is checked. Callback
  exceptions invalidate the corpus. No Blue layout/private unwinding claim.
- Six C++ cases produce **319 contacts / 3 host errors**: empty, ordered duplicate
  and full signed IDs, append/replace/force/error mutation, nonfinite bits,
  300 contacts, and same-owner recursive dispatch. Rust compares exact traces.
- Eleven direct contact tests pass, including real 130-neighbour gradient
  output through the same dispatcher, independent nested cursors, world/owner
  identity and explicit unsafe-list handling. No formal credit.
- **215 passing observations / 117 distinct names** across fifteen selected
  commands: core debug 111; optimized angular 5/oracle 2/motion 5/missile 10/
  partition 11/liveness 4/liveness-owner 2/force 5/contact 11/concrete-kernel 4;
  optional-integration library 45. All three strict clippies pass.
- **Twelve codegen tests** pass, including the new restricted callback type and
  yielding-loop nodes. Rust/Python generation drift and format pass; six
  architecture views / twenty-one decisions pass generation/drift checks.
- `world-dispatch-original-matrix/summary.json`: **291 passes**, **274 with native
  calls / 17 without**, **2 legacy identity failures, 27 errors, 138 retained
  missing-Blue blockers**. No new native regression. Expected nonzero matrix
  exit is retained, not called a passing campaign.
- Core reader accepts all **174 consumed inputs** and actual compiler/command /
  oracle receipts; native reader accepts current source/binary/case bindings.
  Acceptance negative checks pass. Core and full acceptance are **INCOMPLETE**;
  credited specification and production refinement proofs remain zero.

Current canonical writer is update_migration.py; classifications remain
**24 WORLD_CORE / 2 HOST_ADAPTER / 1 MIXED / 505 UNKNOWN**. Eighteen discovered
features are a partial denominator; fifteen have all named runtime witnesses.
The newly discovered state.old-new-kinematics has no implementation or witness.
All 936 source/configuration holes, twelve presets, complete state/effect scope,
mandatory proofs, stable API/runner, history, robustness/performance/platform /
release gates remain required. Blue-only obligations are later integration;
unreviewed world semantics and core proofs are not deferred wholesale.

preservation-world-dispatch.json accounts for all **117** existing tracked
Rust files: **106 byte-identical / the same 11 intentional generated changes**,
none lost. Existing collision arithmetic/assertions remain. Protected .agents
HEAD/status, inspected blue-rs files, original prompt suffix and Goal files
remain preserved. No reset/commit/push; configured reasoning unchanged and
exact named Max control unavailable.

One freshness diagnosis is retained in world-dispatch-attempts.json: matrix
was initially recorded before source_inventory refreshed its generator ID;
queue correctly rejected that summary. Existing runner then reused observations
only after verifying the same binary/cases/Python sources and rebuilt the
summary against the current inventory; final queue readers accept it. Do not
repeat unchanged fixture failures. On a generator-changing next batch, refresh
the canonical inventory **before** writing the new native matrix, then refresh
the queue after actual observations. No unchanged formal/tool campaign ran.
Historical charon binding, CBMC subnormal sqrt, unsupported Verus f64 and whole
Kani module/API closure diagnostics remain required open work, not fresh proof.

## Next exact implementation: mutable source old/new kinematics

Read next_batch.json and its source packet. Original Ballpark.cpp:429-639
computes results into mOldPos/mOldVel, then swaps old/new per body. Command
SetBallPosition/SetBallVelocity (4738-4801) overwrite **both** vector slots.
A callback from an earlier body can therefore replace a later pending result;
immutable PreparedMotion replay would overwrite that valid mutation.

New AddBall (3355-3370) initializes old_pos=new_pos-dt*new_vel and old_vel=new_vel.
Replacement updates new vectors while retaining old vectors. Miniball writes
are separate (3570-3605). Ball.cpp:880-905/1280-1296/1476-1487 also reads the old
vectors for query caches/time-fraction evaluation/acceleration. Resolve their
qualified symbols and full writers/readers; adding unused metadata is not closure.

Generate a distinct bit-preserving old/scratch pose owner beside the existing
current Avian Position/LinearVelocity owner. Separate command writes-both from
behavior/publication writes, commit computed results into mutable scratch before
publication, and prevent Avian from advancing source-handled bodies. Publish one
body at a time by actual source swap and fresh mode/dead/mushroom/moved/owner
gates. Only moved missiles append iterative IDs and owner self collision, then
use the **existing CollisionPass/next_source_collision** for kind-6 callbacks
outside FFI locks/borrows, before following body publication. Validate an
unchanged-source two-body publication/setter oracle and real callback mutations.
Retain dynamic orientation, troll/mushroom interactions, full pending snapshots,
PyOS/lifetime and mandatory proof gaps until implemented; do not introduce a
fixed-plan workaround or second event algorithm.

The following contact-state checkpoint is historical provenance; its counts,
bindings and next-work proposal are superseded by the current section above.

# Destiny-rs migration continuation

Status: **INCOMPLETE**. The standalone world-engine Goal remains **active**,
with no explicit token budget. This Goal turn is **PROGRESS**: generated source
contact ownership and warp-angle admission now reach the real headless engine,
with independent source observations. No agent, new chat, automation or daemon
was created; no live process handles remain.

## Current authoritative checkpoint: contact state

Read the Goal contract before continuing:
`/home/astral/.codex/attachments/9d617859-4f8f-4cc4-8be6-cf6862cf40ef/pasted-text-1.txt`.
It was fully reread and its digest remains in world-goal-resume.json. Complete
all world mechanics before Blue/Blueexposure/exefile/legacy packaging and
transport. Their world semantics and mandatory core proofs remain required.

Both current upstream and fork main still match source authority
`114e89fa584f0e1a81eb0a679aabf41af8870072`; current remote Rust
`destiny-rs-dev` also matches visible HEAD
`8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268`. Read-only remote checks are saved
in latest-original-remote-refs.json. All **531** older qualified identities and
the newer partition regression remain: **532 definitions, 458 Python / 74 C++**.
Do not reopen the resolved authority choice because old controller metadata
still names the legacy catalog. All work is local, uncommitted and unpublished.

Implemented through the existing typed rules/Genco/syn/span pipeline:

- `DestinyBallMetadata.source_contacts` is the persistent ordered source
  mCollisions owner; full logical BallSnapshot captures/restores it. Duplicates
  and every signed 64-bit value survive; absent older fields default empty.
  No second contact map or alternate collision algorithm was introduced.
- Existing live/free force reset is now reset_source_behavior_state and clears
  contacts too. Dead and nonfree bodies retain the previous list. Existing
  noniterative kernels retain their arithmetic and append identities; the
  transport concatenates each pair/shape scratch list into a dynamic Vec.
  A 130-neighbour case proves the fixture is not capped by model storage.
- Iterative resolve projects actual location collider IDs into the existing
  derived Frame resource, using real ball or registered static shape IDs,
  skipping only source -1. It preserves order/duplicates/zero. This ID-only
  projection is not the complete source collision-location/time/history cache.
- update_source_angles now reads source_contacts rather than Avian
  CollidingEntities. Iterative location IDs remain separate before the later
  missile phase. Public typed getters validate actual body existence.
- Both collision kernel APIs gained output transport; the four existing
  concrete fixtures receive a scratch output argument and retain every old
  assertion/helper. No copied collision body, runner or queue was added.

Independent source evidence and current checks:

- A real aligning Warp mode=3 negative baseline changes yaw/pitch when only an
  Avian contact flag differs, despite both source lists being empty. Receipt
  world-contact-negative-baseline-execution.json and its diagnosis remain.
  The corrected production path passes that same witness.
- Existing stop oracle executes unchanged Potential and records each contact
  sequence, 130 sorted candidates and the original angular method at empty and
  nonempty contact states. All nine stored angular fields match in both
  collision settings (36 exact comparisons/profile). The retained force corpus
  still checks 288 physical/force lanes and 36 overlap-force lanes/profile.
- **205 passing observations / 112 distinct names** across fifteen selected
  commands: core debug 106; optimized angular 5/oracle 2/motion 5/missile 10/
  partition 11/liveness 4/liveness-owner 2/force 5/contact 6/concrete-kernel 4;
  optional-integration library 45. All three strict clippies pass.
- **Eleven codegen tests** pass, including structured generic methods/match
  patterns and rejection of injected/empty rules. Rust/Python generation drift,
  formatting and git diff whitespace checks pass. Six generated architecture
  views / twenty decisions pass drift validation.
- Fresh `world-contact-original-matrix/summary.json` retains **291 passes**,
  **274 with native calls / 17 without**, **2 legacy identity failures, 27 errors,
  138 retained missing-Blue blockers**. No new native regression. The command
  correctly returns nonzero for this incomplete matrix.
- Final core/native readers accept actual source/binary/command/environment/
  fixture bindings. Acceptance negative checks pass. Both core and full
  acceptance remain **INCOMPLETE**. Credited spec/production proofs remain zero.

One canonical writer remains update_migration.py. Classification remains
**24 WORLD_CORE / 2 HOST_ADAPTER / 1 MIXED / 505 UNKNOWN**. Sixteen discovered
features are a partial denominator; fourteen have all named runtime witnesses.
Every feature retains open semantics/assurance. All 936 source/configuration
holes, twelve presets, complete field/effect inventory, mandatory proofs,
stable API/runner, history, robustness/performance/platform/release gates remain.
Missing production mapping/witness for spatial.partition-hash and external
state-transfer are still explicit acceptance failures, not removed obligations.

preservation-world-contacts.json accounts for all **117** pre-existing tracked
Rust files: **106 byte-identical / the same 11 intentional generated changes**,
none lost. Protected .agents HEAD/status, inspected blue-rs sources, original
prompt suffix and both Goal files match saved hashes. No reset/commit/push.
Configured reasoning was preserved; exact named Max control remains unavailable.

Durable diagnostics: early warp setups selected source fallback Stop because
of target distance; explicit mode assertion and a distant target established
the real semantic failure. A fixture uses production::resolve alias, now covered
by guarded call-site mapping. Client requested ID -1 is source-remapped, so the
130-contact fixture supplies actual allocated-domain IDs instead. A Rust test
literal was fixed without changing its value. A concurrent architecture check
was rejected by the canonical-writer lock while observe-core was live; it was
rerun only after that same handle completed and then passed. All attempts remain
under runs/ and world-contact-attempts.json. No unchanged formal campaign ran.
Charon source binding, CBMC subnormal sqrt, Verus f64 and whole Kani module/API
closure failures remain historical diagnostics and required open obligations.

## Next exact production work

Read next_batch.json: **moved-missile collision dispatch at the exact per-body
publication boundary**. Original Ballpark.cpp:530-639 updates a sorted free
body's pose/angles, checks delta.LengthSq()<0.1, reinserts, and only then, only
for missiles, appends iterative location IDs and the master's missing/cloaked/
outside-bubble owner self collision before DispatchCollisions. Ball.cpp:468-488
captures list size once but rereads each ID and LastC for every callback.
Appends beyond that captured size are ignored; source host error reports and
continues. Shrinking below captured size is unsafe original behavior, not a
portable equivalence case to invent.

Build its unchanged-source oracle through the existing checker with a typed
host-event observation boundary, then generate per-body publication/dispatch
continuations through current CallCallback::Ball kind 6 and existing FFI drain
outside locks/borrows. Prevent Avian from exposing later planned poses early.
Actual callback queries/mutations must occur before following body publication;
do not queue all events after whole-world publication. Preserve cached dead,
mushroom/troll, owner and ordering effects. Typed headless event access,
dynamic orientation, full pending-state/location/history snapshots and required
proof correspondence remain open until implemented. No Blue installation or
second event/physics engine is a prerequisite.

The following force checkpoint is historical provenance; its counts, source
bindings and next-work proposal are superseded by the current section above.

# Destiny-rs migration continuation

Status: **INCOMPLETE**. The standalone world-engine Goal remains active with no
explicit token budget. This Goal turn is **PROGRESS**: generated persistent
controller/collision force state now reaches actual headless simulation and
logical snapshots. No delegated agent, new chat, automation or daemon exists.

## Current authoritative checkpoint

Read the Goal contract before continuing:
`/home/astral/.codex/attachments/9d617859-4f8f-4cc4-8be6-cf6862cf40ef/pasted-text-1.txt`.
It was reread and its unchanged digest is in world-goal-resume.json. Complete
all world mechanics before Blue/Blueexposure/exefile/legacy packaging or
transport; world meanings behind those boundaries remain core obligations.

Original pin: `114e89fa584f0e1a81eb0a679aabf41af8870072`. Both current upstream
and fork main were rechecked and still match (`latest-original-remote-refs.json`).
All 531 older source-qualified identities plus the new partition regression
remain: **532 definitions, 458 Python / 74 C++**. Visible branch is
`destiny-rs-dev`, HEAD `8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268`.
All migration changes remain local, uncommitted and unpublished.

## Implemented and independently checked this batch

- `src/generated_forces.rs` and force_rules.json define source mLastG/mLastC
  in one existing DestinyBallMetadata owner. Full BallSnapshot fields retain
  all six binary64 bit patterns; old logical snapshots default absent fields
  to zero. No separate force map or bounded proof-model owner was introduced.
- Reset happens only after actual live/free control admission. Dead bodies
  retain prior values. Computed controller force is stored before collision
  processing; the redundant PreparedMotion acceleration copy was removed.
  Both source collision paths consume the actual retained owner, including
  dead/global candidates, and commit collision-force mutations. A dead body
  can receive a new collision force without advancing its pose.
- Existing collision arithmetic is unchanged. Noniterative mutable scratch
  transport now publishes its force output. Its retained concrete fixture
  explicitly performs the source per-tick reset; every old assertion remains.
  Changes are deterministic Genco/typed rules/recorded syn transformations,
  with staged edit guards. No production feature body was hand patched.
- An actual reachable client Goto tick, single-ball InitializeBubbles and
  master transition exposed the old failure: live x=2 versus independent C++
  x=2.8046712046127142. The failed receipt remains in
  world-force-negative-baseline-execution.json and its diagnostic record.
- The existing stop oracle now also compiles unchanged GotoThrust, Potential,
  CollideTwoSpheres, Quadratic and integration/stop functions. It computes
  historical thrust itself. Three four-tick cases cover underflow, nonunit dt
  and oblique 3D direction; erased-force counterfactuals remain recorded.
  Exact source overlap-force fragments check both bodies. This is a typed
  field/candidate boundary, not full Evolve/iterative-history/Blue ABI proof.

Current verification, after final generation and source binding:

- **193 passed observations / 106 distinct names**. Core debug 100; optimized
  angular 5, angular oracle 2, motion 5, missile 10, partition 11, liveness 4,
  liveness owner/cache 2, force 5 and retained concrete collision fixtures 4;
  optional-integration library 45. All three selected strict clippies pass.
- Force corpus: **288 exact physical/force lanes per profile**, plus **36
  overlap-force lanes**. Five actual headless checks also cover dead-to-live
  reset, source ID pair-selection order, both-path snapshot/future replay,
  signed zero/nonfinite payload bits and older logical snapshot defaults.
- **Ten codegen tests** cover the actual slice/receiver/initializer constructs,
  grouping and rejection behavior. Rust generation drift and format pass;
  Python adapter drift also passes. Six architecture
  views / nineteen decisions describe current implementation and open gaps.
- `world-force-original-matrix/summary.json`: **291 assertion passes**, 274 with
  native calls and 17 without; **2 legacy identity failures, 27 errors and 138
  retained missing-Blue blockers**. No new native regression. All 458 Python
  definitions remain; unchanged missing-Blue evidence was explicitly reused.
- Current core/native receipt readers accept actual source/binary/command/
  environment/fixture bindings. Negative acceptance checks and architecture
  drift pass. Both core and full acceptance remain **INCOMPLETE**.

The single canonical writer remains update_migration.py; queue is
`.aegis/destiny/work-queue.json`. Classification remains **24 WORLD_CORE,
2 HOST_ADAPTER, 1 MIXED, 505 UNKNOWN**. Fifteen discovered features are a
partial denominator; thirteen have all named runtime witnesses. Each retains
open semantics/assurance. All 936 source/configuration holes, twelve presets,
complete field/effect inventory, mandatory proofs, stable API/runner, history,
robustness/resource/performance/platform and release gates remain required.
Credited specification/production proof counts are **zero**. Blue-only
obligations are later work; unreviewed imports/fixtures are not deferred core.

Source force persistence is integrated_partial. Complete collision locations,
contact lists, last-collision state, dynamic orientation and intermediate
callback/error/pending-work histories remain open. Whole Kani imports still
need generated Tracking dependency closure and old initialize/cells fixture
API adaptation. Existing charon binding rejection, CBMC subnormal sqrt and
Verus f64 limitations remain; no unchanged verifier attempt was repeated.

Durable diagnostics: C++ fragment extraction initially omitted its final nested
brace and a scaffold timeOfImpact declaration; source-marker bounds corrected
that without changing original algorithms. AST generation exposed a broad
field selector, syn receiver type inconsistency and a missing trailing comma;
selectors/structured cloning were corrected, with dedicated generator tests.
Strict lint required helpers before the test module. The first overlap test
used the wrong source ID order: source GetNextCollisionFree requires live ID
before dead neighbour ID; the source-valid setup was corrected, keeping all
numeric assertions. All failed/successful attempts remain under runs/. No new
optional analyzer install, proof weakening or second runtime/queue occurred.
Configured reasoning was preserved; exact named Max control is unavailable.

Preservation: `preservation-world-forces.json` verifies all **117** pre-existing
tracked Rust files: **106 byte-identical / 11 intentional generated changes**,
none lost. Protected .agents HEAD/status, inspected blue-rs sources, initial
prompt suffix and both Goal files match their saved hashes. No reset, commit
or push occurred. No live process handles remain at this checkpoint.

## Next exact production work

Use next_batch.json: source-owned collision contact identities, warp-angle
admission and ordered DoCollision payloads. Ball::mCollisions selects aligning
warp direction and emits collider ID plus LastC. Runtime update_source_angles
currently reads Avian CollidingEntities instead; source noniterative and
iterative lists are populated at different phases. Inspect Ball.cpp:335-490,
Ballpark.cpp:600-645 / 2990-3006 and the existing event/notification adapters.
Generate exact reset/retention/order and typed payloads through the existing
owner/continuation path, reusing kernels and real ID/mini registries. Validate
independent source traces and actual headless callbacks/angles across both
modes and master/client/dead/live states. Do not invent surrogate IDs,
deduplicate source events, postpone world semantics as hosting, or handwrite
production feature bodies. Broader state/history/proof/quality gates stay open.

The following historical checkpoint is retained for provenance. Its old counts,
next-work proposal and bindings are superseded by the current section above.

# Previous liveness checkpoint (historical source bindings)

Status: **INCOMPLETE**. Continue the active standalone-world-engine Goal.
This turn is **PROGRESS**: generated source master liveness now reaches the
actual headless Bevy schedule and has independent membership plus runtime
checks. No delegated agent, automation, daemon or separate chat was created.
The Goal has no explicit token budget and remains active.

## Previous liveness details

Goal contract:
`/home/astral/.codex/attachments/9d617859-4f8f-4cc4-8be6-cf6862cf40ef/pasted-text-1.txt`.
The contract was reread; its digest is retained in `world-goal-resume.json`.
Complete the standalone engine before Blue/Blueexposure/exefile, legacy wheels
or authenticated transport; their world meanings remain required core work.

Original authority remains `114e89fa584f0e1a81eb0a679aabf41af8870072`.
Both upstream and fork main were checked again and still match this pin; see
`latest-original-remote-refs.json`. All 531 older qualified tests plus the new
partition regression remain: **532 definitions, 458 Python and 74 C++**.
No old identity was removed. Do not reopen the resolved source choice because
immutable controller metadata still names the older catalog revision.
Visible checkout: `destiny-rs-dev`, HEAD
`8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268`.
All migration work remains local, uncommitted and unpublished.

Completed this batch through the existing Genco/syn/typed-rule pipeline:

- Existing `bubbles::Tracking` now owns keep-alive registration and group
  membership separately, alongside its retained interactivity state. Typed
  source rules preserve raw registration without automatic membership,
  add-before-remove transitions (including equal IDs), empty/phantom groups,
  sentinel guards, large IDs and ClearBubbles/initialization behavior.
- Same-value SetBallInteractive no longer repairs a cleared membership set:
  source membership changes only when its flag changes. A preserved Original
  setter assertion is now classified WORLD_CORE and passes with native calls.
- Source InDeadBubble is generated from typed rules: clients and warp mode
  remain live; master liveness reads actual membership counts. A derived,
  Entity-keyed Frame caches the decision before each free body control phase.
  Cached dead bodies skip control, collision response/publication, integration,
  angular updates, mushroom effects and partition reinsertion. The existing
  Avian schedule temporarily receives Static components for those entities;
  actual source free state, velocities and poses are retained. Components are
  restored before the runtime proceeds; no second motion algorithm was added.
- Proximity samples liveness again. A dead owner with prior regular sensor
  members gets the source forced final check and membership clear. A dead
  owner with an empty set skips that check. The retained timer/candidate/event
  implementation is reused. Full reentrancy and malformed host semantics are
  still open.
- Full logical source_partition_v1 snapshots retain all membership state.
  Mid-evolution pending work/history/cache correspondence is not established.
  Full collision-force history across dead/global candidates is the next
  verified production gap; this feature remains integrated_partial.

Current validation (all receipts refreshed after final fixture generation):

- `world-core-observations.json`: **183 passing observations / 101 distinct
  names**. Core debug 95; optimized angular 5, angular oracle 2, motion 5,
  missile 10, partition 11, liveness integration 4, liveness owner/cache 2, retained concrete collision fixtures 4;
  optional-integration library 45. Core, optional-integration and codegen
  strict clippies pass. The current source/binary/case readers accept the
  bindings. These observations are not formal proof or full-engine acceptance.
- `liveness_oracle_samples.json`: **70 unchanged C++/real-CPython membership
  states and 3,640 all-mode/bubble query pairs**. Both liveness and count are
  compared at each pair, alongside registry/set state. The oracle reuses the
  existing source extractor and process supervisor. Real Python integer,
  dict/set APIs run with the GIL retained; interpreter, library and header
  identities are recorded. Field/scope-exit declarations are explicit scaffold
  adapters; full AddToBubble/Evolve, Blue ABI, invalid/aliased host objects,
  supported platforms and formal correspondence remain open.
- Two existing concrete collision contract modules receive only the explicit
  source-client cached_dead=false input and provenance comments. Every original
  assertion is retained; all four concrete tests run in the existing engine
  library against its actual collision kernels, in debug and optimized builds.
  This supplies ordinary runtime evidence only. The whole Kani crate still
  fails ordinary compilation on generated Tracking module/dependency closure
  and older initialize/cells fixture APIs; see
  `world-liveness-kani-compile-diagnosis.json`. Repair remains required and no
  fake proof-only module or weakened assertion was substituted.
- Four actual headless liveness tests cover dead master pose/velocity/angular/
  partition preservation versus moving clients in both collision settings;
  warp/same-flag/registration effects; snapshots with 100 registered IDs and
  over 64 groups plus future replay; and the dead sensor final pass/re-entry.
  A separate actual runtime continuation test verifies that membership changes
  after control preparation do not recompute that tick's cached motion gate.
- Existing 82 fixed partition transitions and 80 client phase observations
  (48 nonunit-dt ticks / 456 physical numeric lanes per profile) were rerun
  through the same unchanged C++ checker. All previous angular, formation,
  stop, warp, missile, partition, collision-label and client proximity checks
  remain passing in the complete selected core regression.
- `world-liveness-final-original-matrix/summary.json`: **291 assertion passes**,
  274 with native calls and 17 without; **two legacy identity failures, 27
  errors, 138 retained missing-Blue blockers**. No new native regression.
  All 458 Python definitions remain. Unchanged missing-Blue execution evidence
  was reused explicitly; it does not defer unreviewed world semantics.
- Eight generator tests, Rust/Python generation drift, formatting, architecture
  drift and negative acceptance checks pass. Six architecture views/eighteen
  decisions describe actual implementation and its open boundaries.

The single canonical queue remains `.aegis/destiny/work-queue.json`.
Classification: **24 WORLD_CORE, 2 HOST_ADAPTER, 1 MIXED, 505 UNKNOWN**.
Fourteen discovered features are a partial denominator; twelve have all named
runtime witnesses, while every feature retains open semantics/assurance.
Three adapter obligations remain deferred. All 936 source/configuration holes
and twelve Original presets remain required. **Core and full acceptance are
INCOMPLETE; credited specification and production proof counts are zero.**
Complete inventory/fields/effects/configurations, typed engine API/runner,
mandatory proofs, pending history/state transfer, interactions, long-run,
robustness, resource/performance/platform and release gates remain open.

Diagnostics and durable attempts:

- Generation first rejected a new evidence schema mismatch, absent collision
  test literal and an incorrect cleanup-site count. It then exposed missing
  rooted-path support and missing expression grouping in the AST receiver
  insertion. Each was diagnosed, corrected in the permitted input/machinery,
  and regenerated through all-staging edit guards. No live output was hand
  patched or partial failed generation published.
- Strict lint rejected an unused generator import and two nested identity
  guards; these were corrected without suppressions. The snapshot fixture
  first used a nonexistent RestoreSnapshot title and now uses the existing
  Deserialize endpoint with unchanged replay assertions.
- The retained collision-filter test assigned labels directly without source
  live membership. Its setup now supplies membership through the typed API;
  every old isolation/contact assertion remains. All failures and corrected
  attempts remain under runs/. No unchanged formal retry or optional analyzer
  installation occurred. Exact named Max control is unavailable; configured
  reasoning was preserved.
- The existing Kani reader still rejects uninitialized nested charon. The
  recorded CBMC subnormal sqrt modeling counterexample and Verus bridge's
  unsupported f64 lowering remain unresolved. No numeric domain, tolerance,
  assumptions or mandatory proof obligation was weakened.

Preservation: `preservation-world-liveness.json` verifies all **117** initial
tracked Rust files are present: **106 byte-identical / eleven intentional generated
changes**. Protected `.agents` HEAD/status, inspected blue-rs sources, original
prompt suffix and both Goal files match their saved hashes. No source branch
was reset, no new production state owner or queue was introduced, and no push
or publication occurred. No process is live at this final checkpoint. No work is presumed running.

## Previous next-work selection (force portion now implemented)

Read `next_batch.json`: persistent source controller/collision force history.
The pinned EvolveBehaviorForBall resets mLastG/mLastC only for an evolved live
body, then stores thrust. Dead/global candidates retain prior force state;
response/prediction reads it and can mutate both bodies' collision force.
The actual runtime currently projects this tick's prepared acceleration or
ZERO and always zero collision_acceleration, losing that retained state.
Inspect pinned initialization/reset/read/write regions, then generate one
persistent authoritative owner and full logical snapshot mappings. Reuse the
existing collision algorithms, typed rules, span editor and source oracle;
validate an unchanged C++ two-body stale-force sequence and real future ticks
in both response modes. Do not handwrite feature bodies or import bounded
projection storage. Keep all previous assertions and open full-engine gates.

The records below retain historical work and diagnostics. Their old counts and
bindings are superseded by the current checkpoint above and actual readers.

## Authority and preserved work

- Original authority: upstream `carbonengine/destiny` and fork `QuasarRay/destiny`
  `main`, both checked at `114e89fa584f0e1a81eb0a679aabf41af8870072` on
  16 October 2026. The user explicitly selected newest Destiny while preserving
  all Rust work. Old catalog source: `ef738406b61ad06bae2f402857e464735ab106a2`.
- Visible branch: `destiny-rs-dev`, HEAD
  `8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268`. It was fast-forwarded from
  `f5bb8edafb27d35165ceafeb39be8b0498e553e9`, preserving the hashes of all
  117 pre-existing tracked Rust files. Existing branch verification work was
  inspected and retained. No branch was reset or deleted.
- Initial user work: modified `.agents` gitlink, untracked `blue-rs/`, and
  `DESTINY_RS_COMPLETE_IMPLEMENTATION_PROMPT.md`. Preserve all of it.
  `.agents` actual checkout HEAD was
  `db5e63489e9ecad188cd5d3cc7af0025b55b66b7`; deployed governance was not edited.
- Local migration changes are uncommitted and unpublished. Preservation and
  branch audit receipts are in `.aegis/destiny/migration/preservation-*.json`
  and `rust-branch-reuse-audit.json`.

## Historical implementation records (superseded counts and bindings)

1. `tools/migrate_destiny.py` and locked `verification/codegen/` use actual
   LibTooling scalar IR, Genco emission and `syn` transformations. The three
   distance-query call sites now execute the existing unchanged Rust
   `Vec3::length`/`norm_squared` expression, preserving original binary64 order.
   Production adapter: `src/generated_geometry.rs`; caller changes:
   `src/runtime.rs`; registration: `src/lib.rs`.
2. The recorded Python AST pass fixes the `GetBoxCenter` tuple representation
   and Boolean-to-integer input for `isCloaked` in `python/destiny/_facade.py`.
   Its authoritative rules and both immutable/output digests are tracked.
3. The recorded Rust state-guard transformation extracts the unchanged
   `OriginalParkProjection::uncloak_ball` condition, adapts mode/stamp access,
   and wraps the existing native massive-state and sensor-membership mutations
   together. It fixes proper-warp uncloaking without copying bounded projection
   storage or a second state machine into production. Source regions and
   unproved boundaries: `uncloak_source_evidence.json`.
4. The source-distribution configuration includes `verification/spec`, anchors
   forced includes at the root, and excludes local caches, controller/reference
   checkouts and the private user prompt. The existing artifact manifest tool
   can hash/check an actual source archive with `--archive` and `--check`.
   CMake tracks the specification path dependency's sources/manifests so their
   changes also invalidate the native build command.

No new or modified production function body was handwritten. Existing kernels
were reused unchanged; changed bodies are reproduced by recorded syntax passes.
The direct Kani harness is generated in
`verification/kani/src/generated_geometry_contracts.rs`, with registration in
the existing Kani crate. Independent expected values are tracked original-header
oracle samples, not Rust-generated expected outputs.

## Historical reproduction and evidence

```sh
python tools/migrate_destiny.py inventory
python tools/migrate_destiny.py extract
python tools/migrate_destiny.py generate
python tools/migrate_destiny.py generate --check
python verification/tools/transform_python_adapters.py --check
python verification/tools/update_migration.py architecture --check
python verification/tools/update_migration.py self-test
python verification/tools/update_migration.py acceptance
```

The last command must currently exit unsuccessfully with `INCOMPLETE`.
`update_migration.py` is the sole migration writer of the canonical
`.aegis/destiny/work-queue.json`. Do not create another queue. The original
531-case Aegis metadata remains historical and cannot replace latest scope.

Historical observations after the proper-warp correction (superseded by the current receipts above):

- Locked/offline Rust formatting, all-feature/all-target checking, all-feature
  tests and strict clippy passed: **58 tests**, zero ignored or failed.
- Codegen formatting, tests and strict clippy passed: **3 tests**, including
  missing/ambiguous state-guard rejection. Clean Rust/Python drift checks passed.
  Queue rejection checks also cover omitted cases, stale original authority
  and unknown architecture components. Current fixture receipts bind the exact
  case inventory, runner versions and observed Rust/Python/build input sets.
- Ordinary Python tests passed against real debug and optimized native builds:
  **111 tests** each, zero skips.
- Correctly constructed independent original-header comparison passed
  **2,104 exact f64-bit comparisons** in each build, using 263 vectors, two
  radii and four API paths. Reports: `geometry-current-debug.json` and
  `geometry-current-release.json` under the migration state directory.
- At that earlier checkpoint, the complete Python selection: **458 definitions**, **280 assertion
  passes**, **7 failures**, **33 errors**, **138 missing-Blue blockers**.
  **263 passes execute native calls**; 17 assertion passes do not and receive
  no native-integration credit. Latest pointer:
  `.aegis/destiny/migration/latest-original-native-matrix.json`.
- Linux debug and optimized linked C11/C++17 smoke tests passed (2 per build)
  and relocated CMake consumer checks passed (1 per build). Local wheel
  installation outside the checkout discovered its own adjacent library;
  29 existing differential tests passed without skips. These are observations,
  not the full required release matrix. Packaging commands, exact archives and
  current results are retained in `package-*.json`/logs in migration state.
- Doxygen: 364 XML documents schema-validated, 2,107 documentation records,
  zero unresolved file joins, **78 ambiguous identity joins** still open.
  No complete compiler inventory or conformant CPG is established. Six
  architecture views and eight decisions are generated from `architecture.json`.

The first geometry negative receipt used the wrong ego-radius argument order.
Only its center-distance half is a valid negative observation: **452/1,052
center comparisons differed** before the correction. Its surface-distance
half and old total must not be credited. An initial matrix runner also had
basename collisions in ticker log paths; it is retained only as diagnosed
blocked evidence. Current matrix logs have source-hash names.

## Formal blockers and durable attempts

The source inventory retains all **531 old cases plus the new partition case**:
**532 definitions**, 458 Python and 74 C++. Twelve original Windows/macOS
presets yield 6,384 case/configuration pairs. Full behavioral-feature scope is
unresolved. All 78 C/C++ file extraction holes remain explicit across the
12 presets (936 source/configuration holes). No original C++ case or full
original platform configuration has been accepted here.

- The full original CMake configure fails because `carbon-blueConfig.cmake`
  or `carbon-blue-config.cmake` is unavailable. Required pinned dependencies
  include Blue >=6.2, Core >=3.1, Math >=1.2.1 and exefile interpreter >=1.1.1.
  Supply the packages/registry access and their intended original target ABI;
  a mock `blue` module must not substitute for the real boundary.
- The existing immutable Aegis formal source snapshot reader blocks on
  uninitialized nested Kani gitlinks (first `charon`). Exact top-level pins
  were materialized: Kani `0d2328a93f0e0ff66132d6bfa1a7d884877cf862`,
  Verus `aef82eda71838deef4a8cd0260fda6c9504be9b9`. Do not remove gitlink
  directories to bypass source binding or launch unrelated autoharness work.
- One direct supervised Kani 0.68.0 run of
  `generated_geometry_contracts::original_geometry_oracle_samples` failed
  **1/27 checks**, with its **1/1 cover satisfied** (31.139 seconds, 120-second
  timeout). It was not classified by Aegis and grants no proof credit. The
  numeric assertion body is unchanged; later generator-provenance comments
  changed, so the retained run is diagnostic historical evidence.
- A separately generated C/CBMC 6.11.0 check reproduces the `1e-160` literal
  discrepancy: native C exit 0; CBMC exit 10, **1/39 failures**, unwind 128
  with unwinding assertions. The builtin `sqrt` model admits nondeterministic
  roots satisfying rounded multiplication, which permits different bits for
  the subnormal intermediate. Preserve `subnormal-sqrt-diagnostic.*`; do not
  alter the comparison, input domain or stubs to hide this counterexample.
  No unchanged proof retry was run. Existing Verus scalar lowering excludes f64.
- Pinned working Verus executable:
  `.aegis/destiny/tools/verus/verus-x86-linux/verus`, version
  `0.2026.09.20.aef82ed`; `/opt/verus/verus` is an older version.
- LibTooling setup had two diagnosed fixes: LLVM23 USR header movement and
  explicit compiler resource directory. Doxygen 1.18.0 and CMake 4.2.1 official
  release archives were hash-verified. clang-doc23 supports JSON, not YAML.
  Tool/provenance receipts are in `reuse.json` and `.aegis/destiny/tools/`.
- PyPI provisioning stalled on `files.pythonhosted.org`; it was stopped.
  scikit-build-core's Git clone stalled before object transfer and was stopped.
  One immutable official source archive fallback succeeded at
  `24bd675a4a677e75324b61a358eee19c098f4c58` (v0.11.6), with generated tag-based
  version metadata only. Pathspec v0.12.1 is pinned at
  `6485791e1b5cf2ef4e756ae392fa80f2c5045d4c`. Do not repeat unchanged failed
  network setup. This does not add analysis dependencies to ordinary builds.

Current credited proof counts are zero in the refreshed newest-source queue.
This means current proof evidence is not credited; it does not erase historical
proof work. Runtime tests, annotations, compiler success and documentation never
substitute for missing specification/production-refinement evidence.

## Historical next-batch proposal (superseded by next_batch.json)

Use the current source-bound `next_batch.json` and the Next exact work section
above. The angular, formation, stop, warp-alignment and missile batches are
implemented scoped progress; do not restart them from older continuation text.
