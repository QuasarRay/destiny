# Latest-source Destiny-rs migration

The authoritative Original Destiny revision is
`114e89fa584f0e1a81eb0a679aabf41af8870072`, the remote `main` tip checked on
16 October 2026. The user explicitly selected the newest original while
preserving existing Rust work. The visible Rust branch remains
`destiny-rs-dev`; its existing remote work was fast-forwarded from `f5bb8ed`
to `8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268` before the local changes.
Every previously tracked Rust file was byte-preserved during that update.
The initial `.agents`, `blue-rs/`, and implementation-prompt user work was
preserved. No local migration changes have been published.

The migration is incomplete. `source_inventory.json` retains all 531 older
fixture-qualified cases and the added partition case: 532 definitions,
458 Python and 74 C++. Twelve explicit original Windows/macOS presets remain
in the inventory and require semantic mapping to the standalone core targets.
These counts describe source tests; a complete behavioral-feature
denominator is still unresolved. The 78 C/C++ file extraction holes remain
visible across all 12 presets rather than being removed from scope.

The current milestone is the complete standalone world engine. Original Blue,
Blueexposure, exefile, legacy wheels and authenticated transport are later
integration work. World meanings expressed through those interfaces remain
required core behavior. `world_scope.json` records reviewed source regions,
assertions, linked feature IDs and classification audits. Its twenty-seven reviewed
fixtures comprise twenty-four WORLD_CORE, two HOST_ADAPTER and one MIXED; 505 remain
UNKNOWN. The mixed fixture produces a required core state-transfer obligation
and a deferred stream/hosting obligation. Missing Blue does not classify or
defer any unreviewed fixture.

## Implemented changes and provenance

- `compiler_geometry.json` contains actual Clang LibTooling identities,
  signatures, source lines, binary64 expressions and the isolated-header
  compiler configuration. This is not a full original-engine compilation.
- `geometry_semantics.json` is typed input for the restricted emitter. The
  generator checks the existing `Vec3::length`/`norm_squared` Rust syntax
  against the extracted C++ expression, then reuses those methods. Their
  implementation is unchanged.
- Genco emits `src/generated_geometry.rs` and the source-bound Kani harness.
  A `syn`/`quote` transformation connects three actual Bevy position-query
  sites. `rust_transformations.json` binds the preserved Rust base, input
  identities, generator, outputs and regeneration commands. Clean staging
  regenerates the rewritten files from immutable Rust inputs, so the drift
  check detects edits in the generated call paths.
- `python_adapter_rules.json` records five source-backed representation rules.
  The AST pass changes only selected expression spans: original tuple results
  for `GetBoxCenter` and Boolean-to-integer `isCloaked` input conversion.
  `python_transformations.json` records both source and output identities.
  Three composable rules admit and encode/decode the source RotationalAgility
  field. Full raw cloak-attribute, uint8 and Blue-conversion domains remain unresolved.
- `uncloak_rule.json` and `uncloak_source_evidence.json` bind the original
  proper-warp predicate and the existing specification guard. A `syn` pass
  transplants that guard around the existing native massive-state and
  sensor-membership mutations. It adapts field access through the current
  runtime mode lookup and ECS metadata; no bounded projection storage or
  duplicate state-transition implementation is introduced. The unchanged
  original warp-uncloak assertion now passes through 21 actual native calls.
- `reuse.json` records inspected Trinity settings, inspected blue-rs emission
  machinery, the directly reused Aegis supervisor, tool roles and source
  hashes. Trinity inputs/XML/semantics were not imported. The existing blue-rs
  parser skips missing/malformed XML, so it was not used as a completeness
  validator or added as a runtime dependency.
- `angular_source_evidence.json` and `angular_rules.json` bind original scalar
  fields, mutation phases and roll inputs. The `syn` pass transplants the
  unchanged reviewed angular AST, removes bounded projection storage, and
  binds friction, dt, current time and roll settings to actual runtime inputs.
  Original platform atan2 and expression grouping are preserved. Snapshots
  store exact f64 bits, including source-generated nonfinite values.
- The same generator connects formation motion to actual loaded offsets,
  source signed-char indices, the unchanged old-orientation rotation kernel,
  and existing goto thrust/integration. Both original moving/stopped ten-step
  traces and the original angle fixture now pass. Full dynamic orientation,
  source collision-history correspondence and valid-domain refinement remain
  open; these observations do not establish a complete controller.
