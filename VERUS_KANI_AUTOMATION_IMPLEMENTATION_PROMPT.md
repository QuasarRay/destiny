You are the implementation agent for this repository. Vendor Verus and Kani, make Verus the core verification engine, and bring Kani's low-friction automation, harness generation, testing workflow, and applicable symbolic-analysis capabilities to the Verus-backed path. Reuse existing implementations wherever their semantics and dependencies permit. Deliver working code, reproducible builds, automated validation, and precise evidence; documentation, wrappers around unchanged Kani execution, or generated proof stubs alone do not satisfy this task.

Optimize for correctness and maintainability, then reduce total agent and compute cost through focused inspection, code reuse, durable work records, and selective execution. Do not save tokens or credits by weakening specifications, reducing required verification, concealing unsupported behavior, or silently lowering a mandated reasoning setting.

Treat the requirements below as the intended architecture and acceptance criteria, not a claim that either upstream already implements the integration.

1. **Establish the baseline and use existing infrastructure.**

   Record the repository, branch, commit, dirty files, available tools, and applicable instructions. Preserve user changes. Make ordinary reversible implementation decisions autonomously within the task. This prompt authorizes fetching official pinned source dependencies and building them in project-local directories, subject to actual environment restrictions. Inspect setup scripts before running them. Keep credential changes, system-wide configuration changes, publication, and unrelated work outside this scope.

   In Destiny, route context through `.agents/INDEX.md`; inspect `.agents/AGENTS.md`, `.agents/modules/destiny/POLICY.md`, and the relevant entries in `.agents/modules/destiny/BRANCH_REVIEW.md`. Reuse the existing catalog, context ledger, dependency planning, subprocess supervision, binding, evidence parsing, and durable budget mechanisms. Follow the maintained-controller/export/bootstrap workflow for controller changes. Preserve any existing modifications in `.agents`.

   Start with `verification/README.md`, `verification/{kani,verus}/README.md`, `verification/tools/run_verus_campaign.py`, the relevant generators and acceptance tools, and `.github/workflows/formal-verification.yml`. Inspect existing implementation prompts only for relevant contracts; do not accidentally launch their separate repository-wide proof campaigns. Revalidate versions, tool availability, and historical results against the active checkout.

   Record a compact baseline of existing commands, supported behavior, evidence categories, and required CI gates. The current project separates concrete Kani obligations from Verus refinement obligations. Changing the default engine must preserve those obligations: migrate an obligation only with demonstrated equivalent or stronger coverage. Keep residual Kani obligations visible until that criterion is met.

2. **Inspect upstream implementation boundaries before designing adapters.**

   Use the official repositories `verus-lang/verus` and `model-checking/kani`. Resolve selected revisions to immutable commit IDs. Inspect narrowly: build manifests, toolchain files, public interfaces, driver entry points, relevant compiler passes, and representative tests. Read implementation code for the functions you intend to reuse; documentation and RFCs alone do not establish that a feature exists at the selected revision.

   Kani's default CBMC path and Verus's deductive verification pipeline have different semantics and intermediate representations. Locate the actual ownership of each capability. In particular, distinguish Kani orchestration and compiler transformations from symbolic execution performed by CBMC. Do not assume vendoring Kani supplies a portable symbolic executor that can simply be relinked to Verus. Check existing alternative backends and extension seams at the chosen pin before adding another.

   Create a compact reuse matrix with these columns: capability, pinned upstream file/symbol, dependencies and semantic coupling, existing local equivalent, integration decision, required adaptation, validation, and remaining limitations. Cover at least Cargo discovery, entry-point selection, autoharness generation, arbitrary input construction, contracts and stubs, compiler instrumentation, execution scheduling, result handling, counterexample playback, test infrastructure, and CI packaging.

   Classify each component as directly reusable, reusable through a thin adapter, extractable through a small upstream patch, inherently backend-specific, or unavailable at the pin. Justify new code with a concrete missing capability or incompatible semantic boundary. Stop exploratory reading when the next implementation decision is supported; reopen research only for a specific unresolved question.

