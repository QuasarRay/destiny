# V4 release gates

The source is a v4 integration candidate. Production release requires every
gate below on the exact delivered archive.

## 1. Locked Rust quality gate

```bash
cargo fmt --all -- --check
cargo check --all-features --all-targets --locked
cargo test --all-features --locked
cargo clippy --all-features --all-targets --locked -- -D warnings
```

Acceptance: no warnings/errors; native network, snapshot, rollback, collision,
room, recipient-redaction, and bubble-membership tests all run.

## 2. Linked ABI and relocated CMake gate

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DDBC_CARGO_LOCKED=ON
cmake --build build --config Release --parallel 2
ctest --test-dir build -C Release --output-on-failure
cmake --install build --config Release --prefix "$PWD/stage"
cmake -S tests/consumer -B consumer-build -DCMAKE_PREFIX_PATH="$PWD/stage"
cmake --build consumer-build --config Release
ctest --test-dir consumer-build -C Release --output-on-failure
```

Run on Linux, macOS, and Windows. Acceptance includes executable C11/C++17
link tests and the legacy `share/carbon-destiny` package path.

## 3. Wheel and differential gate

```bash
python -m build
python -m venv clean-venv
clean-venv/bin/python -m pip install --no-index dist/*.whl
DESTINY_BEVY_COMPAT_LIBRARY=/absolute/path/to/built/library \
  python -m unittest discover -s tests -p 'test_backend_differential.py' -v
```

Use `clean-venv/Scripts/python` on Windows. Acceptance: import succeeds without
the source tree, native discovery resolves only the wheel-adjacent library, and
the native/contract motion, query, membership, geometry, and capture scenario
matches.

## 4. Supply-chain and notice gate

```bash
cargo audit --file Cargo.lock
cargo deny check
```

Acceptance: no unreviewed advisory, yanked dependency, wildcard dependency,
unknown source, or disallowed license. Generate/review the complete transitive
license inventory and ship every required notice with the binary/wheel.

## 5. Real Carbon deployment gate

Exercise at least two authenticated clients and one server using the product's
actual Lightyear transport:

- singlecast reaches only its bound link;
- narrowcast frames reveal no other recipient IDs;
- duplicate/unknown Carbon identities fail closed with no partial send;
- global, bubble, owner-cloaked, pending-removal, and disconnect visibility is
  correct;
- inbox duplication, overflow, rebase, reconnect, and full-state recovery work;
- sender/link buffers remain within configured process budgets under loss and
  a slow/nonreading client;
- credential, authorization, encryption, replay, and revocation policies hold.

## 6. Archive integrity gate

Regenerate `ARTIFACT_MANIFEST.md`, verify every listed hash, scan the archive
for caches/build output/secrets, and record the outer ZIP SHA-256 in the release
record.
