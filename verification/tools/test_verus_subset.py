#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification/tools/verus_subset.py"
SPEC = importlib.util.spec_from_file_location("verus_subset", MODULE)
assert SPEC and SPEC.loader
vk = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = vk
SPEC.loader.exec_module(vk)


class TranslatorTests(unittest.TestCase):
    def parse(self, text):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(text)
            return vk.parse_source(path)

    def test_branch_and_literal_loop_are_structural(self):
        function = self.parse(
            "pub fn checked(x: bool) {"
            "for _ in 0..3 { if x { assert!(x); } else { assert!(!x); } }}"
        )[0]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(
                "pub fn checked(x: bool) {"
                "for _ in 0..3 { if x { assert!(x); } else { assert!(!x); } }}\n"
            )
            lowered = vk.lower_function(vk.parse_source(path)[0], path, Path(directory))
        self.assertEqual(lowered.count("assert("), 6)
        self.assertIn("exact literal range 0..3 exhaustively unrolled", lowered)

    def test_each_parameter_remains_distinct_and_independent(self):
        function = self.parse(
            "pub fn independent(a: u8, b: u8) { assert!(a == a && b == b); }\n"
        )[0]
        self.assertEqual(function.params, (("a", "u8"), ("b", "u8")))

    def test_ghost_arithmetic_is_rejected_instead_of_widened(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn arithmetic(x: u8) { assert!(x + 1 > x); }\n")
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn assumed(x: u8) { kani::assume(x + 1 > x); assert!(true); }\n")

    def test_machine_arithmetic_is_preserved_in_executable_branch_conditions(self):
        function = self.parse(
            "pub fn arithmetic(x: u8) { "
            "if x + 1 > x { assert!(true); } else { assert!(true); } }\n"
        )[0]
        self.assertEqual(function.body[0].cond.text, "((x + 1) > x)")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(
                "pub fn arithmetic(x: u8) { "
                "if x + 1 > x { assert!(true); } else { assert!(true); } }\n"
            )
            lowered = vk.lower_function(vk.parse_source(path)[0], path, Path(directory))
        self.assertIn("if ((x + 1) > x) {", lowered)

    def test_float_is_rejected_instead_of_real_substitution(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn floaty(x: f64) { assert!(x == x); }\n")

    def test_target_width_types_are_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn width(x: usize) { assert!(x == x); }\n")
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn signed_width(x: isize) { assert!(x == x); }\n")

    def test_dynamic_or_named_loop_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn no_dynamic(x: u8) { for i in 0..3 { assert!(x == x); } }\n")

    def test_generic_and_reference_syntax_are_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn generic<T>(x: T) { assert!(true); }\n")
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn reference(x: &u8) { assert!(true); }\n")

    def test_assumptions_are_scoped_and_recorded(self):
        function = self.parse(
            "pub fn assumption(x: bool, y: bool) { "
            "if x { kani::assume(y); assert!(y); } else { assert!(!x); } }\n"
        )[0]
        self.assertEqual(
            vk.collect_assumptions(function.body),
            [{"line": 1, "expression": "y"}],
        )
        paths = vk.collect_assertion_paths(function.body)
        self.assertEqual(len(paths), 2)
        self.assertIn(("x", "y"), [row.conditions for row in paths])
        self.assertIn(("!(x)",), [row.conditions for row in paths])

    def test_contradictory_assumption_gets_explicit_nonvacuity_obligation(self):
        function = self.parse(
            "pub fn contradiction(x: bool) { "
            "kani::assume(x); kani::assume(!x); assert!(x); }\n"
        )[0]
        paths = vk.collect_assertion_paths(function.body)
        self.assertEqual(paths[0].conditions, ("x", "!(x)"))
        lowered = vk.lower_nonvacuity(function, paths)
        self.assertIn("exists|x: bool| (x) && (!(x))", lowered)

    def test_discovery_requires_exact_pinned_kani_identity_and_no_proof_credit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            discovery = root / "discovery.json"
            discovery.write_text(json.dumps({
                "default_engine": "verus",
                "proof_credit": False,
                "kani_source_commit": vk.KANI_COMMIT,
                "candidates": [{
                    "function": "branch_valid",
                    "disposition": "kani_eligible",
                    "reason": "fixture",
                }],
            }))
            row = vk.require_kani_selection(discovery, "branch_valid")
        self.assertEqual(row["disposition"], "kani_eligible")

    def test_selected_unsupported_candidate_does_not_become_success(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            discovery = root / "discovery.json"
            discovery.write_text(json.dumps({
                "default_engine": "verus",
                "proof_credit": False,
                "kani_source_commit": vk.KANI_COMMIT,
                "candidates": [{
                    "function": "float_unsupported",
                    "disposition": "unsupported",
                    "reason": "fixture blocker",
                }],
            }))
            with self.assertRaises(vk.Unsupported):
                vk.require_kani_selection(discovery, "float_unsupported")


if __name__ == "__main__":
    unittest.main()
