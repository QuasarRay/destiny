"""Enforce Settings.h / Settings_Blue.cpp declaration correspondence.

Lexical API guard, NOT formal behavior verification.
"""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
HEADER = (ROOT / "src/Settings.h").read_text(encoding="utf-8")
BLUE = (ROOT / "src/Settings_Blue.cpp").read_text(encoding="utf-8")
RUST = (ROOT / "destiny-rs/src/settings.rs").read_text(encoding="utf-8")

DEFAULTS = [
    ("collisionMaxIterations", "usize", "size_t", "20"),
    ("useIterativeCollision", "bool", "bool", "false"),
    ("useDynamicalOrientation", "bool", "bool", "false"),
    ("disableDynamicalOrientationForMissiles", "bool", "bool", "false"),
    ("useNewOrbit", "bool", "bool", "false"),
]
METHODS = ["ApplyConfiguration", "GetConfiguration",
           "GetDefaultConfiguration", "ResetConfiguration"]
PY_METHODS = ["Apply", "Get", "GetDefault", "Reset"]

class SettingsHeaderContract(unittest.TestCase):
    def test_only_two_original_object_types_are_authoritative(self):
        for name in ("SettingsConfiguration", "GlobalSettings"):
            self.assertRegex(HEADER, r"BLUE_CLASS\s*\(\s*" + name + r"\s*\)")
            self.assertRegex(RUST, r"\bpub\s+struct\s+" + name + r"\b")
        self.assertNotIn("struct SettingsBridge", RUST)
        self.assertNotIn("trait CanReadDestinySettings", RUST)
        self.assertNotIn("trait CanApplyDestinySettings", RUST)

    def test_five_exact_default_names_global_locations_and_fields(self):
        for name, rust_ty, cpp_ty, literal in DEFAULTS:
            self.assertIn("g_" + name + "Default", HEADER)
            self.assertIn("g_" + name + "Default", RUST)
            self.assertRegex(HEADER, r"\bextern\s+" + cpp_ty + r"\s+g_" + name + r"\s*;")
            self.assertRegex(RUST, r"\bpub\s+static\s+g_" + name + r"\s*:")
            self.assertRegex(HEADER, r"\b" + cpp_ty + r"\s+m_" + name + r"\s*=")
            self.assertRegex(RUST, r"\bpub\s+m_" + name + r"\s*:\s*" + rust_ty + r"\b")
            self.assertIn("#[blue(read, write, name = \"" + name + "\")]", RUST)

    def test_four_original_method_names_belong_to_globalsettings(self):
        for method in METHODS:
            self.assertRegex(HEADER, r"PyObject\s*\*\s*" + method + r"\s*\(")
            self.assertRegex(RUST, r"\bpub\s+fn\s+" + method + r"\s*\(")
        self.assertEqual(len(re.findall(r"\bpub\s+fn\s+(?:ApplyConfiguration|GetConfiguration|GetDefaultConfiguration|ResetConfiguration)\s*\(", RUST)), 4)
        self.assertIn("pub static s_globalSettings", RUST)

    def test_blue_exports_keep_all_original_python_names(self):
        for method, python in zip(METHODS, PY_METHODS):
            self.assertIn('"' + python + '", ' + method, BLUE)
            self.assertIn('#[blue(name = "' + python + '")]', RUST)
        for name, _, _, _ in DEFAULTS:
            self.assertIn('"' + name + '"', BLUE)
        self.assertIn('module.add("settings", shared)', RUST)

    def test_no_deferred_write_or_per_world_authority(self):
        self.assertNotIn("import_python_requests", RUST)
        self.assertNotIn("publish_bevy_state", RUST)
        self.assertNotIn("take_pending", RUST)
        self.assertIn("g_collisionMaxIterations.store", RUST)
        self.assertIn("synchronize_global_settings_to_bevy", RUST)

if __name__ == "__main__":
    unittest.main()
