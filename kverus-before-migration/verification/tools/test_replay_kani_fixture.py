#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "verification/tools/replay_kani_fixture.py"
SPEC = importlib.util.spec_from_file_location("replay_kani_fixture", MODULE)
assert SPEC and SPEC.loader
replay = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = replay
SPEC.loader.exec_module(replay)


class ReplayConstructionTests(unittest.TestCase):
    def test_temporary_harness_imports_original_source_by_path(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "original.rs"
            source.write_text("pub fn bad(x: bool) { assert!(x); }\n")
            function = replay.subset.parse_source(source)[0]
            text = replay.build_harness_source(source.resolve(), function)
        self.assertIn(f'#[path = "{source.resolve()}"]', text)
        self.assertIn("let x: bool = kani::any();", text)
        self.assertIn("original_source::bad(x);", text)
        self.assertNotIn(source.read_text() if source.exists() else "pub fn bad", text)

    def test_harness_preserves_independent_parameter_calls(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "original.rs"
            source.write_text("pub fn bad(a: u8, b: u8) { assert!(a == b); }\n")
            function = replay.subset.parse_source(source)[0]
            text = replay.build_harness_source(source.resolve(), function)
        self.assertIn("let a: u8 = kani::any();", text)
        self.assertIn("let b: u8 = kani::any();", text)
        self.assertIn("original_source::bad(a, b);", text)


if __name__ == "__main__":
    unittest.main()
