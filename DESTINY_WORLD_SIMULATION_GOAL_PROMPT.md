You are the implementation agent for Destiny-rs. Your first priority and first completed product must be a production-ready, headless Rust/Bevy world-state and world-state-mutation simulation engine containing every game mechanic and world-simulation behavior provided by Original Destiny, with maximum behavioral fidelity.

Complete that engine before integrating Original Blue. Blue, Blueexposure, exefile, legacy Python hosting, and their packaging must not become prerequisites for implementing the world simulation. Preserve their integration obligations for the later milestone. This is a sequencing change, not permission to omit a mechanic, simplify a controller, discard a world-state transition, or claim that the entire legacy integration is finished.

This prompt supersedes the earlier all-at-once migration sequence for the current world-engine objective. Preserve its requirements for generated implementation, exact semantics, reproducibility, idiomatic Rust/Bevy, and truthful verification. Use the world-engine acceptance gate below for the first milestone; retain a separate gate for the eventual complete legacy integration.

The executable outcome is a reusable simulation engine that an ordinary Rust host can construct, populate, mutate, step, query, snapshot, restore, and replay without Blue. Its actual Bevy runtime must implement all Original Destiny world logic. A facade, a feature registry, a successful proof model, an in-memory test double, or a small demonstration does not satisfy this goal.

1. **Attach the work to the correct Codex Goal.**

   When the user issues this prompt for implementation, it explicitly authorizes using Codex Goals for this objective. Inspect the available goal tools and call `get_goal` at startup. Reuse an existing matching goal; do not create a duplicate. If there is no unfinished goal, use `create_goal` with the following objective, retaining this file as its detailed acceptance contract:

   > Deliver a production-ready standalone Rust/Bevy world-state and world-state-mutation engine implementing every Original Destiny game mechanic and world-simulation behavior with maximum fidelity. Complete all core behavior, generated implementation, direct runtime integration, idiomatic refactoring, independent validation, required core proofs, reproducible builds, and architecture records. Verify completion against the source-derived world-feature inventory and current production evidence. Postpone Original Blue/Blueexposure/exefile integration and legacy hosting/packaging until the world engine is accepted, while retaining all world semantics currently exposed through those boundaries. Preserve existing work, use bounded metaprogramming, and continue across turns until the world-engine acceptance gate passes or a genuine external blocker prevents all meaningful progress.

   Do not specify `token_budget` unless the user supplies one. If the active goal includes the broader legacy migration, preserve that objective and record world-engine acceptance as its first milestone; do not mark the broader goal complete at that milestone. If an unrelated active goal occupies the thread, do not overwrite it or falsely complete it to free the slot. Surface the conflict and continue any independently authorized work.

   Use the actual goal tool contracts as the lifecycle authority. `update_goal` changes status, not the objective or budget. Call it with `complete` only after the relevant completion audit passes. Use `paused` only on an explicit user request, and stop work when paused. Do not invent automatic resume, clear, budget-change, or completion operations. Respect user/system budget and interruption controls.

   Mark `blocked` only when the same genuine external blocker has recurred for at least three consecutive goal turns and no meaningful independent work remains, following the active tool contract. On a user-resumed blocked goal, start a fresh blocker audit. A missing Blue package, unfinished optional analyzer, or one unresolved proof does not block unrelated world implementation. Do not spin through repeated failures to manufacture three blocked turns.

   At each continuation, inspect the current files and evidence. Classify the previous turn as progress, a verified wait on a live handle, or no progress. A report alone is not progress. An old log, continuation note, or lockfile is not evidence of a live process. Poll the same live handle after an observation timeout rather than restarting work without evidence that it stopped. Keep the goal active while useful work remains and completion is unproved.

   Save compact continuation state and let the Goal control continuation. Do not create reminder automations, self-messaging loops, new chats, or background processes as substitutes. If goal tools are unavailable, preserve a resumable work record and report that limitation without claiming automatic continuation. See the official [Codex Goals guide](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex); runtime tool contracts take precedence over examples.

   This execution request authorizes Goal-managed continuation. Older continuation notes saying that background execution was not authorized do not cancel this newer authorization; their restrictions on independent daemons or unrelated background jobs still apply.