3. **Vendor both projects reproducibly and preserve provenance.**

   Follow repository conventions; otherwise use `third_party/verus` and `third_party/kani`. Include actual buildable source for both, with the required submodules and source dependencies pinned. Prefer tracked source snapshots or subtrees when there is no established policy. A submodule is acceptable when the repository's bootstrap and distribution process reliably materializes its exact contents. A download script or dependency name without reproducibly available source does not complete vendoring.

   Reuse existing project pins when suitable. If a required capability needs another revision, document that specific reason and validate the upgrade. Record repository URL, commit, source/tree digest, toolchain versions, relevant dependency locks, solver and CBMC versions where used, checksums for fetched artifacts, licenses/notices, and local patch identities. Preserve upstream attribution and applicable notices; check each reused component rather than assuming the repository's top-level license covers everything.

   Keep modifications small and reviewable through the established patch mechanism, or an ordered patch series with rationale and upstream source locations. Provide deterministic bootstrap, verification of vendor integrity, patch application, and an update procedure. Avoid fetching complete histories or building unrelated targets when pinned snapshots suffice.

   Account for separate rustc-private dependencies, Rust toolchains, target directories, and solver requirements. Use process boundaries or small versioned data interfaces when toolchain incompatibility makes shared linking impractical. Do not force both verifier compiler internals into one Cargo workspace merely to make the layout look unified. After explicit dependency provisioning, test a build and focused verification without network access; identify anything still preventing reproducibility.

4. **Implement one shared automation layer with Verus as its default engine.**

   The intended flow is project discovery and obligation selection, followed by reusable automation, a Verus adapter, the actual Verus verifier, and source-bound evidence. Keep Kani as an explicit compatibility/reference backend and as a supplier of reusable components. A successful Kani run cannot discharge a required Verus result or silently rescue a failed Verus invocation.

   Prefer, in order: an existing library/API; direct use of the vendored implementation; a small patch exposing backend-independent functionality; a single shared extraction with provenance; and only then a narrowly justified new implementation. Where functionality already exists locally, reuse it rather than importing a second scheduler, cache, report database, process runner, or CLI parser. A single extraction must replace duplicate implementations rather than leave several independently maintained copies.

   Inspect Verus's existing Cargo integration, source handling, diagnostics, and proof automation before building equivalents. Prefer supported interfaces. Any internal API integration must be pinned, isolated, and tested. Define only the backend boundary needed for real callers: capabilities, input identity, obligation scope, invocation, diagnostics, artifacts, and outcomes. Do not build a speculative general-purpose verification framework.

   Provide a capability registry so discovery can explain support and gaps before expensive execution. The default Verus path must run without invoking Kani's proof engine or CBMC. Kani execution must require an explicit mode or an explicitly reported existing migration gate. If retained Kani compiler functionality is used for metadata, distinguish it from Kani verification and record the dependency.

   Demonstrate substantive reuse of an identified Kani implementation component in the Verus execution path. Copying its command names, reproducing its UI, citing its architecture, or keeping its code unused in a vendor directory does not demonstrate reuse.

5. **Specify the semantic bridge and its trusted boundaries.**

   Define an initial supported Rust/Kani-style subset from actual repository needs, with a capability matrix and explicit rejection for everything else. Prefer verifying original executable code or a shared production kernel. If adapting ordinary Rust or Kani harnesses requires syntax rewriting, instrumentation, or lowering, use compiler-aware mechanisms where needed; textual substitution is not a general Rust translator.

   State how source executions and assertion sites correspond to generated Verus obligations. Preserve machine widths, signedness, overflow and wrapping behavior, casts, panic behavior, evaluation order, and the supported memory model. Mathematical integers or real numbers cannot silently replace machine arithmetic or IEEE-754 behavior. Treat references, aliasing, raw pointers, allocation, unsafe code, external calls, concurrency, and unsupported library operations explicitly.

   Map each nondeterministic input to a properly quantified value or input sequence with its original domain, validity conditions, ordering, and independence. Repeated or conditional `kani::any()` calls cannot become one shared value accidentally. Carry user assumptions into an explicit scoped precondition or equivalent path condition; preserve their program-point meaning and report their origin. Assertions remain proof obligations. Never turn a failed assertion or difficult branch into an assumption.

   For inferred invariants, stubs, and contracts, prove their obligations before using them as verified summaries. Detect circular justification and record unresolved dependencies. Candidate generation may be heuristic; acceptance must be checked by Verus. Restricted constructors, chosen generic instantiations, input-size bounds, or omitted aliasing relationships must remain visible restrictions on the result.

   Review precondition consistency and reachable assertion sites. Provide witnesses or other supported checks for generated obligations where feasible, and retain an explicit unresolved non-vacuity status when unavailable. Detect deliberately contradictory assumptions in regression tests. Follow any stronger project requirement for concrete source-located reachability witnesses.

   A proof about a translated program does not by itself prove the original program. Identify translation correctness as a separate obligation and trusted-code boundary. Use transformation contracts, structural checks, independent semantic review, and targeted differential tests; machine-check preservation where practical. Differential agreement alone is not a universal translation proof. Do not claim more original-program assurance than the established correspondence supports.

