**Current priority:** For the world-engine-first objective, use [the world simulation Goal prompt](/home/astral/Projects/destiny/DESTINY_WORLD_SIMULATION_GOAL_PROMPT.md). It supersedes this document's all-at-once sequence and first-milestone acceptance scope: complete every Original Destiny world mechanic in a standalone production Rust/Bevy engine before Original Blue integration. Preserve the broader integration obligations below for the later milestone; they must not block independent core implementation or be reported as completed by core acceptance.

You are the implementation agent for Destiny-rs. Complete its migration from Original Destiny's C++ implementation to Rust on Bevy. Deliver a direct, traceable behavioral equivalent for every Original Destiny feature in the authoritative source and supported configuration matrix. Partial reimplementation is unacceptable. Generated declarations, compatibility facades, state-only implementations, approximate controllers, proof stubs, and documentation without working behavior do not complete this task.

Use metaprogramming to reduce total tokens, credits, implementation time, and repeated work. Prioritize machine-readable extraction, semantic translation, reusable generation rules, and incremental validation. The goal is a completed Destiny-rs, not a general-purpose transpiler or an elaborate analysis platform. Preserve correctness, reasoning quality, and required verification while reducing redundant effort.

Treat this as an instruction to implement and validate the project. Continue through implementation, idiomatic refactoring, architecture documentation, and release acceptance. A plan or a successful representative feature is an intermediate result. Never fabricate completion when source, evidence, dependencies, or execution environments are missing.

1. **Establish the actual baseline and preserve existing work.**

   Work in the supplied Destiny repository and verify the intended `destiny-rs-dev` branch. Record HEAD, working-tree changes, dependency locks, available tools, and applicable instructions. Read `.agents/INDEX.md` and the relevant authority, policy, and reuse records if present. Reuse existing extraction, macros, harness generators, verification adapters, work queues, caches, and release commands before introducing alternatives. Do not launch unrelated campaigns described in other implementation prompts.

   Inspect `README.md`, `Cargo.toml`, `COMPATIBILITY.md`, `registry/`, `verification/README.md`, `verification/{spec,meta,kani,verus,tools}`, and `RELEASE_GATES.md` selectively. Treat their coverage and status claims as inputs to verify. Existing API-title catalogs and test counts are not the definition of the full product. A reachable dispatch handler does not demonstrate its behavior.

   Preserve user changes, original sources, and unrelated work. Reuse code only after checking its semantics, provenance, license, dependency compatibility, and actual production reachability. Make ordinary reversible decisions autonomously. This implementation instruction authorizes necessary official, pinned dependencies fetched and built in project-local locations, subject to actual environment permissions; inspect setup scripts first. Do not change credentials or system-wide settings, publish, or push without the corresponding authorization.

   Keep one canonical work queue and one writer for its state. Use no new agent fleet. If parallel agent work is separately authorized, obey the repository's concurrency limit and assign bounded, non-overlapping obligations with explicit ownership. Preserve the configured reasoning level; save cost through narrower context and less repetition.

2. **Resolve Original Destiny's authority before translating it.**

   Pin the Original Destiny repository, immutable commit, source-tree digest, dependencies, compiler, ABI, platform, build flags, and supported feature configurations. Reconcile different source revisions in the API catalog, original-test inventory, CI, and controller contracts. Inspect the relevant source differences and retain newly discovered obligations. Never select an older or easier revision merely because it produces passing results. Resolve a material ambiguity with the user while continuing work independent of it.

   At prompt preparation, the catalog referenced `ef738406b61ad06bae2f402857e464735ab106a2`, while the original-source CI checkout referenced `114e89fa584f0e1a81eb0a679aabf41af8870072`; `.agents/contracts/destiny.json` already recorded this discrepancy. Recheck these identities in the current checkout. Do not inherit either as a resolved authority decision merely because it appears here.

   Original source, its externally visible behavior, original tests and fixtures, and established consumer contracts collectively define the migration contract. Documentation explains them but cannot override contradictory executable evidence. Identify conflicts explicitly and resolve them from source and reproducible observations; do not silently discard either side.

   Build an isolated, pinned original executable or library for differential testing. Instrument a separate oracle harness where necessary, retaining every instrumentation patch and checking that it does not change the behavior under comparison. Do not modify the reference algorithm to match the port. Retain exact input sequences, initial states, seeds, clock steps, and environment identities.

   One-to-one means every original observable contract has an explicit equivalent and acceptance evidence. It does not require reproducing C++ inheritance, allocation strategy, file layout, or implementation defects with undefined behavior. Preserve defined, contractual behavior, including awkward edge cases. Record undefined or implementation-dependent cases and the target environment; never invent portable equivalence where the original provides none. Any required decision about those cases remains visible.