2. **Resume from verified current work and start the next simulation change.**

   Inspect the branch, HEAD, dirty files, source authority, and relevant repository instructions. Preserve all existing Rust, generated code, verification, documentation, `.agents`, and `blue-rs` work. Do not reset the checkout or restart the migration. Read `.agents/INDEX.md` and only the policy/source records needed for the selected obligation. Treat the pasted agent report and continuation record as leads to verify, not proof that a check is current.

   This implementation request authorizes necessary official, pinned engine dependencies and selected verification tools to be fetched/built in project-local locations, subject to actual environment permissions. Inspect setup scripts and preserve provenance. It does not authorize credential changes, system-wide modifications, publication, or starting the deferred Blue integration. Prefer already available dependencies and honor durable failed-attempt records.

   Start with `verification/migration/CONTINUATION.md`, `source_inventory.json`, `next_batch.json`, `reuse.json`, the current work queue, `tools/migrate_destiny.py`, `verification/codegen`, and `verification/tools/update_migration.py`. Reuse the existing generator, syntax transformations, source maps, architecture records, process supervisor, and evidence parsers. Keep `update_migration.py` as the canonical migration queue writer unless a concrete change requires an explicitly documented replacement. Do not create a second queue.

   At prompt preparation, local `main` and the migration inventory identified Original Destiny commit `114e89fa584f0e1a81eb0a679aabf41af8870072`; the older retained catalog used `ef738406b61ad06bae2f402857e464735ab106a2`. The current Rust HEAD was `8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268` with uncommitted migration work. Verify these identities and preserve the recorded authority decision. Do not reopen a resolved revision choice merely because older controller metadata has not caught up. If newer authoritative source is intentionally adopted, retain and reconcile earlier obligations rather than resetting coverage.

   The inspected next batch is source-owned binary64 yaw/pitch/roll and prior angular state, followed by formation thrust and orientation in the real motion tick. Verify it against the current runtime. Inspect `src/runtime.rs::{set_velocity,get_ball_property,original_motion_input,prepare_motion_tick}` and the relevant original `Ball.cpp`/`Ballpark.cpp` regions. Reuse appropriate kernels in `verification/spec/src/original_park.rs`, `original_rotation.rs`, and `original_motion.rs`, without importing their bounded model storage as the production domain.

   If that gap still exists, generate its correction and compare the original velocity/angle assertions and moving/stopped formation traces before widening. If already fixed, select the next verified core gap. Keep baseline inspection bounded; do not rerun completed discovery, package installation, or unrelated proof campaigns before making this simulation change.