- `speed_source_evidence.json` binds the original stop, time-factor and linear
  integration definitions. A recorded span transformation separates the source
  `DestinyMaxVelocity` parameter from Avian's hard speed limit, preserving one
  owner for getters/setters, snapshots, checkpoint restore and optional
  replication. It removes the non-source clamps and computes source collision
  outputs before damping preparation can zero an underflowed input velocity.
  Both original stop fixtures and 35 independent unchanged-C++ samples pass in
  both collision modes: 420 exact world position/velocity bit comparisons per
  build profile, including Taylor position and time-factor underflow.
- `warp_alignment_source_evidence.json` binds Original EvolveWarp/EvolveGoto.
  The generator extracts the reviewed negative-stamp guard and connects aligning
  warp to the existing source goto thrust. All ten original positions and
  per-step Warp-mode assertions pass directly in Rust. Proper warp activation,
  complete transition/callback behavior and required proofs remain open.
- `missile_source_evidence.json` binds source steering, launch, rotational
  fields and retained rotation. A recorded AST pass reuses the existing
  missile arithmetic with actual friction/dt/tick inputs. The production tick
  calls that kernel; existing admission, launch and follow-field kernels remain
  unchanged. Rotational agility uses one f32 owner and a versioned nonfinite
  bit codec; raw writes remain distinct from constructor normalization.
  Stationary launch and retained GetRotatedVector reuse scalar-angle rotation.
  Every generated Rust output identifies its source revision, preserved Rust
  base, generator and semantic-bundle digests and regeneration recipe.

## Maintainer commands

Ordinary consumers use the existing locked Cargo/CMake routes. They do not
need analysis tools. Regeneration uses Rust 1.95, the locked codegen crate,
Clang/LibTooling/clang-doc 23.1.1, and the source Git objects retained locally.
The installed clang-doc supports JSON rather than YAML; the recipe verifies
that capability before using it.

```sh
python tools/migrate_destiny.py inventory
python tools/migrate_destiny.py extract
python tools/migrate_destiny.py generate
python tools/migrate_destiny.py generate --check
python verification/tools/transform_python_adapters.py
python verification/tools/transform_python_adapters.py --check
cargo test --manifest-path verification/codegen/Cargo.toml --locked
cargo test --no-default-features --locked --offline
python verification/tools/update_migration.py observe-core
```

`observe-core` reuses the existing Aegis process supervisor. It retains current
debug engine tests, optimized angular, motion and missile tests and strict
engine/optional-integration/codegen lint results with source, generator,
fixture, command and tool identities. The reader checks actual named test
output and freshness. It grants runtime observations only, never proof or
release credit. The independent angular samples are produced by
`python verification/tools/check_angular_migration.py samples`: 136 unchanged
original C++ transitions and nine exact field bits per transition. Reuse the
retained sample corpus when its inputs are unchanged. The oracle's 64-bit
host `long` does not validate the original Windows long-width configuration.
The stop reference corpus is reproduced with
`python verification/tools/check_stop_migration.py`. Its unchanged EvolveStop,
SetBallTimeFactor and Integrate methods use the real Vector3d header and source
scalar widths. Expected values are C++ outputs, not Rust-generated expectations.
`tests/original_motion_contract.rs` runs the production Bevy scenarios without
Python/Blue and preserves the original stop/warp assertions and comparison rules.

`python verification/tools/check_missile_migration.py` runs unchanged source
missile/field/time-factor/integration slices, retaining 42 transitions and 15
raw/constructor f32 patterns in `missile_oracle_samples.json`. Eighteen launch
and rotation cases execute actual Carbon Math 1.2.2 and DirectXMath apr2025
SSE2 non-FMA code, pinned through the original registry recipes and checked
archive/source hashes. The archive's CRLF exports match the reviewed LF Git
blobs after only line-ending conversion; the exact compiled bytes are retained.
`tests/original_missile_contract.rs` preserves all four original launch,
twenty-step steering and collider scenarios in both collision settings. It also
checks 756 destination/position/velocity bit comparisons per profile, non-default
launch and attached/detached query bits, f32 snapshots, generation isolation and
twenty future replay ticks. The dynamic constructor test observes only that
already exposed state boundary: complete dynamic evolution and dynamic snapshot
restore remain unsupported. Admission/callback/target-loss, platform/domain
closure and mandatory proofs are still required.

The independent oracle is compiled from the unchanged original `Vector3d.h`.
Its adapter provides only the external `Vector3` declaration needed to parse
unused overloads; the selected original methods use double fields and `sqrt`.
The complete adapter, compilation database, tool receipts and shared library
are retained under `.aegis/destiny/migration/`. The tracked seven oracle
samples also support clean Kani harness regeneration without a mutable cache.
Full independent numeric comparison is reproducible with:

