# Rust port status

The Rust workspace is an additive implementation. It does not replace or delete the original C++/Python Blue or Blueexposure sources; those remain the compatibility oracle.

## Implemented

- Bevy 0.20.0 as the Blue runtime kernel (`App`, `World`, schedules, resources, components, assets, reflection).
- PyO3 0.30.1 Python exposure and `_bluepycore` extension entry point.
- Opinionated `BluePlugins` ordering for headless Blue runtimes.
- Blue lifecycle, startup arguments, path/search-path semantics, object metadata, resource asset/index storage, and clock state.
- Blueexposure class/function/enum registration metadata backed by Bevy reflection.
- `BlueScriptCallback` for Python callable storage/invocation.
- `#[blue_pyclass]`, `#[blue_methods]`, `#[blue_function]`, `#[blue_enum]`, `#[derive(BlueClass)]`, and `#[blue_test]` metaprogramming.
- Automatic PyO3 `None` defaults for trailing `Option<T>` arguments in Blue-generated methods/functions.
- Doxygen XML and source test discovery.
- Genco-generated legacy symbol/test catalogs and Genco-generated Kani integration harnesses.
- Generated Kani proof twin for every current native `#[blue_test]`.

## Migration evidence

- Legacy declarations discovered from source: 556.
- Unique legacy test identities after deduplication: 553.
- Current native Rust semantic tests: 11.
- Current generated Kani harnesses: 11.
- Current explicit legacy-to-Rust semantic mappings: 2.
- Dangling mappings: 0.

Three duplicate test identities exist in the original Python suite; the migration inventory deduplicates those identities rather than pretending they are distinct contracts.

## Deliberately incomplete

The legacy API surface is much larger than the implemented Rust behavior. Streams, marshal/black persistence, full resource-manager behavior, scheduler/tasklet compatibility, weak/reference semantics, structure/list/dict wrappers, statistics, logging, OS/network helpers, and many Blueexposure compatibility details still require semantic ports and legacy-test mappings.

The generator does not emit placeholder implementations or trivially passing Kani proofs for those missing contracts. Unmapped tests remain visible migration gaps.