3. **Enforce the boundary between world mechanics and later integration.**

   Classify each original obligation in the same canonical inventory as `WORLD_CORE`, `HOST_ADAPTER`, `MIXED`, or `UNKNOWN`, with source-backed rationale. Split `MIXED` records into linked core and adapter obligations without losing the original identity. `UNKNOWN` cannot count as deferred integration or completed core. The file name, import list, or dependency on Blue does not decide semantic ownership.

   | Complete in the world-engine milestone | Defer to the Original Blue integration milestone |
   | --- | --- |
   | All world state, rules, controllers, queries, mutations, tick phases, geometry, collision, visibility, and lifecycle behavior | Blue/Blueexposure registration, object hosting, reflection ABI, and exefile runtime setup |
   | Command meaning, defaults, validation, state effects, callback semantics, ordering, and notification payloads | Python/Blue object conversion, legacy module identity, wrapper registration, and runtime dispatch plumbing |
   | Snapshot meaning, complete state capture, restore, history, prediction/reconciliation, replay, and stream-defined state changes | Original Blue stream/container APIs and exact Blue-private encoding/ABI interoperability |
   | Destiny client/server world rules, relevance, action sequencing, world-update batching, and merge/history logic | Authenticated transport deployment, socket/link integration, legacy wire envelopes, and host credentials |
   | A standalone engine library, direct host API, headless runner, supported engine-platform builds, and engine release artifacts | Legacy Python wheels, Original Blue package discovery, relocated legacy consumers, and integrated Carbon product packaging |

   Some encoding fields or callback details change simulation behavior. Their meaning belongs in the core now even when their Blue representation is deferred. Generate typed input/output records and source-backed boundary mappings. Do not delete world logic merely because it resides in `*_Blue.cpp`, a Python module, a stream reader, or a client/server wrapper.

   Do not spend the first milestone completing Blue, `blue-rs`, Blueexposure, exefile, or the original package registry. Reuse available tooling from them when directly useful and compatible. Only inspect the narrow dependency semantics needed to implement a world feature. Missing host packages may block a particular legacy test environment; they do not justify stopping source analysis, implementing other mechanics, or testing the standalone engine.

   Retain later integration cases and their failures with an explicit `DEFERRED_BLUE_INTEGRATION` status. That is neither a pass nor an exclusion from the overall migration. Define core acceptance and full-migration acceptance separately so the world engine can be accepted honestly while the broader compatibility product remains incomplete.

4. **Inventory every original world mechanic, not only failing tests.**

   Build the core inventory from source, build configurations, original fixtures, public and private entry points, and observed consumer contracts. Retain the existing original test identities as `source::fixture::test`, setup/teardown, helpers, parameterizations, and duplicate names from different fixtures. The current inventory's 532 test definitions are a starting observation, not the number of world features and not an authoritative final denominator.

   Reconcile declarations, bodies, registered operations, enum variants, state fields, serialization readers/writers, controller dispatch, and every branch that mutates or derives world state. Account for supported compile-time variants. Unknown branches, unparsed units, unresolved effects, ambiguous symbol joins, and unreachable-looking mechanics remain explicit until source-backed resolution. A dead wrapper does not prove a mechanic is unused.

   Use the following minimum coverage checklist and expand it from the source:

   | Family | Required world behavior |
   | --- | --- |
   | Identity and lifecycle | World/ballpark creation and teardown, stable IDs, replacement, delayed removal, re-addition, ownership, dependent-object cleanup, spawn/despawn callbacks, and mutation restrictions during evolution |
   | Authoritative state | Position, linear/angular state, original stored angles and previous values, mass/inertia/radius, speeds/fractions, flags, modes, targets, timing, and every other state field that affects current or future behavior |
   | Controllers | Goto, follow, stop, orbit, all warp phases, missile/target tracking, formation, boid, mushroom, troll, field, rigid/static and miniball behavior wherever defined; every transition, threshold, reset, and target-loss case |
   | Motion and orientation | Original thrust, acceleration, damping, speed limits, alignment, angle/quaternion rules, old/new state ordering, prediction/interpolation, and fixed-step integration |
   | Collision and geometry | Iterative/non-iterative response, filtering, masses, overlaps, contact ordering, spherical/compound geometry, mini-spheres/capsules/boxes, oriented boxes, and original query semantics |
   | Spatial organization | Partitions and their hashes, boxes, bubbles, membership, boundary crossing, distance/intersection/neighborhood queries, global/local objects, and original tie-breaking |
   | Visibility and proximity | Cloaking variants, proper-warp interactions, observer relevance, occlusion, owner/global rules, sensor periods, entry/exit transitions, and notification ordering |
   | Time and sequencing | Clock units, start/stop/pause, tick indices, phase order, multiple steps, scheduled actions, and all original random-state dependencies |
   | World mutations and events | Every setter/action, parameter domain and default, errors, mutation order, callbacks, reentrancy rules, batching, and event contents/lifetimes |
   | History and state transfer | Full state and pending work, snapshots, restore, rewind, history merge, update application, deletion/recreation, correction/replay, and future-state equivalence after restoration |
   | Client/server simulation rules | Authoritative action effects, ticker semantics, ordering, relevance/bubble updates, state deltas, history and batching semantics, independent of the eventual transport |

   Include all mechanics that Original Destiny actually contains. Do not invent unrelated combat, inventory, economy, or other whole-game systems merely because Destiny is used by a game. Conversely, do not omit an original mechanic because it is unfamiliar or currently absent from the Rust facade.