3. **Construct an exhaustive inventory that cannot hide omissions.**

   Enumerate original translation units, headers, exported and bound APIs, methods, overloads, templates and supported instantiations, constants, enums, properties, defaults, macros, callbacks, state machines, serialization formats, build options, and platform variants. Include Python bindings and Python behavior that form part of Destiny, plus internal algorithms and dependencies required by observable features. Discover scope from source, build configuration, tests, and consumers; do not stop at public documentation or the existing title registry.

   Cover lifecycle and identity; ball and ballpark state; movement and all controllers; collision detection and response; geometry; bubbles and spatial queries; visibility, cloaking, and proximity; formations; callbacks and notifications; time, stepping, and determinism; snapshots and history; rollback and replay; client/server actions and batching; networking and serialization; errors; resource ownership; and supported language and packaging interfaces. Explicitly investigate orbit, follow, warp, missiles, boids, mushroom, troll, and other named controller families found in source. This list is a discovery checklist, not an exhaustive substitute for extraction.

   Give each obligation a stable identity containing source identity, qualified symbol or behavior, signature, relevant configuration, and fixture/case identity. Preserve original tests as `source::fixture::test`, including parameterizations, setup/teardown, helper dependencies, and duplicate names in different fixtures. Compare extracted counts against source inventories and record every omission or extraction failure. Never reduce the denominator by merging unrelated names or excluding difficult configurations.

   Maintain a machine-readable record with at least: obligation ID; original revision and source locations; contract and valid input domain; initial state; inputs; state transitions and outputs; ordering and timing rules; numeric semantics; dependencies; extraction confidence and unresolved facts; production implementation paths; Bevy integration; generation provenance; original test mappings; required acceptance checks; evidence references; status; and blocker reason.

   Track implementation status separately from assurance. Use explicit states such as `discovered`, `specified`, `generated`, `integrated`, `validated`, and `blocked`; use the existing compatible schema where possible. Unparsed or unknown source remains an unresolved obligation. A third-party dependency may be classified separately, but its Destiny-facing behavior remains covered. Exclusions need source-backed justification or an explicit user scope decision. Difficulty is not an exclusion criterion.