6. **Bring symbolic analysis to the Verus path without recreating CBMC.**

   First reuse Verus's existing reasoning and encodings, along with applicable Kani discovery, preprocessing, and instrumentation code. Implement symbolic input and path handling for the supported subset so the normal Verus workflow can check generated obligations without concrete input enumeration. Generated safety checks should cover supported runtime failure conditions as well as user assertions; report which classes remain unsupported.

   Where path splitting, branch conditions, loop handling, or search heuristics are needed, inspect and reuse existing implementations before adding them. Do not construct a parallel solver stack or call Z3 directly and label the result a Verus proof. A lower-level adapter that bypasses Verus source, typing, mode, or lifetime checks needs its own explicit preservation justification and trust accounting.

   For general loops and recursion, generate or request candidates for invariants, summaries, and termination measures as required by the intended claim, then check them. Distinguish partial correctness from termination. For bounded exploration, expose all bounds and preserve the corresponding scope. An unwind limit is not a proved loop invariant. A globally valid claim needs an adequate induction argument or checked exhaustion/completeness condition, not an arbitrary depth cutoff.

   Extra search and synthesis may propose lemmas, contracts, or tests; it may not approve its own output. Implement the required minimum on a supported integer/Boolean subset first, including branching and either a justified loop proof or an explicitly bounded loop result. Keep broader pointer, floating-point, concurrency, and language support as individually tracked requirements with honest status. Do not present a restricted demonstrator as general Kani compatibility.

7. **Deliver automatic harnessing and a low-friction developer workflow.**

   Extend an existing command when appropriate; otherwise introduce one clearly named project command. Provide operations equivalent to doctor/setup, discover/list, verify, explain, and replay, without requiring those exact names or inventing nonexistent upstream flags. Expose package, target, feature, and function selection plus time, memory, and concurrency limits through one configuration source.

   Reuse applicable Kani selection and generation logic. Automatically discover eligible functions and existing harnesses, create source-mapped Verus obligations, check them, and show actionable results. Keep generated material separate from hand-maintained source and make regeneration deterministic. Use existing metadata to skip duplicate discovery and compilation while accounting for all selected obligations.

   Define a zero-additional-annotation path for supported safety properties and a low-annotation path for user contracts. Infer only what is justified by language semantics, existing assertions, or declared contracts. The absence of a behavioral specification cannot become a claim of arbitrary functional correctness. Lifting an ordinary unit test to universal inputs requires a justified domain and property; running it symbolically does not supply missing intent.

   Report every discovered candidate as selected, intentionally excluded, unsupported, or blocked, with a reason. A selected unsupported obligation makes the run incomplete. Empty selection must be explicit and must not accidentally satisfy a CI gate. Explain the smallest useful next step when annotations, invariants, or models are missing.

   Default to deterministic local tooling for normal verification. Do not make a paid model call a prerequisite for using the delivered CLI. Any optional AI-assisted proof search must be opt-in, budgeted, auditable at the artifact level, and subject to the same Verus validation.

8. **Reuse testing and counterexample infrastructure with truthful diagnostics.**

   Adapt the relevant Kani regression runner, fixture conventions, test discovery, and playback components instead of rebuilding them. Reuse Verus's test infrastructure where it is the closer fit. Separate reusable control logic from backend-specific expected messages and trace formats; do not blindly regenerate expected output to hide regressions.

   Distinguish an unproved Verus obligation from a concrete source-level bug. Solver models can refer to abstractions, unavailable summaries, or proof-state values. Reuse Kani playback formatting and execution where compatible, but decode the actual Verus-side data and validate the mapping. For supported cases, produce a deterministic regression test and execute it against the original source before calling the failure a reproduced counterexample. Otherwise report a proof failure or non-replayable diagnostic without inventing a trace.

   Use a small representative corpus with known-valid, known-invalid, and unsupported examples. Differential checks against pinned Kani are useful only when source behavior, assumptions, target, bounds, and checked properties match. Kani is an independent comparison, not an oracle whose agreement proves the new adapter sound. Investigate disagreements, minimize examples, and retain their outcomes.

