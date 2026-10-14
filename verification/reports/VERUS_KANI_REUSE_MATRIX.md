# Verus/Kani reuse matrix

Pins: Verus aef82eda71838deef4a8cd0260fda6c9504be9b9 and Kani
0d2328a93f0e0ff66132d6bfa1a7d884877cf862.

| Capability | Pinned upstream file/symbol | Dependencies and semantic coupling | Existing local equivalent | Integration decision | Required adaptation | Validation | Remaining limitations |
|---|---|---|---|---|---|---|---|
| Cargo discovery | Kani kani-driver/src/project.rs | Cargo plus Kani compiler metadata | Aegis process/evidence control only | Reuse through Kani metadata-only subprocess | Parse kani-list.json 0.1 | fixture discovery and version/schema check | pinned Kani toolchain required |
| Entry selection | Kani args/autoharness_args.rs | crate-qualified regex selection | Destiny proof-lane exact selection | Reuse Kani selection, then only narrow it | map selected pretty name to parsed function | fixture smoke | simple free functions only |
| Automatic harness generation | Kani AutomaticHarnessPass / AutomaticArbitraryPass | rustc MIR and Kani intrinsics | none | Reuse compiler eligibility/generation analysis as metadata; do not transplant MIR | generate Verus obligation from accepted source AST | valid/invalid live fixtures | not general Kani harness equivalence |
| Arbitrary inputs | Kani automatic pass and Arbitrary models | Kani MIR/model semantics | Verus universal proof inputs | adapt only independent scalars | preserve each parameter as a distinct Verus parameter | independent-input regression | ADTs, refs, pointers and collections unsupported |
| Contracts and stubs | Kani contract/stubbing transforms | Kani-specific assumptions/instrumentation | existing Destiny proof lanes | do not migrate yet | reject capability | registry | no contract automation in this slice |
| Compiler instrumentation | Kani kani_middle/transform | backend/rustc coupled | Verus rust_verify pipeline | backend-specific | structural parser only for declared subset | rejection tests | translation remains a trusted boundary |
| Verus Cargo/source integration | Verus cargo-verus metadata/plan and rust_verify cargo_verus | Verus rustc-private toolchain | existing direct Verus campaign | retain supported executable boundary | generated one-entry source | strict counted output | separate from concurrency campaign |
| Scheduling | Kani harness runner and Verus cargo plan | backend-specific | Aegis supervisor/budgets | do not import another scheduler | sequential one-function verification | bounded CI timeout | no parallelism yet |
| Result handling | Verus counted summary | exact count | Aegis destiny_reuse.checked_output | reuse existing strict parser | expect exactly one generated Verus entry | live valid fixture | richer inventory pending |
| Playback | Kani concrete_playback/test_generator.rs | concrete CBMC trace semantics | no equivalent for Verus proof state | backend-specific; do not fake | report non-replayable proof failure | invalid fixture | no Verus witness decoder yet |
| Test infrastructure | upstream regression patterns | backend messages differ | Destiny formal CI | small representative corpus | valid/invalid/unsupported crates | unit plus live CI | corpus intentionally small |
| CI packaging | pinned Kani/Verus toolchains | separate rustc-private versions | existing formal workflow | preserve process boundary | immutable gitlinks and build recipe | source-build plus smoke jobs | no whole-project parity claim |

## Boundary decisions

Kani automatic-harness generation is compiler/MIR coupled, while symbolic
execution in Kani's default verification path is performed by CBMC. This
integration does not relabel Kani/CBMC as a portable symbolic executor. It
reuses Kani's actual compiler-derived discovery/eligibility path and then checks
a separately generated, source-bound Verus obligation.

Ordinary Rust assert! is runtime behavior in Verus, not a Verus proof command.
The supported parser therefore lowers accepted assertions explicitly.
Unsupported syntax fails closed.

The existing Kani corpus and counted Verus concurrency campaign remain
independent evidence. No denominator, historical receipt, proof marker, or
residual correspondence obligation is changed by this slice.
