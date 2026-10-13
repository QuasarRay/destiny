#!/usr/bin/env python3
"""Record pinned assertion/setup sites using Aegis extraction, without proof credit.

The extractor is advisory: every record begins unreviewed. In particular imported
assertion helpers, dynamic dispatch and C++ macros still require source review.
Original Python/C++ code is read from local immutable Git objects, never executed.
"""
import argparse
import ast
import hashlib
import json
from functools import lru_cache
from pathlib import Path
import re
import sys
import textwrap


def generate(root, controller):
    sys.path.insert(0, str(controller / "infra"))
    from agentinfra.catalog import cpp_case, original_text, python_case, qualified_index

    rows = qualified_index(root)
    @lru_cache(maxsize=64)
    def read(path):
        return original_text(root, rows[0]["original_commit"], path)

    cases = []
    assertions, dependencies, imports = {}, {}, {}
    for row in rows:
        record = {key: row[key] for key in (
            "id", "source", "fixture", "test", "line", "original_source_sha256")}
        record.update(assertion_ids=[], dependency_ids=[], extraction_status="UNREVIEWED")
        try:
            packet = (python_case if row["source"].endswith(".py") else cpp_case)(
                row["source"], row["test"], row["fixture"], read)
            assert packet["status"] == "EXTRACTED", packet["status"]
            sites = {}
            for snippet in packet["snippets"]:
                path, start, content = snippet["path"], snippet["start"], snippet["text"]
                if path.endswith(".py"):
                    parsed = ast.parse(textwrap.dedent(content))
                    nodes = [n for n in ast.walk(parsed) if isinstance(n, ast.Assert) or (
                        isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute)
                        and (n.func.attr.startswith("assert") or n.func.attr == "fail"))]
                    locations = [(start + n.lineno - 1, ast.unparse(n)) for n in nodes]
                else:
                    locations = [(start + content.count("\n", 0, match.start()), match[0])
                                 for match in re.finditer(r'\b(?:ASSERT|EXPECT)_\w+\s*\(', content)]
                for line, expression in locations:
                    identity = f"{path}:{line}"
                    sites[identity] = {"id": identity, "source": path, "line": line,
                                       "source_sha256": packet["original_files"][path],
                                       "expression": expression}
                if snippet["purpose"] != "original case":
                    dependency = {
                        "id": f"{path}:{start}:{snippet['end']}", "source": path,
                        "start": start, "end": snippet["end"], "purpose": snippet["purpose"],
                        "source_sha256": packet["original_files"][path],
                        "snippet_sha256": hashlib.sha256(content.encode()).hexdigest()}
                    dependencies[dependency["id"]] = dependency
                    record["dependency_ids"].append(dependency["id"])
            assertions.update(sites)
            record["assertion_ids"] = sorted(sites)
            imports.update(packet["imports"])
            record["import_sources"] = sorted(packet["imports"])
            record["unresolved_dependencies"] = packet["dependency_review"]
            record["no_unexpected_exception_required"] = True
        except (ValueError, AssertionError, KeyError, SyntaxError) as error:
            record["extraction_status"] = "BLOCKED"
            record["unresolved_dependencies"] = [str(error)]
        cases.append(record)
    assert len(cases) == len({c["id"] for c in cases}) == 531
    assert len({(c["source"], c["test"]) for c in cases}) == 509
    assert len({c["source"] for c in cases}) == 34
    return {"schema": 1, "original_commit": rows[0]["original_commit"],
            "catalog_sha256": hashlib.sha256((root / "verification/spec/src/catalog.rs").read_bytes()).hexdigest(),
            "reader_sha256": hashlib.sha256((controller / "infra/agentinfra/catalog.py").read_bytes()).hexdigest(),
            "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "notice": "Static assertion/dependency inventory only; review and executable evidence remain required.",
            "assertions": assertions, "dependencies": dependencies, "imports": imports,
            "cases": cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--controller", type=Path)
    parser.add_argument("--output", type=Path, default=Path("verification/cases/inventory.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    document = generate(root, (args.controller or root / ".agents").resolve())
    path = args.output if args.output.is_absolute() else root / args.output
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"cases": len(document["cases"]), "path": str(path),
                      "assertion_sites_with_case_context": sum(len(c["assertion_ids"]) for c in document["cases"]),
                      "distinct_assertion_sites": len(document["assertions"]),
                      "blocked_extractions": sum(c["extraction_status"] == "BLOCKED" for c in document["cases"])}))


if __name__ == "__main__":
    main()
