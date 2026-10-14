#!/usr/bin/env python3
"""Execute exact catalog identities from unchanged pinned Python tests on native.

This produces development boundary observations, never a formal proof receipt.
Every selected test and every imported original Python module is byte-checked
against the behavioral Git objects. A skipped or empty run fails acceptance.
"""
import argparse
import functools
import hashlib
import importlib
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
PIN = "ef738406b61ad06bae2f402857e464735ab106a2"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def original_bytes(source):
    return subprocess.run(["git", "--no-replace-objects", "show", PIN + ":" + source],
                          cwd=ROOT, capture_output=True, check=True).stdout


class CaseResults(unittest.TextTestResult):
    def __init__(self, *args, native_counter, **kwargs):
        super().__init__(*args, **kwargs)
        self.observations = []
        self.native_counter = native_counter
        self.native_start = {}

    def startTest(self, test):
        self.native_start[test.id()] = self.native_counter()
        super().startTest(test)

    def record(self, test, status):
        self.observations.append({"test": test.id(), "status": status,
                                  "native_calls": self.native_counter() - self.native_start[test.id()]})

    def addSuccess(self, test):
        super().addSuccess(test)
        self.record(test, "PASSED")

    def addFailure(self, test, error):
        super().addFailure(test, error)
        self.record(test, "FAILED")

    def addError(self, test, error):
        super().addError(test, error)
        self.record(test, "ERROR")

    def addSkip(self, test, reason):
        super().addSkip(test, reason)
        self.record(test, "SKIPPED")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original-root", required=True, type=Path)
    parser.add_argument("--library", required=True, type=Path)
    parser.add_argument("--case", action="append", required=True)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    original_root = args.original_root.resolve()
    library = args.library.resolve()
    report = args.report.resolve()
    if not report.is_relative_to(ROOT / ".aegis"):
        raise ValueError("Runtime reports must be kept under .aegis")
    if not library.is_file():
        raise ValueError("An actual native library is required")
    inventory = json.loads((ROOT / "verification/cases/inventory.json").read_text())
    catalog = {case["id"] for case in inventory["cases"]}
    if not 1 <= len(args.case) <= 32 or len(set(args.case)) != len(args.case):
        raise ValueError("Select 1..32 distinct exact catalog cases")
    sources = {}
    names = []
    for identity in args.case:
        if identity not in catalog:
            raise ValueError("Unknown qualified catalog identity: " + identity)
        source, fixture, test = identity.split("::")
        if not source.startswith("python/destiny/test/") or not source.endswith(".py"):
            raise ValueError("This runner executes original Python tests only")
        path = original_root / source
        if path.read_bytes() != original_bytes(source):
            raise ValueError("Original source differs: " + source)
        sources[source] = sha(path)
        names.append(source[len("python/"):-3].replace("/", ".") + "." + fixture + "." + test)
    sys.path.insert(0, str(ROOT / "python"))
    import destiny
    from destiny._backend import NativeBackend
    destiny.__path__.append(str(original_root / "python/destiny"))
    backend = NativeBackend(library)
    # Count actual ABI entry calls, including fixture setup/teardown. Merely
    # loading a library cannot turn pure Python settings/geometry execution
    # into native differential evidence. Wrappers forward the exact arguments
    # to the unchanged configured ctypes functions, retaining their lifetimes.
    native_calls = [0]
    for entry in ("dbc_runtime_new", "dbc_runtime_new_with_config", "dbc_runtime_call",
                  "dbc_runtime_call_with_mode_callback", "dbc_runtime_call_with_event_callback",
                  "dbc_runtime_update", "dbc_runtime_update_with_mode_callback", "dbc_runtime_update_with_event_callback"):
        if hasattr(backend._library, entry):
            function = getattr(backend._library, entry)
            def counted(*arguments, _function=function):
                native_calls[0] += 1
                return _function(*arguments)
            setattr(backend._library, entry, counted)
    destiny.set_backend(backend)
    result = None
    try:
        suite = unittest.TestSuite()
        for name in names:
            module_name, fixture, test = name.rsplit(".", 2)
            cls = getattr(importlib.import_module(module_name), fixture)
            case = cls(test)
            if case.id() != name:
                raise ValueError("Fixture discovery selected an unexpected identity")
            suite.addTest(case)
        if suite.countTestCases() != len(names):
            raise ValueError("Empty or incomplete original test selection")
        # Validate helper modules before any scenario, then again after runtime
        # imports. This also rejects edited originals loaded through helpers.
        def validate_original_imports():
            for module in list(sys.modules.values()):
                file = getattr(module, "__file__", None)
                if not file:
                    continue
                path = Path(file).resolve()
                if path.suffix == ".py" and path.is_relative_to(original_root):
                    source = path.relative_to(original_root).as_posix()
                    if path.read_bytes() != original_bytes(source):
                        raise ValueError("Imported original dependency differs: " + source)
                    sources[source] = sha(path)
        validate_original_imports()
        result = unittest.TextTestRunner(verbosity=2, resultclass=functools.partial(
            CaseResults, native_counter=lambda: native_calls[0])).run(suite)
        validate_original_imports()
        observed = {entry["test"] for entry in result.observations}
        scenarios_passed = (result.wasSuccessful() and result.testsRun == len(names)
                    and not result.skipped and observed == set(names))
        native_execution_complete = all(entry["native_calls"] > 0 for entry in result.observations)
        accepted = scenarios_passed and native_execution_complete
        document = {
            "schema": 1, "kind": "native_development_observation",
            "original_commit": PIN, "library": str(library), "library_sha256": sha(library),
            "qualified_cases": args.case, "selected": len(names), "executed": result.testsRun,
            "skipped": len(result.skipped), "passed": sum(entry["status"] == "PASSED" for entry in result.observations),
            "accepted": accepted, "scenarios_passed": scenarios_passed,
            "native_execution_complete": native_execution_complete,
            "observations": [dict(entry, qualified_case=dict(zip(names,args.case)).get(entry["test"]))
                                                    for entry in result.observations],
            "original_sources": sources,
            "production_python_sources": {
                Path(module.__file__).resolve().relative_to(ROOT).as_posix(): sha(Path(module.__file__).resolve())
                for module in list(sys.modules.values())
                if getattr(module, "__file__", None)
                and Path(module.__file__).suffix == ".py"
                and Path(module.__file__).resolve().is_relative_to(ROOT / "python")
            },
            "scope": "Unchanged concrete Python assertions; native acceptance additionally requires observed ABI calls for every scenario. Artifact-to-source rebuild binding and universal formal correspondence remain required.",
        }
        report.parent.mkdir(parents=True, exist_ok=True)
        report.write_text(json.dumps(document, indent=2) + "\n")
        return 0 if accepted else 1
    finally:
        backend.close()


if __name__ == "__main__":
    raise SystemExit(main())
