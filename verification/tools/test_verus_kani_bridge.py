from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

MODULE_PATH = Path(__file__).with_name("verus_kani_bridge.py")
SPEC = importlib.util.spec_from_file_location("verus_kani_bridge", MODULE_PATH)
assert SPEC and SPEC.loader
bridge = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bridge)


class MetadataTests(unittest.TestCase):
    def test_kani_chosen_and_skipped_are_not_conflated(self) -> None:
        candidates = [
            {
                "function": "destiny_original_spec::model::assign_flag",
                "controller_obligation": "state-contracts-reviewed-inventory",
                "verus_entry": "verification/verus/state_contracts.rs",
                "reason": "Boolean identity",
            },
            {
                "function": "destiny_original_spec::model::follow_allowed",
                "controller_obligation": "proofs-reviewed-inventory",
                "verus_entry": "verification/verus/proofs.rs",
                "reason": "Boolean/equality",
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = {
                "crate_name": "destiny_original_spec",
                "proof_harnesses": [],
                "unsupported_features": [],
                "test_harnesses": [],
                "contracted_functions": [],
                "autoharness_md": {
                    "chosen": ["destiny_original_spec::model::assign_flag"],
                    "skipped": {
                        "destiny_original_spec::model::follow_allowed": {
                            "GenericFn": "fixture blocker"
                        }
                    },
                },
            }
            (root / "fixture.kani-metadata.json").write_text(json.dumps(payload))
            result = bridge.collect_autoharness_metadata(root, candidates)

        by_name = {item["function"]: item for item in result["records"]}
        self.assertEqual(by_name["destiny_original_spec::model::assign_flag"]["status"], "selected")
        self.assertEqual(by_name["destiny_original_spec::model::follow_allowed"]["status"], "blocked")
        self.assertEqual(
            by_name["destiny_original_spec::model::follow_allowed"]["reason"]["kind"],
            "GenericFn",
        )

    def test_missing_candidate_is_blocked_not_success(self) -> None:
        candidates = [
            {
                "function": "destiny_original_spec::model::assign_flag",
                "controller_obligation": "state-contracts-reviewed-inventory",
                "verus_entry": "verification/verus/state_contracts.rs",
                "reason": "Boolean identity",
            }
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = {
                "crate_name": "destiny_original_spec",
                "proof_harnesses": [],
                "unsupported_features": [],
                "test_harnesses": [],
                "contracted_functions": [],
                "autoharness_md": {"chosen": [], "skipped": {}},
            }
            (root / "fixture.kani-metadata.json").write_text(json.dumps(payload))
            result = bridge.collect_autoharness_metadata(root, candidates)

        self.assertEqual(result["records"][0]["status"], "blocked")
        self.assertEqual(
            result["records"][0]["reason"]["kind"],
            "MissingFromKaniAutoharnessMetadata",
        )

    def test_exact_pattern_does_not_broaden_selection(self) -> None:
        pattern = bridge.exact_pattern("destiny_original_spec::model::assign_flag")
        self.assertEqual(pattern, r"^destiny_original_spec::model::assign_flag$")


if __name__ == "__main__":
    unittest.main()
