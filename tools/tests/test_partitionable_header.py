"""Source-level correspondence guard for Partitionable.h / Partitionable.cpp.

This proves declaration existence, not end-to-end behavioral equivalence.
"""
import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
HEADER = (ROOT / "src/Partitionable.h").read_text(encoding="utf-8")
CPP = (ROOT / "src/Partitionable.cpp").read_text(encoding="utf-8")
RUST = (ROOT / "destiny-rs/src/partitionable.rs").read_text(encoding="utf-8")

FIELDS = ["mId", "mHandled", "mNewPos", "mOldPos", "mNewBubble",
          "mOldBubble", "isInteractive", "mBoxes", "mMyBox", "mPark",
          "mEffectStamp", "isMoribund"]
METHODS = ["ClearBoxes", "DeleteFromBoxes", "InsertInBox",
           "GetBoundingRadius", "GetInflatedRadius", "SetBoxActive",
           "InsertInBoxes"]

class PartitionableHeaderContract(unittest.TestCase):
    def test_exact_cpp_class_constructor_fields(self):
        self.assertRegex(HEADER, r"\bclass\s+Partitionable\b")
        self.assertRegex(CPP, r"Partitionable::Partitionable\s*\(\s*\)")
        self.assertRegex(RUST, r"\bpub\s+struct\s+Partitionable\s*\{")
        self.assertRegex(RUST, r"\bpub\s+fn\s+new\s*\(")
        for field in FIELDS:
            self.assertRegex(HEADER, r"\b" + field + r"\s*;")
            self.assertRegex(RUST, r"\bpub\s+" + field + r"\s*:")
        self.assertEqual(len(FIELDS), 12)

    def test_named_id_and_sorted_set_aliases(self):
        self.assertIn("typedef int64_t ID", HEADER)
        self.assertIn("pub type ID = i64;", RUST)
        self.assertIn("SortedSetOfBoxes mBoxes", HEADER)
        self.assertIn("pub type SortedSetOfBoxes", RUST)
        self.assertIn("pub mBoxes: SortedSetOfBoxes", RUST)

    def test_original_seven_virtual_methods_are_required_and_owner_exposed(self):
        self.assertEqual(HEADER.count(" = 0;"), 7)
        for method in METHODS:
            self.assertRegex(
                HEADER,
                r"\bvirtual\s+\w+\s+" + method + r"\s*\([^;]*\)\s*=\s*0\s*;"
            )
            # Public Partitionable owner and the derived dispatch trait
            self.assertRegex(RUST, r"\bpub\s+fn\s+" + method + r"\s*<T:\s*PartitionableBehavior>")
            self.assertRegex(RUST, r"\bfn\s+" + method + r"\s*\(")
        self.assertNotIn("impl PartitionableBehavior for Partitionable", RUST)

    def test_all_original_constructor_initialized_fields_match(self):
        init = re.search(r"Partitionable::Partitionable\s*\(\s*\)\s*:(.*?)\{", CPP, re.S)
        self.assertIsNotNone(init)
        values = dict(re.findall(r"(m\w+|is\w+)\s*\(\s*([^)]*)\s*\)", init.group(1)))
        expected = {
            "mId": "0", "mMyBox": "nullptr", "mPark": "nullptr",
            "mHandled": "false", "mNewBubble": "-1", "mOldBubble": "-1",
            "isInteractive": "false", "mEffectStamp": "0", "isMoribund": "false",
        }
        self.assertEqual(values, expected)

if __name__ == "__main__":
    unittest.main()
