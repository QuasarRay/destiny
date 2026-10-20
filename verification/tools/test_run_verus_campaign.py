#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification/tools/run_verus_campaign.py"
SPEC = importlib.util.spec_from_file_location("run_verus_campaign", MODULE)
assert SPEC and SPEC.loader
campaign = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = campaign
SPEC.loader.exec_module(campaign)


class CampaignPlanTests(unittest.TestCase):
    def plan(self):
        return {
            "schema": 1,
            "objective": "fixture",
            "obligations": [
                {"id": "a", "method": "verus", "entry": "a.rs"},
                {"id": "b", "method": "verus", "entry": "b.rs"},
            ],
        }

    def test_configured_subset_builds_exact_filtered_plan(self):
        selected = campaign.select_plan(self.plan(), ["b"])
        self.assertEqual(selected["configured_obligations"], ["b"])
        self.assertEqual(
            [row["id"] for row in selected["obligations"]],
            ["b"],
        )
        self.assertEqual(selected["objective"], "fixture")

    def test_unknown_and_duplicate_obligations_fail_closed(self):
        with self.assertRaises(ValueError):
            campaign.select_plan(self.plan(), ["missing"])
        with self.assertRaises(ValueError):
            campaign.select_plan(self.plan(), ["a", "a"])


if __name__ == "__main__":
    unittest.main()
