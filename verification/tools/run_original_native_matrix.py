#!/usr/bin/env python3
"""Run all latest-source Python cases through the existing exact-case runner.

Imports that cannot execute remain explicit failures. The runner checks every
import against original Git blobs; equal legacy/latest hashes preserve the
existing tested fixture machinery without rewriting originals.
"""
import argparse
from collections import defaultdict
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / ".agents/infra"))
from agentinfra.process import run_process


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", required=True, type=Path)
    parser.add_argument("--original-root", required=True, type=Path)
    parser.add_argument("--output-dir", type=Path, default=ROOT / ".aegis/destiny/migration/original-native")
    parser.add_argument("--reuse-blocked", type=Path, help="Retain diagnosed missing-Blue failures while the dependency is still absent")
    args = parser.parse_args()
    directory = args.output_dir.resolve()
    if not directory.is_relative_to(ROOT / ".aegis"):
        raise ValueError("matrix output must remain in .aegis")
    directory.mkdir(parents=True, exist_ok=True)
    inventory_path = ROOT / "verification/migration/source_inventory.json"
    inventory = json.loads(inventory_path.read_text())
    legacy, latest = inventory["revisions"]
    # The reused runner binds every fixture/helper to the legacy Git blobs.
    # Reuse it only while the entire Python source boundary is source-equal.
    older_python = {row["path"]: row["sha256"] for row in legacy["files"] if row["path"].endswith(".py")}
    newer_python = {row["path"]: row["sha256"] for row in latest["files"] if row["path"].endswith(".py")}
    if older_python != newer_python:
        raise ValueError("latest original Python modules/helpers differ; review and update the pinned fixture reader")
    legacy_by_id = {case["id"]: case for case in legacy["cases"]}
    groups = defaultdict(list)
    for case in latest["cases"]:
        if not case["source"].endswith(".py"):
            continue
        if legacy_by_id.get(case["id"], {}).get("source_sha256") != case["source_sha256"]:
            raise ValueError("existing runner's original-source revision needs a new reviewed fixture reader")
        groups[case["source"]].append(case)
    observations, executions = [], []
    prior = json.loads(args.reuse_blocked.read_text()) if args.reuse_blocked else None
    prior_cases = {row["id"]: row for row in prior["cases"]} if prior else {}
    import importlib.util
    missing_blue = importlib.util.find_spec("blue") is None
    for source, cases in sorted(groups.items()):
        for offset in range(0, len(cases), 32):
            batch = cases[offset:offset + 32]
            name = Path(source).stem + "-" + hashlib.sha256(source.encode()).hexdigest()[:8] + "-" + str(offset // 32)
            report = directory / (name + ".json")
            command = [sys.executable, str(ROOT / "verification/tools/run_original_native_cases.py"),
                       "--library", str(args.library.resolve()), "--original-root", str(args.original_root.resolve()),
                       "--report", str(report)]
            for case in batch:
                command.extend(["--case", case["id"]])
            if prior and missing_blue and all(
                    prior_cases.get(case["id"], {}).get("status") in {"BLOCKED_EXECUTION", "BLOCKED_MISSING_BLUE"}
                    and prior_cases[case["id"]].get("source_sha256") == case["source_sha256"]
                    for case in batch):
                for case in batch:
                    observations.append({**prior_cases[case["id"]], "status": "BLOCKED_MISSING_BLUE",
                                         "reason": "Previously diagnosed original execution requires blue; current import lookup still reports it unavailable. No unchanged failed execution repeated.",
                                         "diagnostic_limit": "Initial basename-only log names collided for client/server ticker; those prior per-case outcomes remain blocked and grant no execution credit."})
                executions.append({"source": source, "status": "RETAINED_EXTERNAL_BLOCKER_NO_RETRY"})
                continue
            # Never mistake a report from an older source/library for this run.
            if report.exists():
                retained = json.loads(report.read_text())
                if retained["library_sha256"] != hashlib.sha256(args.library.read_bytes()).hexdigest() or retained["qualified_cases"] != [case["id"] for case in batch]:
                    raise ValueError("retained matrix observation differs from requested binary/cases")
                for path, digest in retained["production_python_sources"].items():
                    if hashlib.sha256((ROOT / path).read_bytes()).hexdigest() != digest:
                        raise ValueError("retained production Python source changed")
                executions.append({"source": source, "status": "REUSED_SAME_BINARY_AND_SOURCES_OBSERVATION", "report": str(report.relative_to(ROOT))})
            else:
                result = run_process(command, cwd=ROOT, timeout=90, env={"PATH": os.environ["PATH"]}, capture_limit=2_500_000)
                (directory / (name + "-execution.json")).write_text(json.dumps(asdict(result), indent=2) + "\n")
                executions.append({"source": source, "exit_code": result.returncode, "report": str(report.relative_to(ROOT)), "timed_out": result.timed_out})
            if report.exists():
                original = json.loads(report.read_text())
                by_name = {row["test"]: row for row in original["observations"]}
                for case in batch:
                    suffix = "." + case["fixture"] + "." + case["test"]
                    matches = [row for name, row in by_name.items() if name.endswith(suffix)]
                    if len(matches) != 1:
                        raise ValueError("missing or ambiguous exact original outcome")
                    observations.append({"id": case["id"], **matches[0], "source_sha256": case["source_sha256"],
                                         "evidence": str(report.relative_to(ROOT))})
            else:
                for case in batch:
                    observations.append({"id": case["id"], "status": "BLOCKED_EXECUTION", "native_calls": 0,
                                         "source_sha256": case["source_sha256"], "evidence": str((directory / (name + "-execution.json")).relative_to(ROOT)),
                                         "reason": "Exact original runner failed before producing an outcome; full diagnostic retained."})
    counts = {status: sum(row["status"] == status for row in observations) for status in sorted({row["status"] for row in observations})}
    document = {"schema": 1, "original_commit": latest["commit"], "library": str(args.library.resolve()), "library_sha256": hashlib.sha256(args.library.read_bytes()).hexdigest(),
                "inventory_sha256": hashlib.sha256(inventory_path.read_bytes()).hexdigest(), "proof_credit": False,
                "matrix_runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "case_runner_sha256": hashlib.sha256((ROOT / "verification/tools/run_original_native_cases.py").read_bytes()).hexdigest(),
                "counts": counts, "cases": observations, "executions": executions,
                "native_passed": sum(row["status"] == "PASSED" and row["native_calls"] > 0 for row in observations),
                "passed_without_native_calls": sum(row["status"] == "PASSED" and row["native_calls"] == 0 for row in observations),
                "production_python_sha256": {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted((ROOT / "python").rglob("*.py"))},
                "production_rust_sha256": {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                                           for directory in (ROOT / "src", ROOT / "verification/spec/src")
                                           for path in sorted(directory.rglob("*.rs"))},
                "build_input_sha256": {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
                                       for path in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "verification/spec/Cargo.toml", "verification/spec/Cargo.lock")},
                "source_binding_limit": "Current sources observed with the selected binary; artifact-to-source reproducible rebuild binding and original-engine correspondence remain separate obligations.",
                "notice": "Unchanged original Python fixtures against Rust native execution. This is not a C++ engine differential or a formal proof."}
    (directory / "summary.json").write_text(json.dumps(document, indent=2) + "\n")
    pointer = {"report": str((directory / "summary.json").relative_to(ROOT)),
               "sha256": hashlib.sha256((directory / "summary.json").read_bytes()).hexdigest()}
    (ROOT / ".aegis/destiny/migration/latest-original-native-matrix.json").write_text(json.dumps(pointer, indent=2) + "\n")
    print(json.dumps({"selected": len(observations), "counts": counts, "proof_credit": False}))
    return 0 if all(row["status"] == "PASSED" and row["native_calls"] > 0 for row in observations) else 1


if __name__ == "__main__":
    sys.exit(main())