```sh
cargo build --all-features --lib --locked
python verification/tools/check_geometry_migration.py \
  --library target/debug/libdestiny_bevy_compat.so \
  --oracle .aegis/destiny/migration/liboriginal_geometry.so \
  --report .aegis/destiny/migration/geometry-delivery.json
```

That checks 263 finite admitted vectors, two radii and four query operations:
2,104 exact binary64 comparisons through the real native library. Missing-ID
and missing-ego outcomes are checked too. No universal/full-domain claim is
made. The original scaled-norm binary supplied a negative control with 452
incorrect center-distance comparisons. Its initial surface comparison fixture
had an argument-order mistake and receives no evidence credit; the corrected
fixture and complete diagnostics are retained separately.

## Documentation, architecture and queue

Doxygen 1.18.0 was downloaded into the project cache from its official release
and verified against the published SHA-256. `Doxyfile` adapts relevant Trinity
settings to pinned Destiny inputs, including private/static/local declarations.
With `DESTINY_ORIGINAL_ROOT` and `DESTINY_DOC_OUTPUT` set to the source cache and
artifact directory, run the pinned binary with that configuration, then:

```sh
python verification/tools/inspect_migration_docs.py
python verification/tools/update_migration.py cpg
python verification/tools/update_migration.py architecture
python verification/tools/update_migration.py architecture --check
python verification/tools/update_migration.py queue
python verification/tools/update_migration.py self-test
python verification/tools/update_migration.py acceptance --scope both
```

The XML inspection checks every original file against Git-bound hashes and
validates index, compound and Doxyfile XML against their respective emitted
XSDs. It retains 2,107 documentation records, all joined to original files,
plus 78 ambiguous identity joins. Documentation does not discharge compiler
or semantic extraction. `cpg_mapping.json` is explicitly a sidecar mapping,
not a conformant CPG; no Joern layers have executed or been validated.

Six architecture views under `docs/architecture/` and twelve decisions under
`docs/adr/` are generated from `architecture.json`. They are an alignment
profile for ISO/IEC/IEEE 42010:2022, not a conformance assessment. Stable
component, concern, view and decision IDs are checked for missing references.

There is one canonical work queue: `.aegis/destiny/work-queue.json`.
`update_migration.py` is its sole migration writer, reusing existing Aegis
readers where available and retaining blocked assurance rather than granting
cached credit. Acceptance refreshes source scope and the queue and retains
`core-acceptance.json`, `full-acceptance.json` and the full-result compatibility
path `acceptance.json` under `.aegis/destiny/migration/`. `--scope` selects the
reported milestone and exit status. Core-only routing preserves mandatory core
proofs and unknowns; it excludes only reviewed host obligations. Full acceptance
also requires all deferred adapters and original product release evidence.
Negative checks reject an omitted original case/core mechanic, stale source
digest, missing production witness, unknown architecture reference and a mixed
fixture deferred wholesale. A hypothetical discharged-core/unmet-host example
checks routing only, without claiming a complete real engine.
Both readiness results remain `INCOMPLETE`. Complete world-field/effect and
semantic-configuration closure validation, feature-to-production refinement,
robustness/performance and complete engine/product release receipt readers
remain undelivered. The checker explicitly rejects acceptance until they exist;
its current rejection result is not itself a completed acceptance audit.
The existing 531-case `check_case_acceptance.py` audits the historical subset
and cannot establish the complete newest-source denominator.

## Remaining implementation and assurance

The current stop/warp Python selection reports 285 assertion passes, 2 failures,
33 errors and 138 diagnosed missing-Blue blockers. Of the passes, 268 execute
native calls and 17 do not; those 17 are not native-execution evidence. The
74 original C++ cases and full supported configuration matrix remain open.
Existing ordinary Rust tests, native ABI smoke tests and the relocated Linux
consumer are separate release observations.

The generated Kani check retained all seven independent oracle assertions.
It failed one of 27 properties on the `1e-160` input; its cover property was
satisfied. A separately generated C check reproduced the discrepancy in
CBMC 6.11.0 while the native C executable passed. The retained CBMC trace
shows a nondeterministic nonnegative `sqrt` root constrained by rounded
`root * root == input`. For this subnormal intermediate that admits a result
different from the correctly rounded original/native result. No assertion,
tolerance, input domain or math stub was changed to manufacture a pass.

The existing Aegis formal preflight also blocks on uninitialized nested Kani
gitlink directories (first `charon`). Exact top-level verifier source pins
have been materialized; unrelated optional nested repositories were not
downloaded merely to bypass this controller limitation. The direct Kani
observation is unclassified and grants no production-refinement credit.
The current Verus scalar bridge excludes floating point. Neither compilation
nor native agreement supplies the missing formal refinement.