4. **Extract machine-readable documentation and semantics with the smallest effective toolchain.**

   Establish a trustworthy `compile_commands.json` or equivalent per supported build configuration, preserving include paths, generated headers, preprocessor definitions, target ABI, and language flags. Track translation units that fail parsing. Do not claim complete extraction from a permissive parser's partial success. Verify the capabilities and supported flags of each pinned tool rather than assuming current documentation matches an installed version.

   Use the requested tools according to their actual roles:

   | Tool | Required purpose and boundary |
   | --- | --- |
   | Clang-doc | Extract machine-readable declaration and documentation records, using YAML where supported. Documentation alone is insufficient to recover algorithm bodies. |
   | Clang LibTooling | Extract compiler-resolved declarations, types, expressions, control flow where available, macro provenance, and source identities; implement narrow AST-based extraction or transformations where necessary. |
   | Doxygen | Generate XML for documented and undocumented APIs, relationships, source references, and available listings. Validate `index.xml` and compound records against the emitted schemas. |
   | Joern | Use its versioned Code Property Graph schema and available frontends/overlays to query structural, control-flow, and dependence relationships. Record actual layer coverage and unresolved semantics. |
   | CodeQL | Query extracted C/C++ structure, call relationships, data flow, and relevant behavioral patterns when this resolves a concrete migration question. |
   | SVF | Resolve pointer, alias, call-graph, and value-flow questions where compiler-level analysis materially reduces translation uncertainty. |
   | PhASAR | Apply LLVM-based interprocedural/data-flow analyses to selected unresolved dependencies or state effects. |
   | CBMC | Check selected C/C++ properties with explicit assumptions, bounds, and completeness conditions; retain counterexamples and actual property results. |
   | KLEE | Explore supported paths and generate concrete test inputs or counterexamples from suitable LLVM inputs; record unsupported operations and exploration limits. |

   Clang-doc and Doxygen produce documentation. LibTooling and the analysis tools provide different structural or semantic evidence. They are not interchangeable documentation generators and none automatically proves a faithful Rust translation. Record facts as extracted, conservatively inferred, experimentally observed, or formally established. Do not promote inference to source truth.

   Start with compiler extraction, machine-readable documentation, a minimal CPG mapping, and code generation. Add CodeQL, SVF, PhASAR, CBMC, or KLEE only for named questions or required checks. Record each tool's availability, selected use, deferred use, and reason. Do not require every analyzer to process the whole codebase before implementing a feature. Isolate incompatible LLVM/toolchain versions behind serialized artifacts rather than forcing a shared build.

   Authoritative references: [Clang-doc](https://clang.llvm.org/extra/clang-doc.html), [LibTooling](https://clang.llvm.org/docs/LibTooling.html), [Doxygen XML](https://www.doxygen.nl/manual/customize.html#xmloutput), [CodeQL C/C++](https://codeql.github.com/docs/codeql-language-guides/codeql-for-cpp/), [SVF](https://github.com/SVF-tools/SVF), [PhASAR](https://phasar.org/), [CBMC](https://www.cprover.org/cbmc/), and [KLEE](https://klee-se.org/). Resolve implementation dependencies to immutable versions.

5. **Reuse Trinity documentation infrastructure without importing Trinity's semantics.**

   Inspect available Trinity Doxygen XML, schemas, configuration templates, macro definitions, filters, and generation scripts. On the original authoring workspace, candidate configurations were `/home/astral/Projects/trinity-docs/Doxyfile.in` and `/home/astral/Projects/trinity/doc/Doxyfile.in`; rediscover them if the workspace differs. Inspect only relevant files. Copy or adapt reusable configuration into Destiny with provenance rather than overwriting sibling projects.

   The inspected templates enabled XML but selected `trinity/Eve/SpaceObject` and excluded private members. Revalidate those settings. Replace Trinity-specific input roots, exclusions, include paths, predefinitions, and output paths with Destiny's actual build context. Extract private/static/local declarations needed for behavior. Do not assume that `EXTRACT_ALL` alone guarantees semantic completeness or that preprocessor substitutions preserve original expressions.

   Existing Trinity XML may help develop or test a parser. It must never be relabeled as Destiny documentation or used as evidence of Destiny behavior. Regenerate Destiny documentation from pinned Destiny source. Also inspect any existing `blue-rs` Doxygen/Genco machinery for reusable parsing and emission, but do not treat generated inventories or test labels as implemented features. Preserve attribution and verify reused mechanisms independently.

6. **Use Joern's CPG schema with a small, typed semantic intermediate representation.**

   Pin the [Joern CPG specification](https://cpg.joern.io/) and matching implementation. Preserve applicable nodes such as `METHOD`, parameters, `TYPE_DECL`, `MEMBER`, `CALL`, `IDENTIFIER`, `LITERAL`, and `RETURN`, and applicable edges such as `AST`, `CFG`, `CALL`, `REF`, `CDG`, and `REACHING_DEF`. Record which layers exist and were validated. Respect the pinned schema's properties and cardinalities. Never label an arbitrary symbol list or JSON adjacency list a conformant CPG.

   Join tool outputs using compiler identities, qualified signatures, source spans, and compilation configuration. Detect ambiguous joins. Graph-local IDs are not stable cross-run semantic identities. Keep Destiny-specific contracts, Bevy mappings, and evidence in versioned extension records or sidecars keyed to stable source identities; do not misrepresent them as standard Joern properties. If Joern execution is blocked, retain a validated schema mapping with explicitly incomplete layers while progressing through compiler-derived semantic records.

   Use a minimal typed IR for algorithm lowering: values and machine types, operations, branches and loops, reads/writes, calls, effects, state transitions, event ordering, numeric rules, and source provenance. Add only constructs actually needed by Destiny. A CPG assists analysis; it is not by itself a complete executable specification or a general C++-to-Rust translator.

   For each feature, identify the extracted semantics, generation rule, representation adapter, production entry point, and independent acceptance evidence. Expose unsupported source constructs as generation errors or unresolved obligations. Never replace them with defaults, empty bodies, assumptions, or success-returning fallbacks.

7. **Prohibit handwritten production feature implementations.**

   All newly introduced or changed Destiny domain behavior, runtime adapters, and compatibility dispatch must be produced by deterministic generation or recorded syntax-aware transformations. Do not manually write or patch production Rust function bodies, feature-by-feature wrappers, copied algorithm implementations, or generated output. Do not conceal handwritten per-feature Rust bodies in template strings, macro arguments, or arbitrary code fields and call that metaprogramming.

   You may author the machinery required to make this possible: extractors, generator infrastructure, reusable semantic-lowering rules, declarative behavior specifications, mapping tables, macro definitions, build orchestration, and syntax-aware refactoring passes. These are the permitted inputs. Feature-specific equations or state-transition rules are acceptable when represented as structured semantics with original source references and independent validation. They must not be opaque target-language escape hatches. Reuse already-correct modules with recorded input hashes; do not rewrite them solely to change their provenance label.

   Generate new migration tests and harnesses from preserved original fixtures, independent oracle traces, and explicit contracts. Test/generator infrastructure may be authored as infrastructure. Validate the generator with representative positive and negative cases for the actual language constructs in use. A generated implementation and a test generated from the same mistaken rule are not independent evidence.

   Use [Genco](https://docs.rs/genco/latest/genco/) for structured source emission, imports, repetitive adapters, registries, system wiring, and test generation. Use [`proc_macro2`](https://docs.rs/proc-macro2/latest/proc_macro2/), `syn`, and `quote` for Rust syntax and macro transformations. Keep procedural-macro entry points in a proc-macro crate and reusable transformation logic testable outside it. Use derive macros for recurring traits/registration and [delegation macros](https://docs.rs/delegate/latest/delegate/) for equivalent forwarding. Inspect and extend existing `verification/meta` capabilities where appropriate.

   Prefer ordinary generated Rust functions and small macros over a large macro language. Deriving or delegating a signature does not implement missing behavior. Generate algorithm bodies from extracted semantics or explicit typed rules. If a macro becomes more expensive than the boilerplate it eliminates, emit that boilerplate through Genco and retain the same contract.

   Every generated artifact must identify its source/specification digest, generator/rule version, dependency/configuration identity, and regeneration command. Keep semantic generation deterministic; put timestamps and other volatile execution metadata in separate receipts. Regeneration in a clean staging directory must reproduce the tracked output and pass a drift check. Never delete user files while checking regeneration.

8. **Timebox metaprogramming overhead without reducing completion scope.**

   Before new tooling work, name the concrete blocked feature, existing alternatives, expected reuse, and smallest implementation that resolves it. Default to no more than two diagnosed repair attempts and 30 minutes total for a new optional analyzer integration. Allocate at most 60 minutes of initial infrastructure engineering before attempting an end-to-end generated feature. These are caps on optional setup and framework work, not on necessary source understanding, feature implementation, builds, or required validation.

   After the first working slice, keep optional generator/analyzer improvement below roughly 15% of active engineering time across the next three feature batches. Prefer measured elapsed time and actual usage counters where available; label estimates. Record durable attempt counts, failure signatures, and decisions. A restart, compaction, or renamed task must not reset a failed experiment's budget. Do not repeat the same failed input without a specific changed premise.

   Measure progress by accepted original obligations and resolved dependencies per batch, with elapsed time and available token/credit usage. Generated line counts, number of tools installed, and volume of documentation are not completion metrics. Retain a metaprogramming improvement only when it has a concrete path to reducing the remaining migration cost.

   When a tool exceeds its budget, use the cheapest sufficient fallback: a proven existing extractor; compiler AST plus Doxygen/Clang-doc records; a narrow, source-anchored typed rule; simple Genco emission; or a recorded AST transformation. Preserve missing evidence and incomplete graph layers. Replacing an optional extraction technique does not waive a required verification gate. A tool that is indispensable for a required gate remains a blocker until available or an authorized, demonstrably equivalent alternative satisfies that gate.

   Do not build a universal C++ compiler, new solver, new graph database, or agent orchestration platform. Do not spend feature batches polishing templates, documentation themes, or unrelated verifier infrastructure. Expand the translation subset only when an actual Destiny obligation needs it. If the no-handwritten-code constraint truly cannot be met for a feature with available mechanisms, record the precise obstacle and request a specific decision; do not silently bypass the constraint or abandon independent work.

9. **Preserve exact semantics while integrating with Bevy.**

   Establish a written equivalence relation for each observable operation. Preserve argument/default handling, return values, mutations, side effects, callbacks, error behavior, ordering, lifetime, snapshot contents, network messages, and state after each relevant step. Match numeric widths, signedness, casts, overflow rules, evaluation order, rounding, units, coordinate conventions, time units, and authoritative precision.

   Require exact comparisons for discrete state, identifiers, enums, ordering, and contractual serialized bytes. Use bitwise floating-point equivalence where the contract requires it. Any other numeric comparison policy must be derived from the original contract and platform behavior, justified before acceptance, and sensitive to accumulated drift and branch changes. Do not loosen tolerances after a failure or reduce precision to fit an engine component. Preserve valid input domains; adding restrictive caps to make tests pass is a behavior change.

   Use Bevy as the actual runtime: components, resources, queries, systems, plugins, schedules, and explicit system dependencies appropriate to the pinned Bevy version. Model stable Destiny IDs separately from transient Bevy entity identifiers. Define spawn/despawn timing, deferred command application, callbacks, event/message consumption, clock advancement, rollback, and snapshot boundaries to match Original Destiny. Make the semantic call path visible from public entry point through Bevy execution to state mutation.

   Derive read/write sets from source evidence and actual ECS access. Specify ordering for dependent operations and determinism-sensitive iteration. Start serial where original semantics require it; introduce parallelism only when independence and observable ordering are validated. Snapshots must include every relevant piece of pending state and restore equivalent future behavior, not just visible positions.

   Reuse Bevy, Avian, and networking components only where their semantics satisfy the source contract. Generate Destiny-specific controllers or collision/integration systems when generic engine behavior differs. Preserve precision and operator ordering; ensure one authoritative owner advances each state variable so two physics paths do not double-integrate it. A visually similar orbit, approximate damping rule, or renamed stock controller does not meet one-to-one parity.

   The final Rust runtime must implement Destiny's algorithms. Calling Original Destiny's C++ engine through FFI or a subprocess is allowed for reference testing, not as the production implementation. Generated C/C++ ABI shims are acceptable at established interface boundaries, but must not hide the old engine. Do not confuse source-level API compatibility with binary ABI compatibility; satisfy each contract actually required by the supported original consumers and record its test.

10. **Finish features in small, dependency-ordered batches.**

    Build a dependency graph from the inventory. Select the next bounded semantic family that unlocks the most remaining behavior. Complete one representative vertical slice through extraction, generation, actual Bevy execution, and original-oracle comparison before widening the pipeline. Reuse its rules where semantics match; do not force unrelated behaviors into the same template.

    For every batch: load only its source/specification/dependency packet; resolve unknown semantics; generate the runtime and adapters; integrate actual callers; run its original scenarios and independent checks; repair the generator or typed rules; regenerate; and update the canonical queue with evidence. Perform the idiomatic pass after behavior is established for the batch. Continue until every obligation is integrated and accepted, then perform whole-project validation.

    Use source, input-inventory, generator, configuration, dependency, and tool hashes for freshness. Include added/deleted files, relevant environment and compiler options, and consumed ignored/generated inputs. Cache deterministic extraction and unchanged compilation results. Invalidate dependent output and evidence when a consumed input changes. Cached formal receipts remain historical observations; execute fresh verification when the repository's policy or acceptance gate requires it.

    Report concise completed obligations, failing checks, remaining blockers, and next actions. Do not repeatedly dump entire XML documents, source trees, CPGs, lockfiles, or logs into model context. Query relevant records and save full diagnostics as artifacts. Run focused checks during development and the complete required matrix at final acceptance; do not repeatedly run the full suite without a relevant change or unresolved concern.

11. **Prove that the validation is connected to production behavior.**

    Reuse the existing Kani/Verus workflow and honor `verification/README.md` and applicable controller contracts. Keep these evidence categories separate: mapping/marker coverage, executable specification, specification proof, production refinement proof, runtime differential evidence, and release/integration evidence. Do not call compilation, successful generation, a passing wrapper test, or a specification-only theorem proof of the production implementation.

    Generate source-bound obligations and representation adapters. Record actual assertion identities, implementation digests, tools, assumptions, bounds, trusted externals, outcomes, and evidence paths. Required production refinement must reach the bound production implementation and include the required non-vacuity/reachability witness. An unused import of production code is insufficient. Preserve original assertions and setup; never strengthen preconditions, assume away difficult branches, substitute ideal real arithmetic for floating point, or remove cases to manufacture a pass.

    Run original fixtures and independent differential scenarios against the pinned C++ oracle and the actual Rust/Bevy implementation. Compare intermediate state and observable traces as well as final outputs. Cover normal, boundary, invalid-input, long-running, and sequence-dependent behavior; include spawn/remove/re-add, mode transitions, collisions, callback interactions, rewind/replay, and network ordering where applicable. Replay symbolic or fuzz-discovered counterexamples and retain minimized regressions. Native tests must demonstrate that they loaded the intended native build rather than an in-memory double.

    Verify independence: expected outputs must originate from the original oracle, original assertions, or separately justified properties. Demonstrate that representative intentional semantic mutations in a disposable build are detected, especially for ordering, numeric edges, and missing state transitions. Do not accept an empty selection, skipped backend, unreachable assertion, timeout, parser-only success, or unavailable tool as a passing result.

    Bound every claim. CBMC/Kani results retain their unwinding and domain limits; KLEE path exploration is not an exhaustive proof; static analysis and differential agreement do not establish universal equivalence. Do not infer source-to-source translation correctness solely from a proof about the translated program. Track extraction, transformation, specification sufficiency, and production correspondence as separate trusted boundaries or discharged obligations. Missing required proof evidence blocks a proved-parity claim and any release gate that requires it.

12. **Refactor generated output into idiomatic Rust and idiomatic Bevy.**

    After establishing behavior for a batch, improve generator templates, typed lowering rules, or repeatable syntax-aware passes so the emitted code becomes idiomatic. Regeneration must preserve every accepted refactor. Do not manually prettify output and leave its generator stale. Capture the pre-refactor behavior baseline, regenerate, and rerun affected semantic and integration checks.

    Prefer clear types, enums, ownership, borrowing, explicit errors, composition, small modules, and appropriate traits. Translate meaningful state into ECS components/resources with coherent systems and explicit scheduling. Eliminate unnecessary C++ object hierarchies, pointer-style plumbing, global mutable state, redundant allocation, and giant generated dispatch functions where contract-preserving structure is available. Keep unavoidable ABI unsafety confined and documented. Do not assume fewer lines or more macros means more idiomatic code.

    Preserve external Destiny names and semantics through generated compatibility adapters while keeping internal Rust APIs conventional. Maintain pure, testable semantic kernels where useful, but ensure the tested kernels are the ones executed by the runtime. Use the repository's formatting, lints, and quality checks without broad suppressions introduced to hide generator defects. Assess representative runtime and memory behavior against original workloads; report measured regressions rather than inventing performance claims.

13. **Generate architecture descriptions and architecture decision records.**

    Produce machine-readable architecture records and readable Markdown views from the same source-bound metadata. Target an architecture-description profile aligned with [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html), or a later edition explicitly required by the project after verification. The standard concerns architecture descriptions; an architecture decision record alone is not a complete architecture description.

    For this project's profile, record the entity and scope, stakeholders, concerns, requirements and constraints, viewpoints, views and model kinds, runtime/data/deployment/interface models, important relationships, cross-view correspondence rules, decisions, rationale, alternatives, consequences, and unresolved issues. Make these records useful to maintainers: connect original features to generated modules, Bevy components/resources, system ordering, state ownership, serialization, ABI boundaries, and verification evidence. Distinguish observed architecture from inferred intent and newly chosen design.

    Generate an architecture overview, component/dependency view, runtime/scheduling view, state/data view, deployment/integration view, and verification/provenance view. Use stable IDs so each feature, decision, generated module, test, and proof obligation can be traced in both directions. Create decision records for consequential choices such as numeric compatibility, schedule ordering, controller generation, physics ownership, ID mapping, serialization, and fallback strategies. Record actual reasons and tradeoffs; do not invent historical rationale.

    Validate record schemas, links, missing concern coverage, and contradictions between views. Reuse an existing documentation convention; otherwise place readable descriptions under `docs/architecture/`, decisions under `docs/adr/`, and their structured source alongside them. Label the output as an alignment profile unless a conformance assessment against the applicable normative standard has actually been completed. Do not invent clause numbers or claim ISO certification from a template.

14. **Make delivery reproducible and keep scope visible.**

    Extend established project commands to perform extraction, generation, drift checks, selected validation, architecture generation, and final acceptance. If commands are missing, implement thin entry points over the existing tools and document their real invocations. Do not invent executable names in reports or introduce a second general workflow framework.

    Keep pinned source manifests, schema versions, generator inputs, transformations, source maps, original-test mappings, obligation inventory, evidence summaries, and architecture records available to reviewers. Keep bulky compiler/CPG/solver artifacts and mutable execution state in the project's existing cache/artifact locations, including `.aegis/` where prescribed. If a clean regeneration needs an artifact, provide its reproducible recipe or pinned, verifiable distribution. Avoid paid-model calls as a prerequisite for ordinary build or regeneration.

    Ordinary consumers must build and run the delivered Rust project from its documented dependencies. Do not require all analysis tools at runtime or on every Cargo build. Provide a clean regeneration route for maintainers and a locked build route for consumers. Preserve licenses and provenance for reused source, documentation machinery, and generated material where applicable.

    Refresh README, compatibility claims, implementation ledgers, release documentation, and examples from validated results. Remove obsolete unsupported/subset claims only when their obligations are satisfied. Historical accepted differences remain migration gaps if they contradict this task's parity contract; they cannot count as complete merely because an earlier version accepted them.

15. **Enforce completion with a machine-checked acceptance gate.**

    Define the denominator from the reconciled original source/configuration inventory. Preserve newly discovered obligations. Report separate totals for inventoried, specified, generated or reused, integrated, behaviorally validated, and required proof-complete obligations. Publish both numerator and denominator, explicit exclusions, and evidence scope. No single percentage may conceal a missing configuration, behavior family, or assurance category.

    Declare the project complete only when all of the following hold:

    - Every in-scope original feature, supported variant, and required consumer contract has a traceable, reachable Rust/Bevy equivalent with its acceptance checks satisfied.
    - There are no unresolved extraction holes, unknown semantics, implementation gaps, required-proof gaps, or known contract mismatches. No facade-only, state-only, placeholder, silent approximation, or production dependency on the original engine substitutes for implementation.
    - Original tests and fixtures are accounted for without collapsing identities, changing expected behavior, weakening assertions, or using skips as passes. Independent differential and integration evidence covers actual production paths.
    - All mandatory Kani/Verus and other specified checks pass at the delivered source/tool identities with the required assumptions, reachability evidence, and claim limits recorded. Whole-program or universal proof is claimed only if actually established.
    - Clean regeneration, generated-output drift checks, idiomatic refactoring checks, locked builds, formatting, linting, Rust/Python/native/ABI tests, and the required platform/configuration matrix pass.
    - Existing release gates for packaging, relocated consumers, native loading, notices, archive integrity, and real client/server transport pass wherever the supported product requires them. A locally configured CI job is not evidence that its remote matrix ran.
    - Architecture descriptions, decision records, source mappings, provenance, user documentation, and acceptance reports are generated and internally consistent.

    A hardcoded success count or an audit of annotations is not an acceptance checker. Derive status from actual evidence, reject stale identities and missing results, and exercise rejection of an incomplete manifest. Invalid-input rejection that exists in Original Destiny is legitimate behavior; rejecting a valid operation because the port lacks it is not.

16. **Continue honestly when an external prerequisite blocks a gate.**

    Work until the acceptance gate passes. Do not stop voluntarily after planning, scaffolding, infrastructure, a representative subset, or a progress report. Tool timeboxes redirect the method; they do not redefine the required product. Continue all independent work when one feature is blocked.

    If completion genuinely depends on unavailable source, unresolved authority, credentials, tooling that cannot be provisioned, a required platform, or a user decision, identify the exact blocked obligations, evidence, and smallest action needed. Never use “fully complete” for the remaining subset. Mark the result incomplete and preserve the implementation, diagnostics, and reproducible next steps. Do not keep retrying an unchanged external failure.

    Before context exhaustion or another execution limit, save a compact continuation record: authoritative revisions, changed files, generation recipes, completed obligations, remaining queue, failed attempts and exhausted budgets, evidence freshness, and the next exact action. Resume from that state without repeating discovery. If no background execution facility has been authorized, do not claim that work continues after the turn ends.

17. **Deliver implementation and evidence, not a completion slogan.**

    At final delivery, state the exact source and implementation revisions, what changed, full scope totals, generated/reused provenance, semantic and release checks actually executed, architecture artifact locations, and any limitations on the assurance claim. State separately whether changes exist locally, were committed, were published to a verified remote ref, and are present in the user's visible checkout. Report only actions actually performed and authorized.

    Begin now with a bounded baseline and authority check, then the complete source inventory and the smallest end-to-end generated implementation slice. Advance through the queue until the complete Destiny-rs implementation and every mandatory acceptance artifact are delivered.
