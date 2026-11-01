# Destiny Partitionable (Bevy port)

Ports **only** the shared state and abstract interface in CCP's `Partitionable.h` / `Partitionable.cpp`.

## Native Bevy choices

- Positions use Bevy `DVec3` (no `Vector3d` recreation).
- Each partitionable is a Bevy `Component`, and associated boxes / owning park use `Entity` references, never raw pointers or process-global state.
- `BoxKey` mirrors the **actual key ordering** in C++ `Partition.h` / `BoxSortComparer` (ix, iy, iz, n); `BTreeMap<BoxKey, Entity>` preserves `std::set` uniqueness and deterministic iteration, unlike ordering by Entity ID.
- Signed C++ `long` fields are `i32` for MSVC/Windows ABI; `ID` is `i64`.

## Abstract methods

The seven pure virtual methods remain required methods of `PartitionableBehavior`. They cannot be correctly implemented using only the supplied base-class files: C++ `Ball` and `StaticCollidable` provide distinct implementations, and `InsertInBoxes` depends on the spatial partition hierarchy.

`forget_box_references()` mirrors only the common *local* portion of C++ `ClearBoxes()`. It **does not** remove entities from Box-owned membership sets. Implement the actual `clear_boxes`, `delete_from_boxes`, `insert_in_box`, `get_bounding_radius`, `get_inflated_radius`, `set_box_active`, and `insert_in_boxes` operations when porting those concrete types and the Box/Partition ECS implementation.

Unlike `SettingsConfiguration`, the original `Partitionable` class is **not** directly exposed via Blue/Python; consequently, this crate does not fabricate Python bindings or a second object model. It can coexist with `destiny-settings` in the same Blue-rs/Bevy workspace.

## Validation

```sh
cd blue-rs
cargo test -p destiny-partitionable
cargo check --workspace --all-targets
cargo test --workspace
```
