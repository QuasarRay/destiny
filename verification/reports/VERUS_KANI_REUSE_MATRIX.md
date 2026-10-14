# Verus/Kani automation reuse matrix

Pinned inputs: Verus aef82eda71838deef4a8cd0260fda6c9504be9b9; Kani 0d2328a93f0e0ff66132d6bfa1a7d884877cf862.

| Capability | Pinned upstream file/symbol | Coupling | Existing Destiny equivalent | Decision | Validation | Remaining limitation |
| --- | --- | --- | --- | --- | --- | --- |
| Cargo discovery | Kani kani-driver/src/call_cargo.rs cargo_metadata/cargo_build | Cargo + Kani toolchain | Existing manifests/controller binding | Reuse through process boundary | autoharness list compiles selected manifest; hashes retained | Different rustc-private toolchains |
| Entry selection | Kani autoharness module + AutoHarnessMetadata | MIR/type eligibility + Arbitrary | Hand-maintained proof plan | Directly reuse chosen/skipped metadata | Exact include regex; missing/skipped block | Initial subset only |
| Autoharness generation | Kani compiler autoharness path | Kani instrumentation | Existing Kani corpus | Reuse metadata-generation path, not CBMC result | list mode implies codegen-only | Generated harness is not translated to Verus |
| Arbitrary inputs | Kani Arbitrary generation | Kani symbolic constructors | Existing Kani obligations | Eligibility signal only | Candidate must be chosen by Kani | Verus quantification comes from reviewed entry |
| Contracts/stubs | Kani contract instrumentation | Backend/compiler specific | Existing Verus contracts | Backend-specific | Unsupported until reviewed | No general lowering |
| Compiler instrumentation | Kani compiler passes | rustc-private/backend semantics | None shared | Keep backend-specific | Process boundary | No transformation-soundness theorem |
| Verus execution | Verus cargo-verus planning + verifier | Verus/vstd toolchain | destinyctl campaign | Reuse existing controller | Existing binding/inventories/receipts/budgets | Bridge maps to whole reviewed entries |
| Scheduling | Kani harness runner / Verus campaign | Backend specific | Existing controller | Reuse controller | No duplicate Verus scheduler | Kani reference remains explicit |
| Result handling | Kani metadata + controller receipts | Different result semantics | Existing status model | Keep independent | Empty/blocked discovery cannot succeed | No cross-engine equivalence assumption |
| Playback | Kani concrete playback | CBMC trace specific | No Verus decoder | Backend-specific, unsupported | replay reports unsupported | Needs source-validated decoder/reproducer |
| Tests | Kani metadata fixtures + existing proof gate | Mostly reusable | Python/Rust CI | Reuse focused parser tests | chosen/skipped/missing regressions | No source replay fixture yet |
| CI packaging | Upstream release/build conventions | Toolchain/downloads | Existing formal workflow | Extend existing campaign | Discovery is checked before single Verus run | Offline source build still pending |

## Initial semantic bridge

The first supported subset accepts only executable-specification functions whose operations are Boolean logic, integer equality, or direct value selection and for which an already reviewed Verus entry exists. Floating point, machine arithmetic, casts, pointers, allocation, unsafe code, loops, recursion, and concurrency are not translated by this slice.

Kani owns eligibility discovery; Verus owns the checked proof. Kani's compiler stores autoharness metadata in crate-relative form (for example `model::follow_allowed`), while include filters are crate-qualified; the registry records both identities explicitly. The bridge uses the pinned Kani autoharness list path because it implies codegen-only, then reads compiler-emitted chosen/skipped metadata. A selected function maps to an existing Destiny controller obligation, and the existing controller checks the exact reviewed Verus inventory.

This does not prove general Rust-to-Verus translation correctness. The source-to-existing-Verus mapping remains a reviewed correspondence boundary, and arithmetic/floating-point cases whose semantics differ are explicitly unsupported.