5. **Define the engine's complete state-transition contract.**

   Model the source semantics as `(world state, ordered inputs, time step, explicit environment) -> (next world state, results, ordered events)`, with queries and intermediate observations where the original exposes them. The environment includes required clock, random, and external-value inputs. Capture all state that changes later behavior, including queued removals, controller history, pending callbacks, sensor timers, collision state, and prediction/history data.

   For every field and mutation, record original source ownership, type, initialization, writers, readers, phase, invariants, persistence, and disposal. Distinguish authoritative state from caches; verify cache rebuilds and invalidation. Eliminate contradictory shadow copies between a Rust facade and Bevy. Stable Destiny IDs must remain independent of transient ECS entity IDs.

   Specify exact input domains and error behavior. Preserve original transaction boundaries, partial effects, callback timing, and sequencing. Do not introduce batch atomicity, deferred callbacks, or fail-and-rollback semantics where they change the original contract. Keep externally supplied input validation at explicit boundaries without using artificial range/size limits to avoid hard mechanics. Treat undefined original behavior explicitly rather than imitating memory unsafety.

   Expose a typed Rust API for world construction, object creation/removal, every supported mutation, time advancement, queries, event access, capture/restore, and replay. API names are design choices; the complete semantic mapping is mandatory. Provide a thin headless executable or replay runner using that exact production library. Test-only implementations and duplicate runner algorithms are forbidden.

6. **Implement on Bevy without changing Destiny's physical or logical model.**

   Use the pinned, compatible Bevy runtime with components, resources, queries, systems, plugins, and explicit schedule dependencies. Implement source-defined phase ordering and deferred-command boundaries. Keep independent worlds isolated. Record which systems read and write each state field and which transitions must be observable immediately.

   Match original precision, integer widths, signedness, overflow/casts, floating-point operation order, rounding, coordinate conventions, angle storage, thresholds, and clock units. Preserve source-owned yaw/pitch/roll where required; a different quaternion convention is not automatically equivalent. Require exact state/byte/enum/order comparisons and bitwise numeric matching where contractual. Any permitted tolerance must come from the source contract and be fixed before judging results, including long-run drift and branch divergence.

   Earlier compatibility annotations such as "implemented with accepted difference" do not waive this task's fidelity requirement. Retain them as core migration gaps wherever they differ from the authoritative world contract; remove the gap only after the equivalent behavior and its required evidence exist.

   Generate Destiny-specific control and integration systems when stock Bevy/Avian semantics differ. One owner must advance each physical state variable; prevent two integrators from applying motion. Generic rigid-body physics, visually similar movement, or a controller that only changes a mode flag cannot substitute for the original algorithm. Preserve original collision/control ordering even when a different engine ordering is convenient.

   Respect the original determinism contract. Define ordering for mutations, queries, contacts, events, and callbacks where it is observable. Validate parallel execution before enabling it; retaining serial execution is acceptable where ordering matters. Do not invent cross-platform bitwise guarantees beyond the established numeric model, and do not loosen original guarantees to match the implementation.

7. **Keep the production engine independent of Blue.**

   Provide a build target for the complete world engine that requires no Blue, Blueexposure, exefile interpreter, legacy Python runtime, or authenticated transport. Optional integration modules must depend on the core; the core must not depend on their runtime types, callbacks, allocators, or global initialization. Use the existing crate structure if it can enforce this boundary; split a crate only when necessary.

   Verify the actual dependency and symbol paths, not just feature names. The current package defaults to `carbon-network`; establish and test the correct engine-only Cargo feature/target selection rather than assuming default or `--all-features` builds establish independence. Keep existing integrations working where possible without making their completion a core prerequisite. Do not change dependency versions just to match a sibling port.

   Use small typed adapters for time, random state, callbacks, events, and external values. These adapters must expose real semantics and allow deterministic tests. They must not synthesize missing mechanics, return hardcoded expected outcomes, or pretend to be an installed Blue runtime. Production mechanics must execute in Rust/Bevy, not through Original Destiny C++ FFI, a subprocess, or a hidden fallback. Original C++ execution is a validation oracle only.

