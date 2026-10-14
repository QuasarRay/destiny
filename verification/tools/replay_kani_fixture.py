#!/usr/bin/env python3
"""Explicit Kani/CBMC reference replay for a source-level failing fixture.

This is intentionally separate from the Verus-default lane. A temporary Kani
proof harness imports the original Rust source file by #[path], obtains a
concrete counterexample from pinned Kani, and then executes Kani's generated
playback test. The original source must remain byte-for-byte unchanged and the
runtime panic must point back to its assertion line.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
KANI_COMMIT = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862"
KANI_VERSION = "0.68.0"

SUBSET_PATH = ROOT / "verification/tools/verus_subset.py"
SPEC = importlib.util.spec_from_file_location("destiny_verus_subset_for_replay", SUBSET_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load verus_subset.py")
subset = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = subset
SPEC.loader.exec_module(subset)

PLAYBACK_RE = re.compile(
    r"Concrete playback unit test for .*?:\s*```\s*(.*?)\s*```",
    re.S,
)
TEST_NAME_RE = re.compile(r"\bfn\s+(kani_concrete_playback_[A-Za-z0-9_]+)\s*\(")


class ReplayError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def run(command: list[str], cwd: Path, timeout: int):
    return subprocess.run(
        command,
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )


def rust_string(value: str) -> str:
    # JSON strings use the same escapes needed for ordinary Rust string literals
    # for the path characters accepted by this CI workflow.
    return json.dumps(value)


def build_harness_source(source: Path, function) -> str:
    params = [
        f"    let {name}: {typ} = kani::any();"
        for name, typ in function.params
    ]
    args = ", ".join(name for name, _ in function.params)
    return "\n".join([
        "#![allow(dead_code)]",
        f"#[path = {rust_string(str(source))}]",
        "mod original_source;",
        "",
        "#[kani::proof]",
        "fn replay_harness() {",
        *params,
        f"    original_source::{function.name}({args});",
        "}",
        "",
    ])


def replay(
    source: Path,
    function_name: str,
    cargo_kani: Path,
    output_root: Path,
    timeout: int,
) -> dict:
    source = source.resolve(strict=True)
    cargo_kani = cargo_kani.resolve(strict=True)
    source_before = sha256(source)

    functions = subset.parse_source(source)
    matches = [function for function in functions if function.name == function_name]
    if len(matches) != 1:
        raise ReplayError(f"function {function_name!r} is not uniquely supported")
    function = matches[0]
    assert_lines = subset.collect_assert_lines(function.body)
    if len(assert_lines) != 1:
        raise ReplayError(
            "initial source replay requires exactly one source assert!; "
            f"found {len(assert_lines)}"
        )

    identity = {
        "schema": 1,
        "engine": "kani_cbmc_reference",
        "kani_version": KANI_VERSION,
        "kani_commit": KANI_COMMIT,
        "explicit_reference_backend": True,
        "verus_proof_credit": False,
        "source": str(source),
        "source_sha256": source_before,
        "function": function.name,
        "assert_line": assert_lines[0],
        "parameter_domains": [
            {"name": name, "rust_type": typ, "independent": True}
            for name, typ in function.params
        ],
        "cargo_kani_sha256": sha256(cargo_kani),
    }
    key = hashlib.sha256(
        json.dumps(identity, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    destination = output_root / key
    src_dir = destination / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    manifest = destination / "Cargo.toml"
    harness_source = src_dir / "lib.rs"

    manifest.write_text(
        "[package]\n"
        f'name = "destiny-kani-replay-{key[:12]}"\n'
        'version = "0.0.0"\n'
        'edition = "2024"\n'
        "publish = false\n\n"
        "[lib]\n"
        'path = "src/lib.rs"\n',
        encoding="utf-8",
    )
    harness_source.write_text(build_harness_source(source, function), encoding="utf-8")
    harness_before = sha256(harness_source)

    verify_command = [
        str(cargo_kani),
        "-Z", "concrete-playback",
        "--concrete-playback=print",
        "--manifest-path", str(manifest),
        "--harness", "replay_harness",
        "--output-format=regular",
        "--jobs=1",
    ]
    try:
        verification = run(verify_command, destination, timeout)
    except subprocess.TimeoutExpired as error:
        raise ReplayError(f"Kani reference verification timed out: {error}") from error

    (destination / "verify.stdout.txt").write_text(verification.stdout, encoding="utf-8")
    (destination / "verify.stderr.txt").write_text(verification.stderr, encoding="utf-8")
    combined = verification.stdout + "\n" + verification.stderr
    if verification.returncode == 0:
        raise ReplayError("expected failing reference harness unexpectedly verified")

    match = PLAYBACK_RE.search(combined)
    if match is None:
        raise ReplayError("Kani did not emit a concrete playback unit test")
    concrete_test = match.group(1).strip() + "\n"
    test_name_match = TEST_NAME_RE.search(concrete_test)
    if test_name_match is None:
        raise ReplayError("concrete playback output does not contain a test function")
    test_name = test_name_match.group(1)
    (destination / "concrete-test.rs").write_text(concrete_test, encoding="utf-8")

    with harness_source.open("a", encoding="utf-8") as stream:
        stream.write("\n")
        stream.write(concrete_test)
    replay_input_sha = sha256(harness_source)

    playback_command = [
        str(cargo_kani),
        "playback",
        "-Z", "concrete-playback",
        "--manifest-path", str(manifest),
        "--",
        test_name,
        "--exact",
        "--nocapture",
    ]
    try:
        playback = run(playback_command, destination, timeout)
    except subprocess.TimeoutExpired as error:
        raise ReplayError(f"Kani concrete playback timed out: {error}") from error

    (destination / "playback.stdout.txt").write_text(playback.stdout, encoding="utf-8")
    (destination / "playback.stderr.txt").write_text(playback.stderr, encoding="utf-8")
    playback_text = playback.stdout + "\n" + playback.stderr
    line = assert_lines[0]
    location_needles = [
        f"{source}:{line}:",
        f"{source.relative_to(ROOT)}:{line}:" if source.is_relative_to(ROOT) else "",
    ]
    source_location_reproduced = any(
        needle and needle in playback_text for needle in location_needles
    )
    source_after = sha256(source)
    harness_after = sha256(harness_source)

    status = (
        "reproduced_source_failure"
        if playback.returncode != 0
        and source_location_reproduced
        and source_after == source_before
        and harness_after == replay_input_sha
        else "non_replayable_reference_failure"
    )
    row = {
        **identity,
        "status": status,
        "key": key,
        "harness_source_before_playback_sha256": harness_before,
        "replay_input_sha256": replay_input_sha,
        "verify_command": verify_command,
        "verify_returncode": verification.returncode,
        "verify_stdout_sha256": hashlib.sha256(verification.stdout.encode()).hexdigest(),
        "verify_stderr_sha256": hashlib.sha256(verification.stderr.encode()).hexdigest(),
        "concrete_test_name": test_name,
        "concrete_test_sha256": hashlib.sha256(concrete_test.encode()).hexdigest(),
        "playback_command": playback_command,
        "playback_returncode": playback.returncode,
        "playback_stdout_sha256": hashlib.sha256(playback.stdout.encode()).hexdigest(),
        "playback_stderr_sha256": hashlib.sha256(playback.stderr.encode()).hexdigest(),
        "source_location_reproduced": source_location_reproduced,
        "source_unchanged": source_after == source_before,
        "replay_input_unchanged": harness_after == replay_input_sha,
        "artifact_dir": str(destination),
        "boundary": (
            "Explicit Kani/CBMC reference replay only. It reproduces a concrete "
            "source-level witness and does not discharge or replace the Verus proof."
        ),
    }
    (destination / "evidence.json").write_text(
        json.dumps(row, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return row


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--function", required=True)
    parser.add_argument("--cargo-kani", type=Path, required=True)
    parser.add_argument(
        "--output-root",
        type=Path,
        default=ROOT / ".aegis/destiny/kani-reference-replay",
    )
    parser.add_argument("--timeout", type=int, default=120)
    args = parser.parse_args()
    output_root = args.output_root if args.output_root.is_absolute() else ROOT / args.output_root
    try:
        result = replay(
            args.source,
            args.function,
            args.cargo_kani,
            output_root,
            args.timeout,
        )
    except (ReplayError, subset.Unsupported, OSError, subprocess.SubprocessError) as error:
        result = {
            "schema": 1,
            "status": "infrastructure_error",
            "engine": "kani_cbmc_reference",
            "explicit_reference_backend": True,
            "verus_proof_credit": False,
            "reason": str(error),
        }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result.get("status") == "reproduced_source_failure" else 1


if __name__ == "__main__":
    raise SystemExit(main())
