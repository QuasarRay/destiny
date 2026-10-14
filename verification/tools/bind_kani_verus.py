#!/usr/bin/env python3
"""Bind Kani compiler discovery to Destiny's reviewed Verus obligations and fresh receipts.

This tool never executes Kani or Verus. Kani discovery is produced by the single
metadata path in run_kani_autoharness_discovery.py + verification/automation.
The existing Verus campaign executes the proofs once; this tool only validates
that the Kani-selected supported identities map to exact fresh campaign receipts.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_REGISTRY = ROOT / "verification/automation/registry.json"
DEFAULT_PLAN = ROOT / "verification/plans/verus-concurrency.json"
KANI_COMMIT = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862"
VERUS_COMMIT = "aef82eda71838deef4a8cd0260fda6c9504be9b9"


class BindingError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def load_json(path: Path) -> Any:
    def unique(pairs):
        out = {}
        for key, value in pairs:
            if key in out:
                raise BindingError(f"duplicate JSON key {key!r} in {path}")
            out[key] = value
        return out

    try:
        return json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=unique,
            parse_constant=lambda value: (_ for _ in ()).throw(
                BindingError(f"non-finite JSON constant {value!r} in {path}")
            ),
        )
    except OSError as error:
        raise BindingError(f"cannot read {path}: {error}") from error


def load_registry(path: Path) -> dict[str, Any]:
    data = load_json(path)
    if data.get("schema") != 1:
        raise BindingError("unsupported registry schema")
    if data.get("default_engine") != "verus":
        raise BindingError("registry default engine must remain Verus")
    if data.get("kani", {}).get("vendor_commit") != KANI_COMMIT:
        raise BindingError("registry Kani commit does not match the pinned integration")
    if data.get("verus", {}).get("vendor_commit") != VERUS_COMMIT:
        raise BindingError("registry Verus commit does not match the pinned integration")
    return data


def supported(registry: dict[str, Any]) -> list[dict[str, Any]]:
    return [row for row in registry["candidates"] if row.get("status") == "supported"]


def bind_discovery(
    normalized_path: Path,
    registry_path: Path,
    plan_path: Path,
) -> dict[str, Any]:
    normalized = load_json(normalized_path)
    registry = load_registry(registry_path)
    plan = load_json(plan_path)

    if normalized.get("default_engine") != "verus":
        raise BindingError("normalized discovery changed the default engine")
    if normalized.get("proof_credit") is not False:
        raise BindingError("Kani discovery must not carry proof credit")
    if normalized.get("kani_source_commit") != KANI_COMMIT:
        raise BindingError("stale Kani discovery source identity")

    discovered = normalized.get("candidates")
    if not isinstance(discovered, list):
        raise BindingError("normalized discovery candidates are missing")

    by_name: dict[str, dict[str, Any]] = {}
    for row in discovered:
        name = row.get("function")
        if not isinstance(name, str) or not name:
            raise BindingError("discovery contains a candidate without a function identity")
        if name in by_name:
            raise BindingError(f"duplicate Kani candidate identity: {name}")
        by_name[name] = row

    supported_rows = supported(registry)
    supported_by_name = {row["kani_metadata_name"]: row for row in supported_rows}
    if len(supported_by_name) != len(supported_rows):
        raise BindingError("registry contains duplicate Kani metadata identities")

    records = []
    unresolved = []
    selected = []
    for name, row in sorted(by_name.items()):
        disposition = row.get("disposition")
        mapped = supported_by_name.get(name)
        if disposition == "kani_eligible" and mapped is not None:
            record = {
                "function": mapped["function"],
                "kani_metadata_name": name,
                "classification": "selected",
                "controller_obligation": mapped["controller_obligation"],
                "verus_entry": mapped["verus_entry"],
                "semantic_reason": mapped["reason"],
                "correspondence_boundary": (
                    "Reviewed source-to-existing-Verus mapping; no general Rust-to-Verus "
                    "translation theorem is claimed."
                ),
            }
            selected.append(record)
        elif disposition == "kani_eligible":
            record = {
                "kani_metadata_name": name,
                "classification": "intentionally_excluded",
                "reason": "not in the declared Verus semantic subset",
            }
        elif disposition == "excluded":
            record = {
                "kani_metadata_name": name,
                "classification": "intentionally_excluded",
                "reason": row.get("reason", "excluded by Kani discovery"),
            }
        elif disposition == "unsupported":
            record = {
                "kani_metadata_name": name,
                "classification": "unsupported",
                "reason": row.get("reason", "unsupported by Kani discovery"),
            }
        elif disposition == "blocked":
            record = {
                "kani_metadata_name": name,
                "classification": "blocked",
                "reason": row.get("reason", "blocked by Kani discovery"),
            }
        else:
            raise BindingError(f"unknown discovery disposition {disposition!r} for {name}")
        records.append(record)

    for name, mapped in sorted(supported_by_name.items()):
        row = by_name.get(name)
        if row is None:
            unresolved.append({
                "function": mapped["function"],
                "kani_metadata_name": name,
                "reason": "missing_from_kani_discovery",
            })
        elif row.get("disposition") != "kani_eligible":
            unresolved.append({
                "function": mapped["function"],
                "kani_metadata_name": name,
                "reason": f"required_supported_candidate_is_{row.get('disposition')}",
                "detail": row.get("reason"),
            })

    if not selected:
        unresolved.append({"reason": "empty_selected_supported_set"})

    plan_ids = {row["id"] for row in plan.get("obligations", [])}
    obligations = sorted({row["controller_obligation"] for row in selected})
    absent = sorted(set(obligations) - plan_ids)
    if absent:
        unresolved.append({"reason": "controller_obligations_absent_from_plan", "ids": absent})

    expected = sorted({row["controller_obligation"] for row in supported_rows})
    if not unresolved and obligations != expected:
        unresolved.append({
            "reason": "selected_obligation_inventory_mismatch",
            "selected": obligations,
            "expected": expected,
        })

    return {
        "schema": 1,
        "status": "BOUND_FOR_VERUS" if not unresolved else "INCOMPLETE",
        "default_engine": "verus",
        "kani_role": "compiler_autoharness_discovery_only",
        "kani_proof_credit": False,
        "kani_commit": KANI_COMMIT,
        "verus_commit": VERUS_COMMIT,
        "normalized_discovery": str(normalized_path),
        "normalized_discovery_sha256": sha256(normalized_path),
        "registry": str(registry_path),
        "registry_sha256": sha256(registry_path),
        "plan": str(plan_path),
        "plan_sha256": sha256(plan_path),
        "records": records,
        "selected": selected,
        "controller_obligations": obligations,
        "unresolved": unresolved,
        "notice": (
            "Kani determines automatic-harness eligibility. Verus remains the proof engine. "
            "The reviewed mapping is an explicit trusted correspondence boundary."
        ),
    }


def check_campaign(binding_path: Path, campaign_dir: Path) -> dict[str, Any]:
    binding = load_json(binding_path)
    if binding.get("status") != "BOUND_FOR_VERUS":
        raise BindingError("cannot credit an incomplete Kani-to-Verus binding")
    if binding.get("kani_proof_credit") is not False:
        raise BindingError("binding incorrectly carries Kani proof credit")

    binding_receipt = campaign_dir / "binding.json"
    audit_receipt = campaign_dir / "audit.json"
    if not binding_receipt.is_file() or not audit_receipt.is_file():
        raise BindingError("Verus campaign binding/audit receipts are missing")
    bound = load_json(binding_receipt)
    audit = load_json(audit_receipt)
    if bound.get("status") != "BOUND":
        raise BindingError("Verus campaign source binding is not BOUND")
    if audit.get("status") != "CHECKED_SCOPED":
        raise BindingError("Verus campaign audit is not CHECKED_SCOPED")

    receipts = []
    for obligation in binding.get("controller_obligations", []):
        path = campaign_dir / f"{obligation}.json"
        if not path.is_file():
            raise BindingError(f"missing fresh Verus receipt for {obligation}")
        payload = load_json(path)
        if payload.get("status") != "CHECKED_SCOPED":
            raise BindingError(f"Verus receipt {obligation} is not CHECKED_SCOPED")
        receipts.append({
            "id": obligation,
            "path": str(path),
            "sha256": sha256(path),
            "status": payload["status"],
        })

    if not receipts:
        raise BindingError("empty Verus receipt inventory cannot satisfy the bridge")

    return {
        "schema": 1,
        "status": "KANI_SELECTED_VERUS_CHECKED_SCOPED",
        "engine": "verus",
        "scope": "reviewed_verus_entries_selected_by_kani_autoharness_discovery",
        "kani_proof_credit": False,
        "verus_checked_scoped": True,
        "binding_sha256": sha256(binding_path),
        "campaign_binding_sha256": sha256(binding_receipt),
        "campaign_audit_sha256": sha256(audit_receipt),
        "receipts": receipts,
        "translation_correctness": "reviewed_mapping_boundary_not_general_translation_proof",
        "notice": (
            "This credits only the mapped reviewed Verus entry scopes. It does not establish "
            "general Kani compatibility, end-to-end translation soundness, or completed Destiny parity."
        ),
    }


def write_output(path: Path | None, payload: dict[str, Any]) -> None:
    text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if path is not None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    print(text, end="")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument("--plan", type=Path, default=DEFAULT_PLAN)
    sub = parser.add_subparsers(dest="command", required=True)

    bind = sub.add_parser("bind")
    bind.add_argument("--normalized", type=Path, required=True)
    bind.add_argument("--output", type=Path, required=True)

    campaign = sub.add_parser("check-campaign")
    campaign.add_argument("--binding", type=Path, required=True)
    campaign.add_argument("--campaign-dir", type=Path, required=True)
    campaign.add_argument("--output", type=Path, required=True)

    sub.add_parser("explain")

    args = parser.parse_args()
    registry_path = args.registry if args.registry.is_absolute() else ROOT / args.registry
    plan_path = args.plan if args.plan.is_absolute() else ROOT / args.plan

    if args.command == "bind":
        normalized = args.normalized if args.normalized.is_absolute() else ROOT / args.normalized
        output = args.output if args.output.is_absolute() else ROOT / args.output
        payload = bind_discovery(normalized, registry_path, plan_path)
        write_output(output, payload)
        return 0 if payload["status"] == "BOUND_FOR_VERUS" else 1

    if args.command == "check-campaign":
        binding = args.binding if args.binding.is_absolute() else ROOT / args.binding
        campaign_dir = (
            args.campaign_dir if args.campaign_dir.is_absolute() else ROOT / args.campaign_dir
        )
        output = args.output if args.output.is_absolute() else ROOT / args.output
        payload = check_campaign(binding, campaign_dir)
        write_output(output, payload)
        return 0

    if args.command == "explain":
        reg = load_registry(registry_path)
        write_output(None, {
            "default_engine": reg["default_engine"],
            "supported_subset": reg["supported_subset"],
            "candidates": reg["candidates"],
        })
        return 0

    raise BindingError(f"unknown command: {args.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (BindingError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "INCOMPLETE", "reason": str(error)}, sort_keys=True))
        raise SystemExit(1)
