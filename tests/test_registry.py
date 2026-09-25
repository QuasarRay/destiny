from __future__ import annotations

import json
import importlib.util
import sys
import unittest
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry" / "title_mapping.json"
IMPLEMENTATION_STATUS = ROOT / "registry" / "implementation_status.json"


class RegistryContractTests(unittest.TestCase):
    def test_registry_is_complete_and_preserves_mapping_counts(self) -> None:
        payload = json.loads(REGISTRY.read_text(encoding="utf-8"))
        records = payload["records"]

        self.assertEqual(len(records), 389)
        self.assertEqual(
            Counter(record["verdict"] for record in records),
            {
                "COMPOSED": 27,
                "DIRECT": 54,
                "NONE": 158,
                "PARTIAL": 144,
                "UNDETERMINED": 6,
            },
        )
        self.assertEqual(sum(bool(record["mapping_items"]) for record in records), 225)

    def test_every_registry_record_has_a_stable_dispatch_contract(self) -> None:
        payload = json.loads(REGISTRY.read_text(encoding="utf-8"))
        seen: set[str] = set()

        for record in payload["records"]:
            self.assertNotIn(record["canonical_title"], seen)
            seen.add(record["canonical_title"])
            self.assertTrue(record["title_id"])
            self.assertTrue(record["item_kind"])
            self.assertTrue(record["category"])
            self.assertIsInstance(record["material_difference"], str)
            for item in record["mapping_items"]:
                self.assertTrue(item["api_title"])
                self.assertIn(item["ecosystem"], {"Bevy", "Avian", "Lightyear", "Replicon", "Aeronet"})
                self.assertTrue(item["version"])

    def test_generated_python_registry_matches_json_registry(self) -> None:
        sys.path.insert(0, str(ROOT / "python"))
        try:
            from destiny._registry import RECORDS
        finally:
            sys.path.pop(0)

        payload = json.loads(REGISTRY.read_text(encoding="utf-8"))
        self.assertEqual(set(RECORDS), {record["canonical_title"] for record in payload["records"]})

    def test_implementation_status_is_complete_and_not_mapping_coverage(self) -> None:
        mapping = json.loads(REGISTRY.read_text(encoding="utf-8"))
        status = json.loads(IMPLEMENTATION_STATUS.read_text(encoding="utf-8"))
        self.assertTrue(status["mapping_and_implementation_are_independent"])
        self.assertEqual(sum(status["counts"].values()), 389)
        self.assertEqual(len(status["records"]), 389)
        self.assertEqual(
            {record["canonical_title"] for record in status["records"]},
            {record["canonical_title"] for record in mapping["records"]},
        )
        self.assertGreater(status["counts"]["unsupported"], 0)

    def test_implementation_status_is_reachability_derived_and_reproducible(self) -> None:
        script = ROOT / "tools" / "generate_implementation_status.py"
        specification = importlib.util.spec_from_file_location("implementation_status_generator", script)
        self.assertIsNotNone(specification)
        self.assertIsNotNone(specification.loader)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)

        committed = json.loads(IMPLEMENTATION_STATUS.read_text(encoding="utf-8"))
        self.assertEqual(module.generate_payload(), committed)
        park_row = next(
            row for row in committed["records"]
            if row["canonical_title"] == "destiny.Ballpark.GetBoxCenter"
        )
        self.assertIn("rust-native", park_row["surfaces"])


if __name__ == "__main__":
    unittest.main()
