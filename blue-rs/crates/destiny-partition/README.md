# Destiny Partition — Bevy ECS port

Source-derived port of `Partition.cpp`, `Partition.h`, and `Partition_Blue.cpp` (CCP ©2014/2023), with Box hierarchy semantics inspected from the repository.

Implemented: 8-level CCP grid, 480m fine unit, exact grid powers, position/key conversion (C++ truncation rather than floor), key clamping, radius-based level selection, box creation/lookup/offsets, coarser parent hierarchy, topmost, active box export, traversal bubble marking, neighboring bubble lookup, hierarchy cleanup after removal, C++-ordered ball/static membership indexes, nearby/proximity/collision candidate filters, and the Blue-exposed `destiny.GetBoxCenter(level,x,y,z)` Python function.

`Partition` is a Bevy Resource; `PartitionBox`, `PartitionBall`, `PartitionStatic`, and existing `Partitionable` are Bevy Components. No unsafe pointers, process-global vectors, custom allocator, or duplicate ECS.

## Intentional integration boundary

The C++ Partition depends on concrete `Ball`, `Box`, `Ballpark`, and `StaticCollidable` behavior, which is not completely ported. `PartitionBall` and `PartitionStatic` hold **only the fields required by CCP's candidate selection** and will be replaced/adapted when those full entities are ported. `Ballpark::GetInteractiveCnt` is passed as an explicit input to `proximity_candidates` rather than invented. Ball's 27-slot active-box flags and its radius/motion integration are not supplied here.

C++ uses unordered-map enumeration for `GetActiveBoxes`, so its ordering is unspecified. Bevy port uses sorted `BoxKey` ordering deterministically. For Python coordinates, values are cast to f32 and back as in C++ `Py_BuildValue("f")`.

Subsequent exact-behavior tests should compare C++ fixtures for candidate ordering, pruning, and extreme coordinates. Note that source C++ has undefined behavior on oversized out-of-range `double` to `int64_t` conversions; Rust casts saturate instead.

```bash
cd blue-rs
cargo check --workspace --all-targets
cargo test -p destiny-partition
cargo test --workspace
```