8. **Preserve metaprogramming as an implementation shortcut.**

   Prohibit manually authored new or changed production feature bodies. Produce them through deterministic generation or recorded syntax-aware transformations. Permitted authored inputs are extractors, reusable lowering rules, structured semantic specifications, mapping tables, macro definitions, generator infrastructure, and build/test orchestration. Do not hide handwritten Rust algorithms inside template strings or arbitrary target-code fields. Retain proven existing kernels unchanged with provenance rather than rewriting them for cosmetic consistency.

   Reuse `tools/migrate_destiny.py`, `verification/codegen`, Genco, `proc_macro2`, `syn`/`quote`, and existing derive/delegation mechanisms. Generate executable state transitions and Bevy wiring, not merely symbols, bindings, or test labels. Prefer ordinary emitted functions when macros become difficult to reason about. Repair the generator or transformation and regenerate; never patch generated output manually.

   Use Clang-doc and Doxygen for machine-readable documentation, LibTooling for compiler-resolved semantics, and the pinned [Joern CPG schema](https://cpg.joern.io/) for structural and dependence mappings. Keep typed semantic IR for executable lowering; a CPG alone is not a translator. Validate graph layers actually produced and keep incomplete layers explicit. Select CodeQL, SVF, PhASAR, CBMC, or KLEE for a named unresolved analysis or validation question rather than installing all of them before the next feature.

   Respect detected tool capabilities: the recorded local clang-doc23 supports JSON, so do not repeat attempts to force YAML without checking the installed version. Reuse inspected Trinity Doxygen settings only as configuration/parser inputs, replacing its source roots and exclusions; regenerate Destiny evidence from Destiny source. A full CPG, universal transpiler, documentation redesign, or verifier automation rewrite must not precede completing real world mechanics unless indispensable for a specific obligation.

   Generate source maps, input/rule/tool hashes, and reproducible commands. Regenerate into staging and compare expected output without destroying user files. Generate tests from preserved original assertions, source-backed properties, and independent oracle inputs. The emitter and its own generated expected values cannot certify one another.

9. **Bound infrastructure work and spend effort on mechanics.**

   Reuse the existing working pipeline immediately. Keep the initial rebaseline and scope-routing change narrow enough to start the next world mutation in the same working session. Do not allocate another greenfield setup phase when generation already works.

   For a new optional analyzer or metaprogramming repair, default to two diagnosed attempts and 30 minutes total. After the first new core batch, keep optional tooling improvements below roughly 15% of engineering time across three batches. These limits apply to optional infrastructure, not necessary feature implementation, source understanding, or mandatory verification. Existing exhausted attempt budgets survive restarts, goal continuations, and compaction.

   When an optional tool stalls, use available compiler records, narrow source-anchored typed rules, simpler Genco emission, or recorded syntax transformations. Do not switch to handwritten production code or weaken a check. Preserve the missing analysis as an explicit gap if relevant. Stop paying for an unchanged failure; a new attempt needs a changed premise and a specific expected outcome.

   Measure accepted core behaviors, resolved dependencies, current failures, elapsed work, and actual token/credit counters when available. Label unavailable counters rather than estimating them as facts. Generated line counts, installed tools, documentation volume, and repeated status reports are not progress. Read bounded source packets, query relevant graph records, cache deterministic extraction with complete input hashes, and run focused checks during development. Reserve the complete engine matrix for meaningful integration points and final acceptance.

10. **Validate mechanics without waiting for the entire Blue application.**

    Establish independent reference evidence at the narrowest faithful boundary. Prefer unchanged original pure functions, headers, and source slices with their real data/math dependencies; compile isolated original kernels where feasible. Use recorded original traces with source/build/fixture provenance. Translate original fixtures into generated standalone Rust scenarios preserving every setup, input, observation, assertion, tolerance, and ordering requirement.

    When an original test imports Blue, identify the assertions about world behavior and port their execution boundary to the actual core. Link the resulting core checks to the original fixture and preserve the host-only assertions for later. Do not turn a missing import into a reason to skip every assertion in the file. Likewise, 138 historical missing-Blue cases must not automatically become 138 deferred features; classify their semantics individually.

    Extraction scaffolding may supply declarations or narrow environmental interfaces needed to compile an original kernel. It may not replace the behavior being measured, invent a layout/value contract, or mock unknown Blue semantics into a passing oracle. Record the slice, external calls, input mapping, changed build environment, and remaining correspondence limits. Source inspection and a translated test are not equivalent to running the complete original executable.

    Do not require a full Blue build for a mechanic whose contract can be independently established and validated at this boundary. Equally, if an unresolved dependency actually determines a core state transition, retain that core obligation as blocked until its semantics and required evidence are available. Continue other mechanics. Maximum fidelity cannot be achieved by relabeling unknown behavior as integration-only.

    Compare intermediate world state, ordered mutations/events, and future evolution after restore, not just final positions or return values. Cover transitions between mechanics, target deletion, collisions during movement, pause/resume, batched actions, rewind/replay, and long sequences. Use fuzz/property tests, adversarial boundaries, and representative semantic mutations in disposable builds to check that missing effects, wrong ordering, or numeric changes are detected. Expected results must not be computed by the implementation under test.

11. **Keep required proofs and assurance truthful without letting one blocker halt unrelated mechanics.**

    Preserve the existing core safety, behavior, Kani, Verus, production-refinement, and non-vacuity obligations. Reuse the established bind/verify/audit workflow and actual production kernels. Separate source/specification review, generated mapping, specification proof, production proof, differential evidence, and release evidence. A model-only proof, passing fixture, current annotation, or compiler success is not whole-engine proof.

    Rebind each result to the current source, generator, tools, configuration, assertions, assumptions, and dependency closure. Check genuine production reachability and required cover witnesses. Do not remove inputs from the binding, weaken assumptions, narrow valid production domains to bounded proof-model storage, or count stale/failed/skipped/empty selections as successful checks.

    The continuation records a subnormal `sqrt`/CBMC modeling discrepancy, an uninitialized Kani `charon` binding issue, and unsupported f64 lowering in the existing Verus path. Verify only the relevant diagnostics; do not repeat unchanged proof runs or rebuild an unrelated verification platform before implementing the next world feature. Fix a tool/model issue through a justified scoped change or supported alternative when possible. Do not alter numeric expectations or stub the difficult behavior to hide it.

    A missing required core proof keeps its assurance gate open while other core implementation advances. Moving a Blue-only obligation to the later milestone is permitted by this sequencing instruction; moving a core proof there is not. Full original-host correspondence may remain a later integration claim when independent core evidence satisfies the core contract. State exactly what has and has not been established; never silently substitute testing for a mandatory proof.

12. **Make the standalone engine production-ready.**

    Deliver a documented, stable core API with explicit ownership, lifecycle, thread/access rules, error behavior, versioned state representation, and a clear event/mutation contract. Build and run from a clean checkout with locked dependencies and without analysis tools or Blue on the consumer path. Keep code generation a maintainer operation, not a paid-model or graph-database dependency at runtime.

    Validate world isolation, deterministic supported execution, stable IDs, replacement/removal, stale references, cache invalidation, cleanup, repeated construction/destruction, and snapshot/restore/replay fidelity. Exercise valid large and dense worlds, worst-case collision/proximity inputs, long-run evolution, and retained history/event queues. Detect leaks, unchecked growth, unsafe aliasing, panics, overflow, and malformed-input failures at the actual production boundary. Preserve original valid domains rather than making an arbitrary cap count as compatibility.

    Establish representative workloads and performance/resource budgets from actual original behavior, repository requirements, and intended hosting conditions before judging results. Measure step cost, memory growth, spikes, and deterministic outcomes in optimized builds; retain seeds and commands. Do not invent measurements or move a failing target. Optimize only after semantic correctness is established, and rerun affected fidelity checks after optimization.

    Define the core platform/configuration matrix from original semantic differences and intended engine targets. Preserve all world-affecting build flags and architecture/numeric differences. An Original Blue product preset is not itself a core implementation dependency: map it to the relevant standalone core checks and defer only its host packaging. Collapsing truly equivalent configurations requires explicit evidence; one host's green run does not validate all targets. Keep unavailable required engine-platform checks incomplete.

    Separate the standalone engine release artifact and its tests from the later Blue/Python/transport release artifacts. Preserve applicable supply-chain, license, provenance, reproducibility, safety, and packaging checks for the engine itself. Do not spend implementation batches rebuilding legacy wheels while core mechanics are missing.

13. **Refactor generated behavior into idiomatic Rust and Bevy.**

    After each batch is behaviorally accepted, improve generation rules, templates, or repeatable AST transformations so the produced code uses clear types, enums, ownership, composition, errors, modules, ECS state, and coherent systems. Preserve source-defined state and external behavior. Avoid giant dispatchers, duplicated states, translated C++ pointer machinery, and macro layers that obscure the actual algorithm.

    Maintain one production implementation of each mechanic. Pure semantic kernels are useful when the actual Bevy systems call them. Internal idiomatic APIs may differ from legacy names; generated mappings and later adapters preserve original contracts. Rerun targeted semantic, integration, and proof checks after changes. Clean regeneration must retain the refactor, and formatting/lints must pass without broad suppressions hiding defects.

14. **Generate architecture records that describe the engine actually delivered.**

    Extend the existing structured architecture metadata and generator. Produce readable views and machine-readable records aligned with [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html), with architecture decisions alongside the descriptions. Include stakeholders and concerns, scope, viewpoints/views, world state ownership, mutation contracts, ECS scheduling, controller/collision composition, time and randomness, snapshot/history/replay, deployment, and evidence/provenance.

    Make the core/host boundary and dependency direction explicit. Record later Blue integration as planned, with input/output contracts and unresolved adapter work; never render it as implemented. Link each core feature and state transition to original source, generated modules, runtime call paths, checks, and decisions. Preserve the existing six views/eight decisions only insofar as they remain accurate, and regenerate as the architecture changes.

    Validate links, record schemas, missing concern coverage, and contradictory views. Describe actual choices and rationale; do not invent historical intent or ISO clause numbers. Architecture records and decision records support maintainability but do not prove behavioral completion or ISO certification.

15. **Extend one acceptance system with separate milestone results.**

    The existing `verification/tools/update_migration.py acceptance` checks the broad migration and currently combines core, original-host, and deployment obligations. Extend that implementation with a clearly documented core scope or mode; this is a requested change, not a claim that such an option already exists. Keep the existing full-migration result truthful. Generate both results from the same canonical source inventory, classified obligations, and actual receipts.

    Add fields equivalent to semantic domain, core obligation IDs, adapter obligation IDs, original fixture/feature IDs, required core configurations, generation provenance, production paths, required evidence, current result, and deferred reason. Use stable IDs and source digests. Classification changes need rationale and an audit trail. Preserve every old obligation; separate counts for original tests, core features, configurations, proof obligations, and host integrations. Mixed fixtures must retain their core assertions even when host checks cannot run.

    Reject missing, ambiguous, stale, unknown, uncovered, or contradictory core records. Reject completion from self-reported flags or hardcoded counts. Exercise acceptance rejection using an omitted core mechanic, a stale source digest, a missing production witness, and a mixed fixture incorrectly deferred wholesale. Verify that an unmet Blue-only check keeps full migration incomplete without incorrectly blocking an otherwise complete core milestone.

    The goal's completion audit must inspect what the acceptance checker really checks, plus the relevant current code, tests, manifests, and evidence. A green report is insufficient if its denominator or conditions omit an original world behavior. Test coverage by itself cannot establish a complete feature inventory.

16. **Apply all world-engine completion gates before declaring success.**

    | Gate | Evidence required |
    | --- | --- |
    | Scope closure | Every original world feature, state field/effect, controller variant, supported semantic configuration, and core assertion is accounted for; unknowns and extraction holes relevant to the core are resolved. |
    | Complete behavior | Every core obligation reaches its real Rust/Bevy implementation, with no stubs, state-only controllers, unsupported valid operations, approximate replacements, or known divergences from the authoritative world contract. |
    | Blue independence | Clean dependency inspection, build, and end-to-end headless execution demonstrate all mechanics without Original Blue, Blueexposure, exefile, legacy Python hosting, or transport initialization. |
    | Fidelity | Independent original-source/fixture/oracle evidence covers state, transitions, queries, timing, errors, event ordering, numeric behavior, interactions, and long-run replay under declared equivalence rules. |
    | Required assurance | Every mandatory core safety/refinement/proof obligation has current valid evidence with assumptions, bounds, production reachability, and trust limits recorded. |
    | Production quality | Relevant format/lint/tests, debug/release checks, robustness, representative performance/resource checks, supported engine-platform checks, API/lifecycle/snapshot validation, and release requirements pass. |
    | Reproducibility | Pinned inputs and recipes regenerate the delivered code and documentation; output drift checks, clean builds, licenses, and artifact integrity checks pass. |
    | Maintainability | Generated output is idiomatic Rust/Bevy; architecture views, decisions, API/run documentation, source mappings, and the deferred Blue integration contract are accurate and reviewable. |

    Only after all gates pass may the result be called a completed production-ready world engine. Report its precise reference revision, supported matrix, evidence, and claim boundaries. This does not mean the full legacy Destiny/Blue product integration is complete. If the active goal is this world-engine objective, complete it after the audit and leave Blue integration as the later milestone. If the active goal still includes that later work, record the core milestone without completing the broader goal.

    Do not start Blue implementation under a world-engine-only goal. A subsequent authorized integration task can consume the stable engine and its recorded contracts after core acceptance. Preserve that work's obligations now so postponement does not become omission.

17. **Continue toward the full core outcome and leave precise resumable state.**

    Work through the dependency-ordered core queue until all gates pass. Each batch should close an actual mechanic or resolve a concrete prerequisite and then verify the resulting production path. The first angular/formation batch, all currently failing tests, or all currently supported API titles are intermediate progress, never the final scope.

    Before each turn ends, persist the source/configuration identities, implemented core obligations, remaining inventory, current failures, deferred host obligations, generator recipes, evidence freshness, used attempt budgets, live process handles if any, and next exact action. Use the existing migration continuation and state locations. A saved checkpoint is not permission to restart discovery or claim invisible background work.

    Report genuine external blockers with their affected obligations and smallest required input, and continue every independent core task. Do not voluntarily stop because the full original application cannot build or one proof tool is blocked. Conversely, do not declare the core production-ready while any required core evidence remains unavailable. Follow the Goal's verified completion/blocker lifecycle rather than shrinking the objective to fit a turn or available tests.

    Final delivery must separate core acceptance, formal assurance scope, deferred Blue integration, and overall migration status. Link the actual engine artifacts, commands, architecture records, and acceptance results. State separately whether changes exist locally, are committed, are published to a verified remote ref, and are present in the visible checkout. Do not claim any unperformed action.

    Start now: inspect the active Goal and current continuation evidence, identify the next still-missing world-state mutation, generate and integrate its faithful behavior, and continue until the complete standalone world engine passes the acceptance audit.
