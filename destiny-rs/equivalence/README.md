# Strict Destiny C++ ↔ Rust equivalence contract

**Status: NOT 100%**. This directory records source-to-Rust correspondence
without representing partial ports as complete.

The requested standard has **four distinct obligations**:

1. **Name equivalence:** each original C++ type, field and method must have
   a documented corresponding Rust item with its exact spelling wherever Rust
   permits it. Constructors, destructors, C++ overloads, `operator==`,
   templates, macros and built-in conversions require explicitly documented
   semantic exceptions, because Rust cannot provide literally identical syntax.
2. **Structural equivalence:** one-to-one owner/type/method boundaries, not
   just facade aliases. Bevy components/resources must be audited for any
   C++ object that has been split or merged.
3. **Behavioral equivalence:** equivalent results and side effects under
   documented observable states, including edge cases, errors, ordering,
   Python behavior, timing and concurrency. Differential tests between
   executable C++ and Rust must cover each relevant method.
4. **Internal equivalence:** each internal helper, invariant, state transition,
   branch and data-structure role must be matched and justified down to the
   explicitly documented Bevy boundary. An algorithm changed by Bevy must have
   a simulation relation proving it refines the original.

## Baseline

`mappings.json` registers only classes that have partial Rust counterparts.
The audit automatically enumerates all C++ .h/.cpp files and declared types
in `src/`, checking exact Rust type and method *names* in mapped files. It
lists all source files and declarations not represented by registered mappings.
Its method parser is **lexical** and may miss or overcount macro-expanded
symbols. It is *not* a C++ AST, Rust HIR or semantic equivalence proof.

Run from the repository root:

```sh
python3 tools/destiny_equivalence_audit.py
python3 tools/destiny_equivalence_audit.py --strict
```

The strict command is expected to fail until the original requirements are
met and independently validated. The default command is diagnostic and
retains a green compilation CI even when equivalence remains unproven.

## Current known divergences

- Original `Box` has a `PyObject* mObject` member, while Rust has a separate
  `BoxObject` component. The original `Box` methods have been assigned to
  `Partition` in Rust.
- Original `Partitionable` declares virtual methods on the class; Rust
  currently has a separate `PartitionableBehavior` trait with altered names
  and signatures.
- `BoxAllocator`, substantial Ball/Ballpark/collision internals, original
  globally shared Settings, and a complete C++/Rust differential oracle are
  not present.
- Rust `Box` and `Key` now have authoritative original type names; their
  earlier Rust-only aliases remain temporarily for callers and tests.

An **all-green Cargo run** means Rust compiles and its tests pass, not that
any of these proof obligations have been discharged.

To ultimately gate equivalence, replace this lexical inventory with a
Doxygen/Clang AST + Rust HIR catalog, map source method and field identities
(including overload signatures), and attach executable differential fixtures
and verified invariants to every mapping. Do not change any `verified_*`
flag without documented, reproducible evidence.

## Original tests

The audit also enumerates Python `test_*.py` modules under
`python/destiny/test/` and the original C++ `Test*.cpp` binaries under
`tests/`. Add a `test_mappings` entry for each with both source and Rust test
file, plus differential evidence, when the harness is actually ported. An
unmapped original test is a strict-gate blocker, not a passing test.

The original test corpus is not replaced by the existing Rust unit tests
merely because both suites have successful exit codes.
### Consolidated Box Python state

The canonical `Box` component now owns `mObject: Option<Py<PyAny>>`
directly rather than assigning the original C++ `Box::mObject` member to a
second `BoxObject` ECS component. This removes a known structural split.
Box is no longer unconditionally `Clone`: cloning a Python-owned reference
requires explicit Python attachment and is not an implicit Box operation.
