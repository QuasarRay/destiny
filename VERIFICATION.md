# Verification record for v4

This file records checks against the exact v4 source tree. It does not reuse a
v3 native checkpoint as proof and does not treat CI configuration or source
inspection as execution.

## Input provenance

- Supplied project ZIP SHA-256:
  `7dd869b77a2c30483dc468a2ca0258ed316a93ce4afe87815ce3d0afbc0a91b6`.
- Supplied audit ZIP SHA-256:
  `ab67a7d3c3da2d5b62bb85a664bb13d64128ab3e4acd0f02aa1217f09ed6e143`.
- The supplied audit consists of eight recommendation-level documents and
  contains no concrete defect identifier, source location, severity, or
  reproducer. `V4_REMEDIATION_STATUS.md` maps all eight recommendations and
  separately records the 64 concrete defects found during the v4 review.

## Executed in this workspace on 2026-08-25

- `PYTHONPATH=python python3 -m unittest discover -s tests -p 'test_*.py' -v`:
  83 tests discovered; 82 passed and the optional native differential test was
  skipped because no built native library exists in this workspace.
- The suite exercised snapshot round trips and malformed mutations, Carbon
  recursive values and retry semantics, state/history bounds, geometry,
  facade/backend distrust, lifecycle behavior, C/C++ guards, registry drift,
  dependency pins, and validation-contract parity.
- A separate deterministic geometry run passed 5,000 cone scaling cases,
  including zero/subnormal angles, and 1,000 triangle vertex-permutation cases
  across logarithmically distributed finite magnitudes.
- Python `compileall` passed for `python`, `tests`, and `tools`. Independent
  `-Wall -Wextra -Werror -pedantic -fsyntax-only` checks passed for the public
  C11 smoke source, C++17 facade smoke source, and C++17 relocated consumer.
  The suite also compiled and ran the standalone C++ JSON/buffer guard program.
- All four shipped JSON files, five TOML files (including `Cargo.lock`), and two
  YAML files parsed successfully.
- `python3 tools/generate_implementation_status.py` was run twice. The second
  generation was byte-identical for all four generated artifacts and retained
  all 389 title rows: 2 conformant, 92 accepted-difference, 2 state-only, 209
  facade-only, and 84 unsupported.
- All eight supplied-audit recommendation IDs and all 64 v4 finding IDs are
  unique, contiguous, and present exactly once in the v4 ledger.
- Cargo manifest/lock versions were compared programmatically. All 11 direct
  dependencies are exact pins and each requested version is present in the
  lockfile; the root package version is `0.4.0` in both files.
- A lexical delimiter/comment/string scan passed over all six Rust source
  files. This is source-level evidence only, not a substitute for Rust parsing,
  formatting, compilation, clippy, or test execution.

## Not executable in this workspace

`cargo`, `rustc`, `rustfmt`, `cmake`, `cargo-audit`, and `cargo-deny` were not
available. Therefore none of the following is claimed to pass here:

- Rust format/check/test/clippy for the exact v4 tree;
- native collision, network, room, capture, lifecycle, or rollback tests;
- linked C/C++ ABI executables or the native backend differential test;
- CMake build/install/relocated linking;
- wheel/sdist build and clean-environment native import;
- RustSec, license, source, or transitive-notice review;
- two-process authenticated Lightyear delivery.

`.github/workflows/ci.yml` defines these gates across Linux, macOS, and Windows,
but a workflow file is not an execution result. See `RELEASE_GATES.md`.

## Packaging completion record

`ARTIFACT_MANIFEST.md` is regenerated only after the last source/document edit.
Its independent coverage/hash check, the clean archive listing, and the outer
ZIP SHA-256 are performed after this file is finalized; the outer digest is
reported with the delivered archive because a ZIP cannot contain its own hash.
