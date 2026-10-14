#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OFFLINE=()
if [[ "${1:-}" == "--offline" ]]; then
  OFFLINE=(--offline)
elif [[ $# -ne 0 ]]; then
  echo "usage: $0 [--offline]" >&2
  exit 2
fi

python3 "$ROOT/verification/tools/vendor_verifiers.py" --verify-only

# This script intentionally does not install toolchains or solvers. Provision
# exactly the versions in verification/vendor/manifest.json first.
cargo +nightly-2026-08-21 build \
  --manifest-path "$ROOT/third_party/kani/Cargo.toml" \
  --locked --release -p kani-driver -p kani-compiler "${OFFLINE[@]}"

cargo +1.98.1 build \
  --manifest-path "$ROOT/third_party/verus/source/Cargo.toml" \
  --locked --release "${OFFLINE[@]}"

echo '{"status":"VENDORED_VERIFIER_BUILDS_COMPLETED","proof_credit":false}'
