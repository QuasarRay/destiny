"""Smoke tests for the diagnostic inventory. These are NOT equivalence proofs."""
import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "destiny_equivalence_audit.py"
spec = importlib.util.spec_from_file_location("destiny_equivalence_audit", SCRIPT)
audit_module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(audit_module)


class InventoryTests(unittest.TestCase):
    def test_ignores_comments_and_finds_class_methods(self):
        text = audit_module.remove_comments(
            "// class Fake { void Phantom(); }\n"
            "class Real { public: void Advance(); int Value() const { return 1; } };"
        )
        classes = list(audit_module.CPP_TYPES.finditer(text))
        self.assertEqual([m.group(1) or m.group(2) for m in classes], ["Real"])
        body = audit_module.balanced_body(text, classes[0].end() - 1)
        self.assertEqual(set(audit_module.class_methods(body)), {"Advance", "Value"})

    def test_rust_field_parser_recognizes_comma_delimited_structs(self):
        source = """pub struct Box {
    pub mKey: Key,
    #[doc = "preserved"]
    pub mObject: Option<Py<PyAny>>,
    mParent: Option<Entity>,
}
"""
        actual = audit_module.rust_struct_fields(source)["Box"]
        self.assertEqual(actual, ["mKey", "mObject", "mParent"])

    def test_missing_source_and_missing_methods_are_detected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "src").mkdir()
            (root / "destiny-rs/src").mkdir(parents=True)
            (root / "src/Example.h").write_text(
                "class Example { public: void Match(); void Missing(); };"
            )
            (root / "src/Example.cpp").write_text("void Example::Missing() {}")
            (root / "destiny-rs/src/example.rs").write_text(
                "pub struct Example; impl Example { pub fn Match(&self) {} }"
            )
            registry = {"mappings": [{
                "cpp_type": "Example",
                "cpp_header": "src/Example.h",
                "cpp_files": ["src/Example.h"],
                "rust_type": "Example",
                "rust_file": "destiny-rs/src/example.rs",
                "verified_behavior": False,
                "verified_structure": False,
                "verified_internals": False,
            }]}
            result = audit_module.audit(root, registry)
            row = result["mappings"][0]
            self.assertTrue(row["exact_type_name"])
            self.assertEqual(row["exact_name_candidates"], ["Match"])
            self.assertEqual(row["missing_exact_method_names"], ["Missing"])
            self.assertEqual(result["unregistered_cpp_files"], ["src/Example.cpp"])
            self.assertFalse(result["strict_equivalence_passed"])


if __name__ == "__main__":
    unittest.main()
