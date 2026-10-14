#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification" / "tools" / "verus_kani.py"
spec = importlib.util.spec_from_file_location("destiny_verus_kani", MODULE)
vk = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = vk
assert spec.loader is not None
spec.loader.exec_module(vk)

class TranslatorTests(unittest.TestCase):
    def parse(self, text):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(text)
            return vk.parse_source(path)

    def test_branch_and_literal_loop_are_structural(self):
        functions = self.parse(
            "pub fn checked(x: bool, n: u8) {"
            "if x { assert!(x && n <= 255); } else { assert!(!x); }"
            "for _ in 0..3 { assert!(n >= 0); }}"
        )
        self.assertEqual([function.name for function in functions], ["checked"])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(
                "pub fn checked(x: bool) { for _ in 0..3 { assert!(x || !x); } }\n"
            )
            function = vk.parse_source(path)[0]
            lowered = vk.lower_function(function, path, Path(directory))
            self.assertEqual(lowered.count("assert("), 3)
            self.assertIn("exact literal range 0..3 unrolled", lowered)

    def test_each_parameter_remains_distinct(self):
        function = self.parse(
            "pub fn independent(a: u8, b: u8) { assert!(a == a && b == b); }\n"
        )[0]
        self.assertEqual(function.params, (("a", "u8"), ("b", "u8")))

    def test_arithmetic_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn no_arithmetic(x: u8) { assert!(x + 1 > x); }\n")

    def test_float_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn no_float(x: f64) { assert!(x == x); }\n")

    def test_dynamic_or_named_loop_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn no_dynamic(x: u8) { for i in 0..3 { assert!(x == x); } }\n")

    def test_generic_function_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn generic<T>(x: T) { assert!(true); }\n")

    def test_reference_memory_syntax_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn reference(x: &u8) { assert!(true); }\n")

    def test_assumption_syntax_is_rejected(self):
        with self.assertRaises(vk.Unsupported):
            self.parse("pub fn assumption(x: bool) { kani::assume(x); assert!(x); }\n")

    def test_kani_list_flatten_ignores_map_keys(self):
        data = {"a.rs": ["crate::one"], "b.rs": ["crate::two"]}
        self.assertEqual(vk.flatten_strings(data), ["crate::one", "crate::two"])

    def test_strict_json_rejects_duplicate_and_nonfinite_values(self):
        with self.assertRaises(ValueError):
            vk.strict_json('{"a": 1, "a": 2}')
        with self.assertRaises(ValueError):
            vk.strict_json('{"a": NaN}')

if __name__ == "__main__":
    unittest.main()