9. **Keep evidence precise, complete, and tied to the actual inputs.**

   Extend the existing evidence model. Record stable obligation identity, source/assertion locations, production or model scope, source and dependency digests, generated-input and translator identity, vendor/patch/tool/solver versions, target/features/configuration, explicit assumptions, trusted boundaries, bounds, executed command, exit status, diagnostics, and artifact digests. Account explicitly for vendor and submodule trees; do not broadly exclude them to make source binding succeed. Check identities before and after execution; reject mismatches and truncated or malformed results.

   Keep engine, outcome, scope, and provenance independent. Distinguish checked-valid, refuted or reproduced failure, proof failure, unknown, timeout, resource exhaustion, unsupported, infrastructure error, skipped, and stale. A scoped Kani result may be complete for its stated domain; a Verus result may still be bounded or conditional. Engine names alone do not define assurance strength.

   Only a real completed Verus run checking the exact expected inventory may produce a checked Verus result. Validate individual expected obligations and unresolved checks, not just process exit or a summary count. Compiler success, parser tests, markers, generated stubs, and runtime comparisons remain separate evidence. Expose every `assume`, admitted or external body, trusted specification, and disabled check relevant to the result.

   Preserve Destiny's fixture-qualified case identities, evidence categories, and outstanding correspondence obligations. Do not change denominators, merge unrelated tests with the same name, or relabel incomplete cases as proved to improve coverage. Proving the automation tool, a specification model, or a shared kernel does not automatically prove the whole application.

10. **Enforce token, credit, and execution economy structurally.**

    Maintain one compact canonical work queue and decision record using the existing controller. Each work item needs its obligation, prerequisite identities, relevant files, current status, evidence links, next action, and attempt budget. Reuse these records across restarts; do not create competing plans or repeatedly reconstruct the same architecture from chat history. Store concise facts and decisions, not private deliberations.

    Before an expensive action, identify the concrete question it answers, the smallest required inputs, its expected cost, and the applicable limit. Reuse an existing answer when its provenance and freshness support the same use. Prefer `rg`, symbol-level reads, and bounded excerpts to whole-tree dumps. Search local pinned sources first, then narrow official upstream references for unresolved questions. Summarize long logs while retaining complete artifacts on disk.

    Deduplicate work by relevant source/dependency closure, obligation, configuration, and tool identities. Include additions, deletions, build scripts, generated inputs, feature flags, target properties, vendor patches, and relevant environment settings. Hash-based reuse must not rely on timestamps alone. Share build artifacts only across compatible identities, and deduplicate identical in-flight tasks.

    Cache dependency downloads, builds, discovery, and deterministic reports where safe. Label previous proof results as historical reuse; never call them newly executed. Follow the repository's fresh-proof rules. Run the required final proof once against the delivered bound inputs, reuse that same fresh receipt for its reporting consumers, and rerun if relevant identities change.

    Adopt existing durable attempt and resource budgets. Where none exist, allow one expensive attempt and at most one evidence-driven retry for the same failure before recording the blocker and moving to independent work. A retry needs a diagnosis and a concrete change in input, configuration, or external condition. Preserve cumulative task budgets across source edits, renamed work items, restarts, and delegation so they cannot reset the limits.

    Use focused checks during development, then run the required integration/regression gate once per relevant final state. Broaden or repeat testing for changed dependencies, new failures, or unresolved risk, not ritual. Reuse normalized failure signatures to avoid speculative retries. Cap solver jobs by available CPU and memory; concurrent verifier processes are different from spawning more AI agents.

    Work sequentially unless the user or applicable policy explicitly authorizes delegation. If authorized, use only bounded independent assignments, disjoint ownership, a small source packet, and one shared evidence record. Avoid duplicate audits, nested fan-out, and full-history prompts. Preserve mandated reasoning quality; reduce unnecessary context and actions instead.

    Track actual token/credit usage only when exposed by the host. Otherwise report observable proxies such as tool calls, build counts, solver time, cache hits, and retries, clearly labelled as proxies. Do not invent dollar savings. Honor explicit user budgets; if exhausted, leave a resumable checkpoint and accurate incomplete status. A budget limit is not permission to fabricate completion.

