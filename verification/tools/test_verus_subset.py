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

    def test_selected_primitive_generic_instance_is_explicitly_substituted(self):
        function = self.parse("pub fn generic<T>(x: T) { assert!(true); }\n")[0]
        selected = vk.instantiate_selected_function(function, "crate::generic::<i32>")
        self.assertEqual(selected.params, (("x", "i32"),))
        self.assertEqual(selected.selected_generic_args, (("T", "i32"),))
        with self.assertRaises(vk.Unsupported):
            vk.instantiate_selected_function(function, "crate::generic::<usize>")

    def test_generic_bounds_and_reference_syntax_are_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn bounded<T: Copy>(x: T) { assert!(true); }\n")
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
        witnesses = vk.collect_assumption_witnesses(function.body)
        self.assertEqual(witnesses[0].prefix_conditions, ())
        self.assertEqual(witnesses[0].conditions, ("x",))
        self.assertEqual(witnesses[1].prefix_conditions, ("x",))
        self.assertEqual(witnesses[1].conditions, ("x", "!(x)"))
        lowered = vk.lower_nonvacuity(function, witnesses)
        self.assertIn(
            "(exists|x: bool| (x)) ==> (exists|x: bool| (x) && (!(x)))",
            lowered,
        )

    def test_assumption_nonvacuity_is_scoped_to_contract_precondition(self):
        function = self.parse(
            "pub fn contracted(x: bool, y: bool) { kani::assume(y); assert!(y); }\n"
        )[0]
        witnesses = vk.collect_assumption_witnesses(function.body)
        lowered = vk.lower_nonvacuity(function, witnesses, (vk.Expr("x"),))
        self.assertIn(
            "(exists|x: bool, y: bool| (x)) ==> "
            "(exists|x: bool, y: bool| (x) && (y))",
            lowered,
        )

    def test_unreachable_assertion_branch_is_not_a_hard_nonvacuity_requirement(self):
        function = self.parse(
            "pub fn guarded(x: u8) { "
            "if x == 255 { assert!(true); } else { "
            "if x + 1 > x { assert!(true); } else { assert!(false); } } }\n"
        )[0]
        self.assertEqual(vk.collect_assumption_witnesses(function.body), [])
        self.assertEqual(len(vk.collect_assertion_paths(function.body)), 3)

    def test_assumption_reachability_depending_on_machine_arithmetic_is_rejected(self):
        function = self.parse(
            "pub fn arithmetic_assume(x: u8) { "
            "if x + 1 > x { kani::assume(true); assert!(true); } }\n"
        )[0]
        with self.assertRaises(vk.Unsupported):
            vk.collect_assumption_witnesses(function.body)

    def test_contract_attributes_are_ignored_by_source_parser_not_used_as_semantics(self):
        function = self.parse(
            "#[kani::requires(x)]\n"
            "#[kani::ensures(|_| x)]\n"
            "pub fn contracted(x: bool) { assert!(x); }\n"
        )[0]
        self.assertEqual(function.name, "contracted")
        self.assertEqual(function.params, (("x", "bool"),))

    def test_compiler_contract_metadata_lowers_to_verus_requires_and_ensures(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text(
                "#[kani::requires(x)]\n"
                "#[kani::ensures(|_| x)]\n"
                "pub fn contracted(x: bool) { assert!(x); }\n"
            )
            function = vk.parse_source(source)[0]
            discovery = root / "discovery.json"
            discovery.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "contracted",
                    "file": "src/lib.rs",
                    "harnesses": ["contracted"],
                    "clauses": [
                        {"kind": "requires", "expression": "x"},
                        {"kind": "ensures", "expression": "| _ | x"},
                    ],
                }],
            }))
            contract = vk.contract_for_function(discovery, "contracted", function)
            lowered = vk.lower_function(function, source, root, contract)
        self.assertEqual([expr.text for expr in contract.requires], ["x"])
        self.assertEqual([expr.text for expr in contract.ensures], ["x"])
        self.assertIn("    requires\n        x,", lowered)
        self.assertIn("    ensures\n        x,", lowered)

    def test_scalar_return_tail_is_preserved_as_executable_verus_return(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text("pub fn identity(x: u8) -> u8 { x }\n")
            function = vk.parse_source(source)[0]
            lowered = vk.lower_function(function, source, root)
        self.assertEqual(function.return_type, "u8")
        self.assertEqual(function.body[-1], vk.ReturnStmt(vk.Expr("x"), 1))
        self.assertIn("fn identity(x: u8) -> u8 {", lowered)
        self.assertIn("return x;", lowered)

    def test_scalar_result_contract_uses_named_verus_result(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text(
                "#[kani::requires(true)]\n"
                "#[kani::ensures(|result: &u8| *result == x)]\n"
                "pub fn identity(x: u8) -> u8 { x }\n"
            )
            function = vk.parse_source(source)[0]
            discovery = root / "discovery.json"
            discovery.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "identity",
                    "file": "src/lib.rs",
                    "harnesses": ["identity"],
                    "clauses": [
                        {"kind": "requires", "expression": "true"},
                        {
                            "kind": "ensures",
                            "expression": "| result : & u8 | * result == x",
                        },
                    ],
                }],
            }))
            contract = vk.contract_for_function(discovery, "identity", function)
            lowered = vk.lower_function(function, source, root, contract)
        self.assertEqual(function.return_type, "u8")
        self.assertEqual(
            [expr.text for expr in contract.ensures],
            ["(__destiny_result == x)"],
        )
        self.assertIn(
            "fn identity(x: u8) -> (__destiny_result: u8)",
            lowered,
        )
        self.assertIn("        (__destiny_result == x),", lowered)
        self.assertIn("return x;", lowered)

    def test_scalar_result_contract_rejects_wrong_result_reference_type(self):
        function = self.parse("pub fn identity(x: u8) -> u8 { x }\n")[0]
        with tempfile.TemporaryDirectory() as directory:
            discovery = Path(directory) / "discovery.json"
            discovery.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "identity",
                    "file": "src/lib.rs",
                    "harnesses": [],
                    "clauses": [{
                        "kind": "ensures",
                        "expression": "|result: &u16| *result == 0",
                    }],
                }],
            }))
            with self.assertRaises(vk.Unsupported):
                vk.contract_for_function(discovery, "identity", function)

    def test_scalar_result_contract_arithmetic_remains_rejected(self):
        function = self.parse("pub fn identity(x: u8) -> u8 { x }\n")[0]
        with tempfile.TemporaryDirectory() as directory:
            discovery = Path(directory) / "discovery.json"
            discovery.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "identity",
                    "file": "src/lib.rs",
                    "harnesses": [],
                    "clauses": [{
                        "kind": "ensures",
                        "expression": "|result: &u8| *result + 1 > x",
                    }],
                }],
            }))
            with self.assertRaises(vk.Unsupported):
                vk.contract_for_function(discovery, "identity", function)

    def test_selected_generic_return_type_is_substituted(self):
        function = self.parse("pub fn generic_return<T>(x: T) -> T { x }\n")[0]
        selected = vk.instantiate_selected_function(
            function,
            "crate::generic_return::<i32>",
        )
        self.assertEqual(selected.params, (("x", "i32"),))
        self.assertEqual(selected.return_type, "i32")

    def test_contract_requires_receive_explicit_nonvacuity_proof(self):
        function = self.parse("pub fn contracted(x: bool) { assert!(true); }\n")[0]
        contract = vk.ContractSpec(
            (("requires", "x"), ("requires", "! x")),
            (vk.Expr("x"), vk.Expr("!(x)")),
            (),
        )
        lowered = vk.lower_nonvacuity(function, (), contract.requires)
        self.assertIn(
            "assert(exists|x: bool| (x) && (!(x)));",
            lowered,
        )

    def test_contract_arithmetic_and_modifies_are_rejected(self):
        function = self.parse("pub fn contracted(x: u8) { assert!(true); }\n")[0]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            arithmetic = root / "arithmetic.json"
            arithmetic.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "contracted",
                    "file": "src/lib.rs",
                    "harnesses": [],
                    "clauses": [{"kind": "requires", "expression": "x + 1 > x"}],
                }],
            }))
            with self.assertRaises(vk.Unsupported):
                vk.contract_for_function(arithmetic, "contracted", function)

            modifies = root / "modifies.json"
            modifies.write_text(json.dumps({
                "contracted_functions": [{
                    "function": "contracted",
                    "file": "src/lib.rs",
                    "harnesses": [],
                    "clauses": [{"kind": "modifies", "expression": "x"}],
                }],
            }))
            with self.assertRaises(vk.Unsupported):
                vk.contract_for_function(modifies, "contracted", function)

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
