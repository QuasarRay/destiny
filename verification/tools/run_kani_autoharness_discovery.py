#!/usr/bin/env python3
"""Run patched vendored Kani autoharness discovery while hard-failing backend tool execution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_clause_evidence(directory: Path) -> list[dict[str, str]]:
    records = []
    for path in sorted(directory.glob("*.clause")):
        parts = path.read_text(encoding="utf-8").split("\n", 4)
        if len(parts) != 5 or parts[0] != "kverus_contract_clause_v1":
            raise SystemExit(f"Malformed KVerus contract-clause evidence: {path}")
        _, crate_name, function, kind, expression = parts
        if not function or kind not in {"requires", "ensures", "modifies"} or not expression.strip():
            raise SystemExit(f"Invalid KVerus contract-clause evidence: {path}")
        records.append({
            "path": str(path),
            "sha256": digest(path),
            "crate_name": crate_name.replace("-", "_"),
            "function": function,
            "kind": kind,
            "expression": expression,
        })
    return records


def assert_effective_kani_sysroot(target_dir: Path, expected_sysroot: Path) -> list[dict[str, str]]:
    observed = []
    for path in sorted(target_dir.rglob("fingerprint/*.json")):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        rustflags = payload.get("rustflags")
        if not isinstance(rustflags, list):
            continue
        values = []
        for index, flag in enumerate(rustflags[:-1]):
            if flag == "--sysroot":
                values.append(rustflags[index + 1])
        for value in values:
            observed.append({
                "fingerprint": str(path),
                "sysroot": value,
            })

    if not observed:
        raise SystemExit("Cargo fingerprints did not record an effective Kani --sysroot")

    expected = str(expected_sysroot)
    mismatched = [row for row in observed if Path(row["sysroot"]).resolve() != expected_sysroot]
    if mismatched:
        raise SystemExit(
            "Kani discovery escaped the vendored dev sysroot; expected "
            f"{expected}, observed {mismatched[:8]}"
        )
    return observed


def bind_clause_evidence(metadata: list[dict], records: list[dict[str, str]]) -> None:
    targets = []
    for payload in metadata:
        crate_name = payload.get("crate_name")
        if not isinstance(crate_name, str) or not crate_name:
            raise SystemExit("Kani metadata is missing crate_name")
        rows = payload.get("contracted_functions", [])
        if not isinstance(rows, list):
            raise SystemExit(f"Malformed contracted_functions metadata for {crate_name}")
        for row in rows:
            if not isinstance(row, dict) or not isinstance(row.get("function"), str):
                raise SystemExit(f"Malformed contracted function metadata for {crate_name}")
            targets.append((crate_name.replace("-", "_"), row))

    for record in records:
        matches = [
            row
            for crate_name, row in targets
            if row["function"].rsplit("::", 1)[-1] == record["function"]
            and (not record["crate_name"] or crate_name == record["crate_name"])
        ]
        if len(matches) != 1:
            raise SystemExit(
                "KVerus parser clause does not map to exactly one Kani contracted function: "
                f"{record['crate_name']}::{record['function']} -> {len(matches)} matches"
            )
        row = matches[0]
        clauses = row.setdefault("clauses", [])
        if not isinstance(clauses, list):
            raise SystemExit(f"Malformed clauses metadata for {row['function']}")
        clause = {"kind": record["kind"], "expression": record["expression"]}
        if clause not in clauses:
            clauses.append(clause)

    for _, row in targets:
        clauses = row.setdefault("clauses", [])
        if not clauses:
            raise SystemExit(
                f"Kani reports {row['function']} as contracted but KVerus received no "
                "Kani-parser clause evidence"
            )
        unique = {
            (clause.get("kind"), clause.get("expression"))
            for clause in clauses
            if isinstance(clause, dict)
        }
        if len(unique) != len(clauses):
            clauses[:] = [
                {"kind": kind, "expression": expression}
                for kind, expression in sorted(unique)
            ]
        else:
            clauses.sort(key=lambda clause: (clause["kind"], clause["expression"]))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--kani-driver",
        type=Path,
        default=ROOT / "third_party/kani/scripts/cargo-kani",
        help=(
            "Kani cargo-mode entrypoint. The vendored default is Kani's developer script, "
            "which execs target/kani/bin/kani-driver with argv[0]=cargo-kani."
        ),
    )
    parser.add_argument(
        "--manifest-path",
        type=Path,
        default=ROOT / "verification/spec/Cargo.toml",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / ".aegis/destiny/kani-autoharness-discovery",
    )
    args = parser.parse_args()

    driver = args.kani_driver.resolve(strict=True)
    if driver.parent.name == "scripts" and driver.name == "cargo-kani":
        kani_repo = driver.parent.parent
        driver_binary = (kani_repo / "target/kani/bin/kani-driver").resolve(strict=True)
        kani_root = (kani_repo / "target/kani").resolve(strict=True)
    else:
        driver_binary = driver
        kani_root = driver.parent.parent.resolve(strict=True)
    compiler = (kani_root / "bin/kani-compiler").resolve(strict=True)
    compiler_marker = b"KVERUS_CONTRACT_TRANSPORT_EMPTY"
    if compiler_marker not in compiler.read_bytes():
        raise SystemExit(
            "Selected Kani driver is not paired with the patched KVerus kani-compiler"
        )

    proc_macro_candidates = sorted(
        path
        for pattern in ("libkani_macros*.so", "libkani_macros*.dylib", "kani_macros*.dll")
        for path in (kani_root / "lib").glob(pattern)
        if path.is_file()
    )
    if len(proc_macro_candidates) != 1:
        raise SystemExit(
            "Expected exactly one Kani proc-macro library in the dev sysroot; "
            f"found {[str(path) for path in proc_macro_candidates]}"
        )
    proc_macro = proc_macro_candidates[0].resolve(strict=True)
    proc_macro_marker = b"kverus_contract_clause"
    if proc_macro_marker not in proc_macro.read_bytes():
        raise SystemExit(
            "Selected Kani dev sysroot does not contain the patched KVerus kani_macros "
            "contract transport"
        )

    manifest = args.manifest_path.resolve(strict=True)
    out = args.output_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)
    clause_dir = out / "kani-contract-clauses"
    fallback_clause_dir = Path(tempfile.gettempdir()) / "kverus-contract-clauses"
    for directory in (clause_dir, fallback_clause_dir):
        if directory.exists():
            shutil.rmtree(directory)
        directory.mkdir(parents=True)
    forbidden_log = out / "forbidden-backend-tools.log"
    if forbidden_log.exists():
        forbidden_log.unlink()

    with tempfile.TemporaryDirectory(prefix="destiny-no-cbmc-") as tmp:
        blocker = Path(tmp)
        for name in ("cbmc", "goto-cc", "goto-instrument", "goto-analyzer"):
            path = blocker / name
            path.write_text(
                "#!/bin/sh\necho " + name + " >> " + repr(str(forbidden_log)) + "\nexit 97\n",
                encoding="utf-8",
            )
            path.chmod(path.stat().st_mode | stat.S_IXUSR)

        env = os.environ.copy()
        env["PATH"] = str(blocker) + os.pathsep + env.get("PATH", "")
        env["KVERUS_CONTRACT_CLAUSE_DIR"] = str(clause_dir)
        cmd = [
            str(driver),
            "autoharness",
            "-Z",
            "autoharness",
            "--list",
            "--format",
            "json",
            "--quiet",
            "--manifest-path",
            str(manifest),
            "--target-dir",
            str(out / "kani-target"),
        ]
        result = subprocess.run(cmd, cwd=out, env=env, text=True, capture_output=True)

    (out / "command.json").write_text(json.dumps(cmd, indent=2) + "\n", encoding="utf-8")
    (out / "stdout.txt").write_text(result.stdout, encoding="utf-8")
    (out / "stderr.txt").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise SystemExit(
            f"Kani metadata-only discovery failed with exit {result.returncode}; see {out}"
        )
    if forbidden_log.exists() and forbidden_log.read_text(encoding="utf-8").strip():
        raise SystemExit("A forbidden CBMC/GOTO backend tool was invoked")

    effective_sysroots = assert_effective_kani_sysroot(out / "kani-target", kani_root)

    metadata = sorted((out / "kani-target").rglob("*.kani-metadata.json"))
    if not metadata:
        raise SystemExit("Kani produced no .kani_metadata.json artifacts")

    combined = []
    for path in metadata:
        payload = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise SystemExit(f"Malformed Kani metadata object: {path}")
        combined.append(payload)

    clause_evidence = read_clause_evidence(clause_dir)
    fallback_clause_evidence = read_clause_evidence(fallback_clause_dir)
    seen_evidence = {
        (record["crate_name"], record["function"], record["kind"], record["expression"])
        for record in clause_evidence
    }
    clause_evidence.extend(
        record
        for record in fallback_clause_evidence
        if (record["crate_name"], record["function"], record["kind"], record["expression"])
        not in seen_evidence
    )
    bind_clause_evidence(combined, clause_evidence)

    combined_path = out / "kani-metadata.json"
    combined_path.write_text(
        json.dumps(combined, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    receipt = {
        "status": "KANI_AUTOHARNESS_DISCOVERY_ONLY",
        "proof_credit": False,
        "manifest": str(manifest.relative_to(ROOT)),
        "driver": str(driver),
        "driver_binary": {
            "path": str(driver_binary),
            "sha256": digest(driver_binary),
        },
        "effective_sysroot": {
            "expected": str(kani_root),
            "fingerprints": effective_sysroots,
        },
        "compiler": {
            "path": str(compiler),
            "sha256": digest(compiler),
            "kverus_transport_marker": compiler_marker.decode("ascii"),
        },
        "proc_macro": {
            "path": str(proc_macro),
            "sha256": digest(proc_macro),
            "kverus_transport_marker": proc_macro_marker.decode("ascii"),
        },
        "contract_clause_evidence": {
            "source": "patched_kani_macros_syn_parser",
            "directory": str(clause_dir.relative_to(ROOT)),
            "fallback_directory": str(fallback_clause_dir),
            "proof_credit": False,
            "records": clause_evidence,
        },
        "combined_metadata": {
            "path": str(combined_path.relative_to(ROOT)),
            "sha256": digest(combined_path),
        },
        "metadata": [
            {
                "path": str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path),
                "sha256": digest(path),
            }
            for path in metadata
        ],
        "list_json": str((out / "kani-list.json").relative_to(ROOT))
        if (out / "kani-list.json").is_file()
        else None,
        "backend_tools_forbidden": ["cbmc", "goto-cc", "goto-instrument", "goto-analyzer"],
    }
    (out / "receipt.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(receipt, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