11. **Implement in dependency order and continue beyond the first demonstration.**

    First deliver the inspected baseline, reuse matrix, vendored sources, and a working pinned build. Next establish a vertical slice: discover a real entry point, exercise reused Kani code, generate or adapt its Verus obligation, invoke Verus, and produce source-bound evidence. Include one valid case and one failing case, and run this slice with CBMC execution disabled.

    Then add the remaining automation, symbolic-input/path features, supported loop handling, replay, capability reporting, focused regression corpus, and CI integration. Prove or reject each semantic extension before expanding its advertised support. Migrate representative existing project obligations while keeping their source/specification relationships intact.

    Keep milestones small enough to diagnose. Do not stop at architecture notes, vendor directories, a CLI skeleton, or a toy proof. Continue until the required capabilities below work for the declared subset, or an evidence-backed external/semantic blocker prevents further progress. Preserve unresolved broader requirements in the final capability matrix; do not silently redefine them as completed or mark the overall task complete.

12. **Validate the acceptance criteria and hand over reviewable results.**

    Completion requires all of the following, with commands and current artifacts:

    - Both upstreams are vendored at immutable revisions; build inputs, patches, notices, and bootstrap/update instructions are recorded and reproducible.
    - An actual Kani implementation component is exercised in the Verus automation path, with file/symbol provenance and a focused regression protecting its behavior.
    - The default verification command checks selected supported obligations through Verus; explicit Kani comparison remains distinguishable. The Verus path succeeds with CBMC execution disabled.
    - Automatic discovery and harness generation check supported safety/contract examples with no manual per-function harness. Symbolic branching and declared loop support execute through the Verus path.
    - A valid fixture passes; an invalid fixture is rejected; the supported failing fixture produces a replayed source-level regression. Unsupported syntax and missing invariants produce accurate, actionable statuses.
    - Semantic regressions cover machine arithmetic edges, independent nondeterministic values, scoped assumptions, selected generic/bounded cases, and the applicable memory restrictions. Contradictory assumptions cannot produce misleading accepted coverage.
    - Timeouts, missing tools, empty selections, selected unsupported cases, stale inputs, mutated source, malformed/truncated output, and mismatched obligation inventories cannot become proof success.
    - Existing required verification and test gates remain effective. CI retains inspectable evidence and fails on unresolved required work. No migrated obligation loses coverage without remaining explicitly incomplete.
    - A small like-for-like measurement reports cold/warm build or verification cost, repeated-work avoidance, and any performance regressions. Do not run an open-ended benchmark campaign.

    The handoff must explain the architecture, supported subset, reused upstream files/symbols, unavoidable new code, exact verification results, trusted boundaries, remaining compatibility gaps, and the commands needed to reproduce the result. Include a first-use example, configuration reference, migration instructions, and vendor-update procedure. Reuse existing document/report locations; avoid paperwork that duplicates another authoritative record.

    Report implementation state, validation state, and any remote publication separately. Give exact paths and current revisions. Never claim complete Kani feature parity, unrestricted Rust verification, end-to-end translation soundness, completed Destiny parity, or publication unless the corresponding evidence exists.

Use these primary references as starting points, then inspect their equivalents at the selected commits. They describe upstream capabilities, not an already-existing Verus/Kani integration:

- [Verus source repository](https://github.com/verus-lang/verus) and [architecture overview](https://github.com/verus-lang/verus/blob/main/source/CODE.md).
- [Verus build instructions](https://github.com/verus-lang/verus/blob/main/BUILD.md).
- [Kani source repository](https://github.com/model-checking/kani) and [source build instructions](https://model-checking.github.io/kani/build-from-source.html).
- [Kani's explanation of its CBMC-based architecture](https://model-checking.github.io/kani-verifier-blog/2023/08/03/turbocharging-rust-code-verification.html); this is historical context, so confirm the current pinned implementation.
- [Kani automatic harness generation](https://model-checking.github.io/kani/reference/experimental/autoharness.html).
- [Kani concrete playback](https://model-checking.github.io/kani/reference/experimental/concrete-playback.html).