The complete original CMake configure fails at missing `carbon-blue` package
configuration. Its pinned registry/dependency declarations, all original
sources and the twelve presets remain in the inventory. Source-compatible
complete formation/angular and stop/warp variants, missile steering, partition APIs, callbacks,
private Blue/stream behavior, supported platforms, complete notices and actual
authenticated transport still require work at their respective milestones.
Historical pre-angular packaging evidence: a local Linux wheel was installed
outside the source tree and passed the 29 existing native differential tests
with no skips after wheel-adjacent library discovery. The source-distribution
policy explicitly includes the required Rust path dependency, anchors includes
at the project root, and excludes local caches, controller/reference checkouts
and the user's private implementation prompt. These are local packaging
observations; they do not close the full release matrix.
CMake also watches the path dependency's sources and manifests when deciding
whether to invoke Cargo. `tools/generate_artifact_manifest.py --archive PATH`
hashes the actual source distribution; `--check` rejects manifest drift.
`CONTINUATION.md` records the next dependency-ordered batch and exact evidence
locations. The active Goal controls continued core work; no independent daemon,
reminder automation, or new chat was created.

Source master liveness now uses generated typed keep-alive/interactivity rules in the existing tracker, a derived per-tick ECS cache, and the actual collision/integration/notification schedule. `liveness_oracle_samples.json` retains 70 unchanged C++/real-CPython membership states and 3,640 mode/bubble query pairs. The production owner and real headless runtime are tested separately. Full collision-force history, pending-work snapshots, reentrancy and mandatory proofs remain open. Regenerate the oracle with `python verification/tools/check_partition_migration.py`; run the headless checks with `cargo test --no-default-features --locked --offline --test original_liveness_contract`.

The two existing collision production-contract modules are also reused directly in the engine library tests, preserving four concrete fixture/large-collection assertions. A recorded AST pass supplies their source client liveness input. This does not run Kani: whole Kani crate imports still need generated Tracking dependency/module closure and older partition fixture API adaptation. The diagnostic and all failed/corrected receipts remain under `.aegis/destiny/migration/`; mandatory core proof gates stay open.

Generated `Forces` now stores Original mLastG/mLastC in each existing ball metadata component. Source live/free admission resets them; dead/global collision candidates retain them, and both actual source response paths publish their collision-force changes. Full logical snapshots preserve six binary64 bit patterns. Three independent unchanged-C++ GotoThrust/Potential scenarios (including an oblique 3D direction) and time-zero overlap-force fragments feed the headless tests in `tests/original_force_contract.rs`. Complete collision locations/last-collision/contact/callback histories and mandatory proofs remain open.

`contact_rules.json` and the existing typed state IR now generate ordered source
contact ownership, reset/retention, snapshot mapping and source warp-angle
admission. `verification/codegen/src/contacts.rs` transports outputs from the
retained collision kernels without copying their algorithms. The same stop
checker additionally records source Potential IDs, 130 sorted spherical
contacts and both aligning-warp contact states through the existing angular
oracle. Run `cargo test --no-default-features --locked --offline --test
original_contact_contract` for six actual headless witnesses. Full logical
snapshots preserve duplicates and all signed 64-bit IDs; absent older fields
default empty. Iterative location IDs stay separate from `mCollisions` before
the late missile phase. Per-body moved-missile dispatch, reentrant payload
reads, complete locations/history/pending-work state and proofs remain open.

The generated direct source dispatcher now uses an operation-owned
`DestinyCollisionPass` and typed `DestinyCollisionNotice` values. Use
`dispatch_source_collisions(id, observer)` for a synchronous Rust host, or
`begin_source_collision_pass` / `next_source_collision` to integrate with the
existing yielding callback continuation. Capture count happens once; each next
notice reads the actual current ID and LastC. Returning false for a contact
produces an immediate `HostError` notice and then continues. Nested dispatch
uses its own captured count. World/Entity/generation binding prevents cross-world
or replaced-owner cursor reuse. Source list shrink below the captured size is
unsafe in C++; Rust reports an explicit error after retaining prior effects.

`collision_dispatch_oracle_samples.json` records six executions of unchanged
Ball.cpp:468-488 through an explicit normally returning host boundary: 319
contacts and three error notices, including same-owner recursive dispatch,
appends, later-ID replacement, force changes and nonfinite bits. Actual PyOS
unwinding/private semantics remain unresolved. The direct dispatcher consumes
real gradient output in the headless tests. Automatic moved-missile per-body
publication and FFI scheduling remain unwired and must reuse this cursor.
