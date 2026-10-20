#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification/tools/bind_kani_verus.py"
SPEC = importlib.util.spec_from_file_location("bind_kani_verus", MODULE)
assert SPEC and SPEC.loader
bridge = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = bridge
SPEC.loader.exec_module(bridge)


class BindingTests(unittest.TestCase):
    def write(self, root: Path, name: str, payload) -> Path:
        path = root / name
        path.write_text(json.dumps(payload), encoding="utf-8")
        return path

    def registry(self):
        return {
            "schema": 1,
            "default_engine": "verus",
            "kani": {"vendor_commit": bridge.KANI_COMMIT},
            "verus": {"vendor_commit": bridge.VERUS_COMMIT},
            "supported_subset": {},
            "candidates": [
                {
                    "function": "destiny_original_spec::model::assign_flag",
                    "status": "supported",
                    "reason": "Boolean identity",
                    "controller_obligation": "state-contracts-reviewed-inventory",
                    "verus_entry": "verification/verus/state_contracts.rs",
                    "kani_metadata_name": "model::assign_flag",
                }
            ],
        }

    def normalized(self, disposition="kani_eligible"):
        return {
            "default_engine": "verus",
            "proof_credit": False,
            "kani_source_commit": bridge.KANI_COMMIT,
            "crates_with_autoharness_metadata": 1,
            "candidates": [
                {
                    "crate_name": "destiny_original_spec",
                    "function": "model::assign_flag",
                    "disposition": disposition,
                    "reason": "fixture",
                    "provenance": "kani_metadata::AutoHarnessMetadata::chosen",
                },
                {
                    "crate_name": "destiny_original_spec",
                    "function": "model::apply_non_negative_setter",
                    "disposition": "kani_eligible",
                    "reason": "outside mapped subset",
                    "provenance": "kani_metadata::AutoHarnessMetadata::chosen",
                },
            ],
            "contracted_functions": [
                {
                    "crate_name": "destiny_original_spec",
                    "function": "model::assign_flag",
                    "file": "verification/spec/src/model.rs",
                    "harnesses": ["model::assign_flag_contract"],
                    "clauses": [
                        {"kind": "requires", "expression": "value"},
                        {"kind": "ensures", "expression": "|_| value"},
                    ],
                    "provenance": "kani_metadata::KaniMetadata::contracted_functions",
                }
            ],
        }

    def test_selected_unmapped_and_contract_candidates_are_distinguished(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            reg=self.write(root,"registry.json",self.registry())
            norm=self.write(root,"normalized.json",self.normalized())
            plan=self.write(root,"plan.json",{
                "obligations":[{"id":"state-contracts-reviewed-inventory"}]
            })
            result=bridge.bind_discovery(norm,reg,plan)
        self.assertEqual(result["status"],"BOUND_FOR_VERUS")
        classes={r["kani_metadata_name"]:r["classification"] for r in result["records"]}
        self.assertEqual(classes["model::assign_flag"],"selected")
        self.assertEqual(classes["model::apply_non_negative_setter"],"intentionally_excluded")
        self.assertFalse(result["kani_proof_credit"])
        self.assertEqual(result["contract_inventory_count"],1)
        self.assertEqual(
            result["contract_inventory"][0]["function"],
            "model::assign_flag",
        )
        self.assertFalse(result["contract_inventory"][0]["proof_credit"])
        self.assertEqual(result["contract_clause_count"],2)
        self.assertEqual(
            result["contract_inventory"][0]["clauses"],
            [
                {"kind":"requires","expression":"value"},
                {"kind":"ensures","expression":"|_| value"},
            ],
        )
        self.assertEqual(
            result["contract_semantics_status"],
            "compiler_parsed_clauses_available",
        )

    def test_configured_function_selection_scopes_expected_registry(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            registry=self.registry()
            registry["candidates"].append({
                "function":"destiny_original_spec::model::follow_allowed",
                "status":"supported",
                "reason":"Boolean equality",
                "controller_obligation":"proofs-reviewed-inventory",
                "verus_entry":"verification/verus/proofs.rs",
                "kani_metadata_name":"model::follow_allowed",
            })
            normalized=self.normalized()
            normalized["candidates"].append({
                "crate_name":"destiny_original_spec",
                "function":"model::follow_allowed",
                "disposition":"kani_eligible",
                "reason":"fixture",
                "provenance":"kani_metadata::AutoHarnessMetadata::chosen",
            })
            reg=self.write(root,"registry.json",registry)
            norm=self.write(root,"normalized.json",normalized)
            plan=self.write(root,"plan.json",{
                "obligations":[
                    {"id":"state-contracts-reviewed-inventory"},
                    {"id":"proofs-reviewed-inventory"},
                ]
            })
            result=bridge.bind_discovery(
                norm,
                reg,
                plan,
                requested_functions=["model::assign_flag"],
            )
        self.assertEqual(result["status"],"BOUND_FOR_VERUS")
        self.assertEqual(result["configured_functions"],["model::assign_flag"])
        self.assertEqual(
            result["controller_obligations"],
            ["state-contracts-reviewed-inventory"],
        )
        selected=[row for row in result["records"] if row["classification"]=="selected"]
        self.assertEqual(
            [row["kani_metadata_name"] for row in selected],
            ["model::assign_flag"],
        )

    def test_unknown_configured_function_is_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            reg=self.write(root,"registry.json",self.registry())
            norm=self.write(root,"normalized.json",self.normalized())
            plan=self.write(root,"plan.json",{
                "obligations":[{"id":"state-contracts-reviewed-inventory"}]
            })
            with self.assertRaises(bridge.BindingError):
                bridge.bind_discovery(
                    norm,
                    reg,
                    plan,
                    requested_functions=["model::not_registered"],
                )

    def test_duplicate_contract_inventory_is_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            reg=self.write(root,"registry.json",self.registry())
            normalized=self.normalized()
            normalized["contracted_functions"].append(
                dict(normalized["contracted_functions"][0])
            )
            norm=self.write(root,"normalized.json",normalized)
            plan=self.write(root,"plan.json",{
                "obligations":[{"id":"state-contracts-reviewed-inventory"}]
            })
            with self.assertRaises(bridge.BindingError):
                bridge.bind_discovery(norm,reg,plan)

    def test_required_supported_candidate_blocker_is_incomplete(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            reg=self.write(root,"registry.json",self.registry())
            norm=self.write(root,"normalized.json",self.normalized("blocked"))
            plan=self.write(root,"plan.json",{
                "obligations":[{"id":"state-contracts-reviewed-inventory"}]
            })
            result=bridge.bind_discovery(norm,reg,plan)
        self.assertEqual(result["status"],"INCOMPLETE")
        self.assertTrue(result["unresolved"])

    def test_empty_discovery_cannot_pass(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            reg=self.write(root,"registry.json",self.registry())
            norm=self.write(root,"normalized.json",{
                "default_engine":"verus",
                "proof_credit":False,
                "kani_source_commit":bridge.KANI_COMMIT,
                "candidates":[],
                "contracted_functions":[],
            })
            plan=self.write(root,"plan.json",{
                "obligations":[{"id":"state-contracts-reviewed-inventory"}]
            })
            result=bridge.bind_discovery(norm,reg,plan)
        self.assertEqual(result["status"],"INCOMPLETE")

    def test_campaign_requires_fresh_checked_receipts(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            binding=self.write(root,"binding-map.json",{
                "status":"BOUND_FOR_VERUS",
                "kani_proof_credit":False,
                "controller_obligations":["state-contracts-reviewed-inventory"],
                "contract_inventory_count":1,
                "contract_clause_count":2,
                "contract_semantics_status":"compiler_parsed_clauses_available",
            })
            campaign=root/"campaign"
            campaign.mkdir()
            self.write(campaign,"binding.json",{"status":"BOUND"})
            self.write(campaign,"audit.json",{"status":"CHECKED_SCOPED"})
            self.write(campaign,"state-contracts-reviewed-inventory.json",{"status":"CHECKED_SCOPED"})
            result=bridge.check_campaign(binding,campaign)
        self.assertEqual(result["status"],"KANI_SELECTED_VERUS_CHECKED_SCOPED")
        self.assertTrue(result["verus_checked_scoped"])
        self.assertFalse(result["kani_proof_credit"])
        self.assertEqual(result["contract_inventory_count"],1)
        self.assertEqual(result["contract_clause_count"],2)
        self.assertEqual(result["contract_semantics_status"],"compiler_parsed_clauses_available")


if __name__ == "__main__":
    unittest.main()
