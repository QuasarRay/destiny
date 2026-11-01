# Partitionable C++ ↔ Rust/Bevy correspondence

**Source:** `src/Partitionable.h` (2015) and `src/Partitionable.cpp`, plus `src/SortedSets.h` and `src/Vector3d.h`.

**Rust:** `destiny-rs/src/partitionable.rs`.

The original is an **abstract base class**: 12 fields, one constructor and seven pure virtual methods. `Partitionable.cpp` supplies only the constructor; the seven behavior bodies exist only on concrete derived classes. No invented default behavior is provided in the Rust abstract contract.

## Type and object mapping

| C++ identity | Rust representation | Boundary |
|---|---|---|
| `typedef int64_t ID` | `type ID = i64` | Exact signed-width alias |
| `class Partitionable` | `#[derive(Component)] struct Partitionable` | Same 12 original fields, in same source order |
| `Vector3d mNewPos/mOldPos` | `DVec3 mNewPos/mOldPos` | Bevy double-precision vector |
| `long mNewBubble/mOldBubble/mEffectStamp` | `i32` | Windows/MSVC `long` width; not a Linux LP64 C++ layout match |
| `SortedSetOfBoxes mBoxes` | `SortedSetOfBoxes = BTreeMap<Key,Entity>` | C++ `std::set<Box*, BoxSortComparer>` replaced by key-indexed Bevy entity set |
| `Box* mMyBox` | `Option<Entity> mMyBox` | Nullable Bevy reference, not ownership of a Box |
| `Ballpark* mPark` | `Option<Entity> mPark` | Nullable Bevy reference; Ballpark not yet ported |
| Constructor `Partitionable()` | `Partitionable::new()` / `Default` | Original initializer values; Vector3d default zero via Bevy DVec3 zero |

`mBoxes` is the only authoritative membership map. The public original name is restored; existing `boxes()`, `track_box()`, `untrack_box()` and `forget_box_references()` are explicitly **Bevy integration adapters**, not additional source methods.

## Seven pure virtual methods

All methods use their exact original C++ names on `Partitionable`. The Rust `PartitionableBehavior` trait declares the same seven required operations, and the inherent `Partitionable` methods forward to the trait's concrete provider:

`ClearBoxes`, `DeleteFromBoxes`, `InsertInBox`, `GetBoundingRadius`, `GetInflatedRadius`, `SetBoxActive`, `InsertInBoxes`.

C++ vtable dispatch via a base-class pointer is represented by explicit Rust generic trait dispatch `Partitionable::Method::<T>(world, object, ...)`. `World` and `Entity` are the required Bevy storage/identity boundary. These signatures are **not binary-identical** to C++, because Rust ECS does not use `this`, C++ pointers, or C++ inheritance layouts.

An original pure virtual function is not implemented in `Partitionable.cpp`, so no concrete physics behavior can be considered equivalent until its subclass implementation is ported and verified.

## Verification

`python3 tools/run_partitionable_differential.py` extracts and compiles the **original C++ constructor initializer list** from `Partitionable.cpp`, with a test-only concrete subclass, and compares its output with Rust. This directly tests constructor defaults, not any pure virtual method.

`python3 -m unittest discover -s tools/tests -p 'test_*.py'` checks the source identities and counts of fields and abstract method declarations. `cargo test -p destiny-rs --test partitionable` tests field defaults, membership order and virtual-method forwarding.

**Status:** Header declarations and constructor values are mapped with documented Bevy/C++ ABI differences. Universal behavioral, object-layout and derived-method equivalence **is not established**.
