#!/usr/bin/env python3
"""Kani-assisted discovery with Verus as Destiny's default verification engine.

Discovery reuses Kani 0.68.0 autoharness compiler metadata. The pinned
autoharness --list path implies codegen-only, so discovery does not execute
CBMC. Verification delegates to Destiny's maintained controller and existing
Verus plan; Kani success can never discharge a Verus obligation.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_REGISTRY = ROOT / "verification/automation/registry.json"
DEFAULT_PLAN = ROOT / "verification/plans/verus-concurrency.json"
DEFAULT_STATE = ROOT / ".aegis/destiny/verus-kani-automation"


class BridgeError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise BridgeError(f"cannot read JSON {path}: {error}") from error


def run(argv: list[str], *, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        argv,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode:
        raise BridgeError(
            f"command failed ({result.returncode}): {' '.join(argv)}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def registry(path: Path) -> dict[str, Any]:
    data = load_json(path)
    if data.get("schema") != 1:
        raise BridgeError("unsupported automation registry schema")
    if data.get("default_engine") != "verus":
        raise BridgeError("automation registry must keep Verus as the default engine")
    return data


def supported_candidates(data: dict[str, Any]) -> list[dict[str, Any]]:
    return [item for item in data["candidates"] if item["status"] == "supported"]


def exact_pattern(function: str) -> str:
    return "^" + re.escape(function) + "$"


def normalize_skip_reason(value: Any) -> dict[str, Any]:
    if isinstance(value, str):
        return {"kind": value}
    if isinstance(value, dict) and len(value) == 1:
        kind, detail = next(iter(value.items()))
        return {"kind": kind, "detail": detail}
    return {"kind": "MalformedSkipReason", "detail": value}


def collect_autoharness_metadata(
    target_dir: Path,
    candidates: list[dict[str, Any]],
) -> dict[str, Any]:
    expected = {item["function"]: item for item in candidates}
    chosen: dict[str, dict[str, Any]] = {}
    skipped: dict[str, dict[str, Any]] = {}
    files = sorted(target_dir.rglob("*.kani-metadata.json"))
    if not files:
        raise BridgeError(f"no .kani-metadata.json files found below {target_dir}")

    artifacts: list[dict[str, str]] = []
    for path in files:
        payload = load_json(path)
        artifacts.append({"path": str(path), "sha256": sha256(path)})
        auto = payload.get("autoharness_md")
        if not auto:
            continue
        for name in auto.get("chosen", []):
            if name in expected:
                chosen[name] = {"metadata": str(path)}
        for name, reason in auto.get("skipped", {}).items():
            if name in expected:
                skipped[name] = {
                    "metadata": str(path),
                    "reason": normalize_skip_reason(reason),
                }

    records: list[dict[str, Any]] = []
    for name, item in sorted(expected.items()):
        if name in chosen:
            records.append({
                "function": name,
                "status": "selected",
                "controller_obligation": item["controller_obligation"],
                "verus_entry": item["verus_entry"],
                "semantic_reason": item["reason"],
            })
        elif name in skipped:
            records.append({
                "function": name,
                "status": "blocked",
                "reason": skipped[name]["reason"],
                "smallest_next_step": "Resolve the Kani-reported eligibility blocker without narrowing the domain.",
            })
        else:
            records.append({
                "function": name,
                "status": "blocked",
                "reason": {"kind": "MissingFromKaniAutoharnessMetadata"},
                "smallest_next_step": "Inspect the pinned Kani metadata artifact and exact definition path.",
            })
    return {"records": records, "metadata_artifacts": artifacts}


def discover(
    *,
    cargo_kani: Path,
    registry_path: Path,
    output: Path,
    state_dir: Path,
) -> dict[str, Any]:
    reg = registry(registry_path)
    candidates = supported_candidates(reg)
    if not candidates:
        raise BridgeError("registry contains no supported candidates")

    version = run([str(cargo_kani), "--version"]).stdout.strip()
    if reg["kani"]["version"] not in version:
        raise BridgeError(
            f"Kani version mismatch: expected {reg['kani']['version']}, got {version!r}"
        )

    manifest = ROOT / reg["manifest"]
    if not manifest.is_file():
        raise BridgeError(f"missing manifest: {manifest}")

    run_dir = state_dir / "discovery"
    target_dir = run_dir / "target"
    run_dir.mkdir(parents=True, exist_ok=True)
    target_dir.mkdir(parents=True, exist_ok=True)

    argv = [
        str(cargo_kani),
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
        str(target_dir),
    ]
    for item in candidates:
        argv.extend(["--include-pattern", exact_pattern(item["function"])])

    result = run(argv, cwd=run_dir)
    parsed = collect_autoharness_metadata(target_dir, candidates)
    selected = [r for r in parsed["records"] if r["status"] == "selected"]
    blocked = [r for r in parsed["records"] if r["status"] != "selected"]
    status = (
        "INCOMPLETE_EMPTY_SELECTION"
        if not selected
        else "INCOMPLETE"
        if blocked
        else "DISCOVERED"
    )
    obligations = sorted({r["controller_obligation"] for r in selected})

    payload = {
        "schema": 1,
        "status": status,
        "engine": "verus",
        "kani_role": "autoharness-metadata-only",
        "cbmc_executed": False,
        "kani_version_output": version,
        "kani_vendor_commit": reg["kani"]["vendor_commit"],
        "verus_vendor_commit": reg["verus"]["vendor_commit"],
        "manifest": str(manifest.relative_to(ROOT)),
        "manifest_sha256": sha256(manifest),
        "registry": str(registry_path.relative_to(ROOT)),
        "registry_sha256": sha256(registry_path),
        "command": argv,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "records": parsed["records"],
        "metadata_artifacts": parsed["metadata_artifacts"],
        "controller_obligations": obligations,
        "notice": (
            "Kani selected source candidates and emitted metadata only. "
            "No CBMC result is represented here, and no Verus result exists until "
            "the maintained controller checks the mapped reviewed entries."
        ),
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return payload


def validate_discovery(
    discovery_path: Path,
    registry_path: Path,
    plan_path: Path,
) -> dict[str, Any]:
    data = load_json(discovery_path)
    reg = registry(registry_path)
    plan = load_json(plan_path)

    if data.get("schema") != 1 or data.get("engine") != "verus":
        raise BridgeError("invalid discovery schema/engine")
    if data.get("cbmc_executed") is not False:
        raise BridgeError("default Verus discovery must not execute CBMC")
    if data.get("kani_vendor_commit") != reg["kani"]["vendor_commit"]:
        raise BridgeError("stale Kani source identity in discovery")
    if data.get("verus_vendor_commit") != reg["verus"]["vendor_commit"]:
        raise BridgeError("stale Verus source identity in discovery")
    if data.get("registry_sha256") != sha256(registry_path):
        raise BridgeError("stale registry identity in discovery")
    manifest = ROOT / reg["manifest"]
    if data.get("manifest_sha256") != sha256(manifest):
        raise BridgeError("stale manifest identity in discovery")

    selected = [r for r in data.get("records", []) if r.get("status") == "selected"]
    blocked = [r for r in data.get("records", []) if r.get("status") != "selected"]
    if not selected:
        raise BridgeError("empty selected candidate set cannot satisfy verification")
    if blocked:
        raise BridgeError(
            "automation discovery is incomplete: "
            + ", ".join(r.get("function", "<unknown>") for r in blocked)
        )

    plan_ids = {item["id"] for item in plan.get("obligations", [])}
    required = set(data.get("controller_obligations", []))
    missing = required - plan_ids
    if missing:
        raise BridgeError(f"discovery references obligations absent from plan: {sorted(missing)}")

    expected = {item["controller_obligation"] for item in supported_candidates(reg)}
    if required != expected:
        raise BridgeError(
            f"discovery obligation inventory mismatch: {sorted(required)} != {sorted(expected)}"
        )

    return {
        "status": "DISCOVERY_BOUND",
        "selected_candidates": len(selected),
        "controller_obligations": sorted(required),
        "plan": str(plan_path.relative_to(ROOT)),
    }


def verify_with_controller(
    *,
    discovery_path: Path,
    registry_path: Path,
    plan_path: Path,
    verus: Path,
    budget_seconds: int,
    state_dir: Path,
) -> dict[str, Any]:
    bound = validate_discovery(discovery_path, registry_path, plan_path)
    controller = ROOT / ".agents/bin/destinyctl.py"
    if not controller.is_file():
        raise BridgeError(
            "maintained controller is not materialized at .agents/bin/destinyctl.py; "
            "follow verification/tools/bootstrap_controller.py first"
        )

    prefix = [sys.executable, "-B", str(controller), "--project", str(ROOT)]
    binding = json.loads(run(prefix + ["bind", str(plan_path.relative_to(ROOT))]).stdout)
    if binding.get("status") != "BOUND":
        raise BridgeError("controller source binding failed")

    result_dir = state_dir / "controller"
    result_dir.mkdir(parents=True, exist_ok=True)
    results = []
    for obligation in bound["controller_obligations"]:
        payload = json.loads(
            run(
                prefix
                + [
                    "verify",
                    obligation,
                    "--budget-seconds",
                    str(budget_seconds),
                    "--verus",
                    str(verus),
                ]
            ).stdout
        )
        (result_dir / f"{obligation}.json").write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        if payload.get("status") != "CHECKED_SCOPED":
            raise BridgeError(f"Verus obligation {obligation} is incomplete")
        results.append({"id": obligation, "status": payload["status"]})

    return {
        "status": "CHECKED_SCOPED",
        "engine": "verus",
        "binding": binding.get("binding"),
        "selected_candidates": bound["selected_candidates"],
        "obligations": results,
        "notice": (
            "These are existing reviewed entry scopes. No general Rust-to-Verus "
            "translation soundness or completed Destiny parity is claimed."
        ),
    }


def run_kani_reference(*, cargo_kani: Path, registry_path: Path) -> dict[str, Any]:
    reg = registry(registry_path)
    argv = [
        str(cargo_kani),
        "autoharness",
        "-Z",
        "autoharness",
        "--manifest-path",
        str(ROOT / reg["manifest"]),
        "--jobs=1",
        "--output-format=terse",
    ]
    for item in supported_candidates(reg):
        argv.extend(["--include-pattern", exact_pattern(item["function"])])
    result = run(argv)
    return {
        "status": "REFERENCE_COMPLETE",
        "engine": "kani",
        "command": argv,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "notice": "Explicit Kani/CBMC reference mode; this cannot discharge a Verus obligation.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument("--state-dir", type=Path, default=DEFAULT_STATE)
    sub = parser.add_subparsers(dest="command", required=True)

    doctor = sub.add_parser("doctor")
    doctor.add_argument("--cargo-kani", type=Path)
    doctor.add_argument("--verus", type=Path)

    discover_cmd = sub.add_parser("discover")
    discover_cmd.add_argument("--cargo-kani", type=Path, required=True)
    discover_cmd.add_argument("--output", type=Path, required=True)

    check = sub.add_parser("check-discovery")
    check.add_argument("--discovery", type=Path, required=True)
    check.add_argument("--plan", type=Path, default=DEFAULT_PLAN)

    verify = sub.add_parser("verify")
    verify.add_argument("--engine", choices=("verus", "kani"), default="verus")
    verify.add_argument("--cargo-kani", type=Path)
    verify.add_argument("--discovery", type=Path)
    verify.add_argument("--verus", type=Path)
    verify.add_argument("--plan", type=Path, default=DEFAULT_PLAN)
    verify.add_argument("--budget-seconds", type=int, default=120)

    sub.add_parser("explain")
    sub.add_parser("replay")

    args = parser.parse_args()
    registry_path = args.registry if args.registry.is_absolute() else ROOT / args.registry
    state_dir = args.state_dir if args.state_dir.is_absolute() else ROOT / args.state_dir

    if args.command == "doctor":
        payload: dict[str, Any] = {
            "status": "READY",
            "registry": str(registry_path.relative_to(ROOT)),
            "vendor_lock": (ROOT / "verification/vendor-lock.json").is_file(),
            "controller": (ROOT / ".agents/bin/destinyctl.py").is_file(),
        }
        if args.cargo_kani:
            payload["kani_version"] = run([str(args.cargo_kani), "--version"]).stdout.strip()
        if args.verus:
            payload["verus_version"] = run([str(args.verus), "--version"]).stdout.strip()
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if args.command == "discover":
        output = args.output if args.output.is_absolute() else ROOT / args.output
        payload = discover(
            cargo_kani=args.cargo_kani,
            registry_path=registry_path,
            output=output,
            state_dir=state_dir,
        )
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0 if payload["status"] == "DISCOVERED" else 1

    if args.command == "check-discovery":
        discovery_path = args.discovery if args.discovery.is_absolute() else ROOT / args.discovery
        plan_path = args.plan if args.plan.is_absolute() else ROOT / args.plan
        print(json.dumps(validate_discovery(discovery_path, registry_path, plan_path), indent=2, sort_keys=True))
        return 0

    if args.command == "verify":
        if args.engine == "kani":
            if not args.cargo_kani:
                raise BridgeError("--cargo-kani is required for explicit Kani reference mode")
            print(json.dumps(run_kani_reference(cargo_kani=args.cargo_kani, registry_path=registry_path), indent=2, sort_keys=True))
            return 0

        if not args.verus:
            raise BridgeError("--verus is required for the default Verus engine")
        if args.discovery:
            discovery_path = args.discovery if args.discovery.is_absolute() else ROOT / args.discovery
        else:
            if not args.cargo_kani:
                raise BridgeError("--cargo-kani is required when --discovery is not supplied")
            discovery_path = state_dir / "discovery.json"
            payload = discover(
                cargo_kani=args.cargo_kani,
                registry_path=registry_path,
                output=discovery_path,
                state_dir=state_dir,
            )
            if payload["status"] != "DISCOVERED":
                raise BridgeError("discovery is incomplete; Verus verification not started")

        plan_path = args.plan if args.plan.is_absolute() else ROOT / args.plan
        print(json.dumps(
            verify_with_controller(
                discovery_path=discovery_path,
                registry_path=registry_path,
                plan_path=plan_path,
                verus=args.verus,
                budget_seconds=args.budget_seconds,
                state_dir=state_dir,
            ),
            indent=2,
            sort_keys=True,
        ))
        return 0

    if args.command == "explain":
        reg = registry(registry_path)
        print(json.dumps({
            "default_engine": reg["default_engine"],
            "supported_subset": reg["supported_subset"],
            "candidates": reg["candidates"],
        }, indent=2, sort_keys=True))
        return 0

    if args.command == "replay":
        print(json.dumps({
            "status": "UNSUPPORTED",
            "reason": (
                "The initial Verus bridge has no source-level concrete model decoder. "
                "A failed deductive obligation is not relabeled as a reproduced bug."
            ),
            "smallest_next_step": (
                "Add a source-validated decoder/reproducer for a supported failing fixture "
                "before enabling replay."
            ),
        }, indent=2, sort_keys=True))
        return 2

    raise BridgeError(f"unknown command: {args.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (BridgeError, OSError, KeyError, ValueError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "INCOMPLETE", "reason": str(error)}, sort_keys=True))
        raise SystemExit(1)
