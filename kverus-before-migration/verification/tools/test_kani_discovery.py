#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification/tools/run_kani_autoharness_discovery.py"
SPEC = importlib.util.spec_from_file_location("run_kani_autoharness_discovery", MODULE)
assert SPEC and SPEC.loader
discovery = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = discovery
SPEC.loader.exec_module(discovery)


class ClauseEvidenceTests(unittest.TestCase):
    def metadata(self, *functions: str):
        return [{
            "crate_name": "fixture_crate",
            "contracted_functions": [
                {"function": function, "file": "src/lib.rs", "harnesses": []}
                for function in functions
            ],
        }]

    def record(self, function: str, kind: str = "requires", expression: str = "x"):
        return {
            "path": "/tmp/clause",
            "sha256": "0" * 64,
            "crate_name": "fixture_crate",
            "function": function,
            "kind": kind,
            "expression": expression,
        }

    def test_binds_and_deduplicates_parser_evidence(self):
        metadata = self.metadata("module::contracted")
        record = self.record("contracted")
        discovery.bind_clause_evidence(metadata, [record, dict(record)])
        self.assertEqual(
            metadata[0]["contracted_functions"][0]["clauses"],
            [{"kind": "requires", "expression": "x"}],
        )

    def test_rejects_ambiguous_function_leaf(self):
        metadata = self.metadata("a::contracted", "b::contracted")
        with self.assertRaises(SystemExit):
            discovery.bind_clause_evidence(metadata, [self.record("contracted")])

    def test_rejects_contracted_function_without_parser_evidence(self):
        metadata = self.metadata("module::contracted")
        with self.assertRaises(SystemExit):
            discovery.bind_clause_evidence(metadata, [])

    def test_reads_versioned_parser_record(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "1.clause"
            path.write_text(
                "kverus_contract_clause_v1\nfixture-crate\ncontracted\nensures\n|r| *r == x",
                encoding="utf-8",
            )
            records = discovery.read_clause_evidence(Path(td))
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["crate_name"], "fixture_crate")
        self.assertEqual(records[0]["function"], "contracted")
        self.assertEqual(records[0]["kind"], "ensures")
        self.assertEqual(records[0]["expression"], "|r| *r == x")


if __name__ == "__main__":
    unittest.main()
