"""Header-level QuaternionD declaration correspondence guard.

This checks that all original declarations still have Rust declarations.
It does not prove functional equivalence or IEEE-754 bitwise equality.
"""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
CPP = ROOT / "src/Quaternion.h"
RUST = ROOT / "destiny-rs/src/quaternion.rs"

class QuaternionHeaderContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cpp = CPP.read_text(encoding="utf-8")
        cls.rust = RUST.read_text(encoding="utf-8")

    def test_original_header_defines_exactly_one_quaternion_value_type(self):
        self.assertEqual(len(re.findall(r"\btypedef\s+struct\s+QuaternionD\b", self.cpp)), 1)
        self.assertRegex(self.rust, r"\bpub\s+struct\s+QuaternionD\s*\{")
        self.assertIn("#[repr(C)]", self.rust)
        for original, replacement in [("double w;", "pub w: f64"),
                                      ("Vector3d v;", "pub v: DVec3")]:
            self.assertIn(original, self.cpp)
            self.assertIn(replacement, self.rust)

    def test_four_cpp_constructors_have_exactly_four_rust_routes(self):
        cpp_ctors = re.findall(r"^\s*QuaternionD\s*\(", self.cpp, re.M)
        self.assertEqual(len(cpp_ctors), 4)
        self.assertIn("impl Default for QuaternionD", self.rust)
        for name in ("new", "from_scalar_vector", "from_axis_angle"):
            self.assertRegex(self.rust, r"\b(?:const\s+)?fn\s+" + name + r"\s*\(")

    def test_all_nine_cpp_operator_overloads_have_rust_traits(self):
        cpp_operators = re.findall(r"\boperator\s*([+\-*/=!]+)\s*\(", self.cpp)
        self.assertEqual(len(cpp_operators), 9)
        self.assertEqual(sorted(cpp_operators), sorted(["+=", "-=", "-", "/", "*", "+", "-", "==", "!="]))
        for trait in ("AddAssign", "SubAssign", "Neg", "Div<f64>", "Mul",
                      "Add", "Sub", "PartialEq"):
            self.assertIn("impl " + trait + " for QuaternionD", self.rust)
        self.assertRegex(self.rust, r"\bfn\s+eq\s*\(")
        self.assertRegex(self.rust, r"\bfn\s+ne\s*\(")

    def test_named_methods_preserve_original_names_and_signatures(self):
        self.assertRegex(self.cpp, r"\bdouble\s+LengthSq\s*\(\s*\)\s*const")
        self.assertRegex(self.cpp, r"\bVector3d\s+RotateVector\s*\(")
        self.assertRegex(self.rust, r"pub\s+fn\s+LengthSq\s*\(\s*&self\s*\)")
        self.assertRegex(self.rust, r"pub\s+fn\s+RotateVector\s*\(\s*&self\s*,\s*vector\s*:\s*&DVec3")
        self.assertNotRegex(self.rust, r"\bpub\s+fn\s+(?:inverse|rotate_vector|length_squared)\s*\(")

    def test_no_new_ecs_object_model_for_cpp_plain_value_type(self):
        self.assertNotIn("#[derive(Component", self.rust)
        self.assertNotIn("impl Plugin", self.rust)
        self.assertNotIn("impl Resource", self.rust)

if __name__ == "__main__":
    unittest.main()
