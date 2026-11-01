#!/usr/bin/env python3
"""Conservative C++/Rust naming and coverage inventory; NOT a semantics prover.

Never interpret the output of this tool as proof of behavioral, structural,
or internal implementation equivalence. C++ macros and the Rust proc-macro
expansion require separate AST/semantic analysis, differential tests and proofs.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from collections import Counter


ROOT = Path(__file__).resolve().parents[1]
CPP_TYPES = re.compile(
    r"\b(?:class|struct)\s+([A-Za-z_]\w*)\b[^;{}]*\{"
    r"|BLUE_CLASS\s*\(\s*([A-Za-z_]\w*)\s*\)\s*:[^;{}]*\{",
    re.S,
)
RUST_TYPES = re.compile(r"\b(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|trait)\s+([A-Za-z_]\w*)\b")
RUST_METHODS = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^()]*>)?\s*\(")
RUST_IMPLS = re.compile(
    r"\bimpl\s+(?:<[^{}]*>\s*)?(?:(?:[\w:<>]+)\s+for\s+)?([A-Za-z_]\w*)\s*\{"
)
RUST_STRUCTS = re.compile(
    r"\b(?:pub\s+)?struct\s+([A-Za-z_]\w*)\b[^;{}]*\{"
)
CPP_METHODS = re.compile(r"(~?[A-Za-z_]\w*|operator\s*(?:\(\)|[-+/*=<>!]+))\s*\(")
IGNORED = {"if", "for", "while", "switch", "return", "sizeof", "alignof", "decltype", "static_assert"}


def remove_comments(source: str) -> str:
    # Preserve line boundaries. This is a lexical approximation, NOT a C++ parser.
    pattern = re.compile(r"//[^\n]*|/\*[\s\S]*?\*/|\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])*'")
    def replace(match: re.Match[str]) -> str:
        text = match.group()
        return "".join("\n" if c == "\n" else " " for c in text)
    return pattern.sub(replace, source)


def balanced_body(text: str, opening: int) -> str | None:
    depth = 0
    for i in range(opening, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[opening + 1:i]
    return None


def class_members(body: str) -> list[str]:
    """Statements and inline-method signatures at direct class scope only."""
    result = []
    start = 0
    depth = 0
    for i, ch in enumerate(body):
        if ch == "{":
            if depth == 0:
                result.append(body[start:i].strip())
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                start = i + 1
        elif ch == ";" and depth == 0:
            result.append(body[start:i].strip())
            start = i + 1
    return result


def class_methods(body: str) -> list[str]:
    found = []
    for item in class_members(body):
        item = re.sub(r"\b(?:public|private|protected)\s*:\s*", " ", item)
        if "(" not in item or item.lstrip().startswith(("#", "typedef", "using")):
            continue
        matches = list(CPP_METHODS.finditer(item))
        if not matches:
            continue
        # C++ attributes/macros can contain parentheses before the signature.
        name = matches[-1].group(1).replace(" ", "")
        if name not in IGNORED and not name.startswith("BLUE_"):
            found.append(name)
    return found



def class_fields(body: str) -> list[str]:
    """Conservative direct-member scanner; ignores macro-generated fields."""
    fields = []
    for item in class_members(body):
        item = re.sub(r"\b(?:public|private|protected)\s*:\s*", " ", item)
        item = item.strip()
        if "(" in item or item.startswith(("#", "friend", "typedef", "using", "enum", "class", "struct")):
            continue
        before = item.split("=", 1)[0].split(":", 1)[0].strip()
        m = re.search(r"\b([A-Za-z_]\w*)\s*$", before)
        if m:
            fields.append(m.group(1))
    return fields


def rust_struct_fields(source: str) -> dict[str, list[str]]:
    """Read named Rust struct fields, which are COMMA-delimited, unlike C++.

    The supported declaration subset is intentionally narrow. This is still
    a lexical diagnostic rather than a Rust AST / HIR parser.
    """
    found = {}
    for match in RUST_STRUCTS.finditer(source):
        body = balanced_body(source, match.end() - 1)
        if body is None:
            continue
        fields = []
        for line in body.splitlines():
            line = line.split("//", 1)[0].strip()
            # Each named field in the current port starts on its own line.
            field = re.match(
                r"(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_]\w*)\s*:",
                line,
            )
            if field:
                fields.append(field.group(1))
        found[match.group(1)] = sorted(set(fields))
    return found


def rust_impl_methods(source: str) -> dict[str, list[str]]:
    found = {}
    for match in RUST_IMPLS.finditer(source):
        body = balanced_body(source, match.end() - 1)
        if body is not None:
            found.setdefault(match.group(1), []).extend(RUST_METHODS.findall(body))
    return {k: sorted(set(v)) for k, v in found.items()}


def source_inventory(root: Path) -> dict:
    cpp_dir = root / "src"
    files = sorted([p.relative_to(root).as_posix() for p in cpp_dir.glob("*")
                    if p.suffix in {".h", ".cpp"}])
    declarations: dict[str, list[dict]] = {}
    for rel in files:
        if not rel.endswith(".h"):
            continue
        text = remove_comments((root / rel).read_text(encoding="utf-8", errors="replace"))
        for match in CPP_TYPES.finditer(text):
            name = match.group(1) or match.group(2)
            body = balanced_body(text, match.end() - 1)
            if body is None:
                continue
            declarations.setdefault(name, []).append({
                "file": rel,
                "methods": class_methods(body),
                "fields": class_fields(body),
            })
    return {"files": files, "types": declarations}


def rust_inventory(root: Path) -> dict:
    files = sorted((root / "destiny-rs/src").rglob("*.rs"))
    results = {}
    for file in files:
        text = file.read_text(encoding="utf-8")
        results[file.relative_to(root).as_posix()] = {
            "types": sorted(set(RUST_TYPES.findall(text))),
            "functions": sorted(set(RUST_METHODS.findall(text))),
            "methods_by_type": rust_impl_methods(text),
            "fields_by_type": rust_struct_fields(text),
        }
    return results


def original_test_sources(root: Path) -> list[str]:
    python = sorted((root / "python/destiny/test").rglob("test_*.py"))
    cpp = sorted((root / "tests").glob("Test*.cpp"))
    if (root / "tests/destinyTest.cpp").exists():
        cpp.append(root / "tests/destinyTest.cpp")
    return sorted(p.relative_to(root).as_posix() for p in python + cpp)


def audit(root: Path, registry: dict) -> dict:
    cpp = source_inventory(root)
    rust = rust_inventory(root)
    records = []
    source_tests = original_test_sources(root)
    test_mappings = registry.get("test_mappings", [])
    registered_tests = set()
    for mapping in test_mappings:
        original = mapping["cpp_or_python_test"]
        if original not in source_tests:
            raise ValueError(f"Unknown original test file: {original}")
        if not (root / mapping["rust_test"]).is_file():
            raise ValueError(f"Missing Rust counterpart test file: {mapping['rust_test']}")
        registered_tests.add(original)
    missing_test_files = sorted(set(source_tests) - registered_tests)
    covered_files = set()
    registered_types = set()
    for mapping in registry["mappings"]:
        original = mapping["cpp_type"]
        target = mapping["rust_type"]
        header = mapping["cpp_header"]
        path = mapping["rust_file"]
        if header not in cpp["files"]:
            raise ValueError(f"Missing C++ source: {header}")
        if path not in rust:
            raise ValueError(f"Missing Rust source: {path}")
        declarations = [d for d in cpp["types"].get(original, []) if d["file"] == header]
        if not declarations:
            raise ValueError(f"Missing C++ declaration: {original} in {header}")
        registered_types.add(original)
        covered_files.update(mapping["cpp_files"])
        missing_cpp_files = [p for p in mapping["cpp_files"] if p not in cpp["files"]]
        if missing_cpp_files:
            raise ValueError(f"Missing declared C++ files: {missing_cpp_files}")
        declared = sorted(set(m for d in declarations for m in d["methods"]))
        declared_fields = sorted(set(field for d in declarations for field in d["fields"]))
        rust_fns = set(rust[path]["methods_by_type"].get(target, []))
        rust_fields = set(rust[path]["fields_by_type"].get(target, []))
        candidates = sorted(set(declared) & rust_fns)
        missing = sorted(set(declared) - rust_fns)
        candidate_fields = sorted(set(declared_fields) & rust_fields)
        missing_fields = sorted(set(declared_fields) - rust_fields)
        records.append({
            "cpp_type": original,
            "cpp_header": header,
            "rust_type": target,
            "rust_file": path,
            "cpp_methods_detected": declared,
            "cpp_fields_detected": declared_fields,
            "exact_field_name_candidates": candidate_fields,
            "missing_exact_field_names": missing_fields,
            "exact_name_candidates": candidates,
            "missing_exact_method_names": missing,
            "exact_type_name": original == target and target in rust[path]["types"],
            "verified_behavior": mapping.get("verified_behavior", False),
            "verified_structure": mapping.get("verified_structure", False),
            "verified_internals": mapping.get("verified_internals", False),
            "notes": mapping.get("notes", ""),
        })
    missing_files = sorted(set(cpp["files"]) - covered_files)
    missing_types = sorted(set(cpp["types"]) - registered_types)
    blockers = []
    if missing_test_files:
        blockers.append(f"{len(missing_test_files)} original test files lack correspondence records")
    if missing_files:
        blockers.append(f"{len(missing_files)} C++ source/header files not registered")
    if missing_types:
        blockers.append(f"{len(missing_types)} C++ declared types not registered")
    for record in records:
        name = record["cpp_type"]
        if not record["exact_type_name"]:
            blockers.append(f"{name}: type name not preserved")
        if record["missing_exact_field_names"]:
            blockers.append(f"{name}: {len(record['missing_exact_field_names'])} C++ field names absent from target type")
        if record["missing_exact_method_names"]:
            blockers.append(f"{name}: {len(record['missing_exact_method_names'])} declared method names absent from target file")
        if not all(record[k] for k in ("verified_behavior", "verified_structure", "verified_internals")):
            blockers.append(f"{name}: behavioral/structural/internal equivalence is unverified")
    return {
        "schema": 1,
        "warning": "Lexical source inventory, not an AST or semantic equivalence proof.",
        "cpp_file_count": len(cpp["files"]),
        "original_test_files": source_tests,
        "unregistered_original_test_files": missing_test_files,
        "cpp_header_count": sum(p.endswith(".h") for p in cpp["files"]),
        "cpp_implementation_count": sum(p.endswith(".cpp") for p in cpp["files"]),
        "rust_source_count": len(rust),
        "registered_mappings": len(records),
        "unregistered_cpp_files": missing_files,
        "unregistered_cpp_types": missing_types,
        "mappings": records,
        "strict_equivalence_passed": not blockers,
        "blockers": blockers,
    }


def markdown(report: dict) -> str:
    lines = [
        "## Destiny C++ ↔ Rust correspondence audit",
        "",
        "**This is NOT a proof of equivalence.** The scanner compares direct field names and method *ownership* lexically, and",
        "does not compare semantics, field layouts, concurrency or compiler expansions.",
        "",
        f"- Original C++ files: **{report['cpp_file_count']}** "
        f"({report['cpp_header_count']} headers; "
        f"{report['cpp_implementation_count']} implementations)",
        f"- Destiny Rust source files: **{report['rust_source_count']}**",
        f"- Explicitly registered type mappings: **{report['registered_mappings']}**",
        f"- Unregistered C++ files: **{len(report['unregistered_cpp_files'])}**",
        f"- Unregistered C++ types: **{len(report['unregistered_cpp_types'])}**",
        f"- Original test files needing verified correspondence: **{len(report['unregistered_original_test_files'])}**",
        f"- Strict equivalence gate: **{'PASS' if report['strict_equivalence_passed'] else 'FAIL'}**",
        "",
        "| C++ type | Rust type | Same type name? | Matched methods | Missing methods | Matched fields | Missing fields | Semantics proven? |",
        "|---|---|---|---:|---:|---:|---:|---|",
    ]
    for r in report["mappings"]:
        lines.append(
            f"| {r['cpp_type']} | {r['rust_type']} | "
            f"{'Yes' if r['exact_type_name'] else 'No'} | "
            f"{len(r['exact_name_candidates'])} | "
            f"{len(r['missing_exact_method_names'])} | "
            f"{len(r['exact_field_name_candidates'])} | "
            f"{len(r['missing_exact_field_names'])} | "
            f"{'Yes' if r['verified_behavior'] else 'No'} |"
        )
    if report["unregistered_original_test_files"]:
        lines.extend(["", "### Unregistered original test files", "",
                      ", ".join("`" + p + "`" for p in report["unregistered_original_test_files"])])
    if report["unregistered_cpp_files"]:
        lines.extend(["", "### Unregistered C++ files", "",
                      ", ".join("`" + p + "`" for p in report["unregistered_cpp_files"])])
    if report["blockers"]:
        lines.extend(["", "### Equivalence blockers", ""])
        lines.extend(f"- {item}" for item in report["blockers"])
    lines.extend(["", "Passing Cargo tests does not make this audit pass.", ""])
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path,
                        default=ROOT / "destiny-rs/equivalence/mappings.json")
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--markdown-out", type=Path)
    parser.add_argument("--strict", action="store_true",
                        help="Fail unless every registered and unregistered gap is resolved")
    args = parser.parse_args()
    try:
        result = audit(ROOT, json.loads(args.registry.read_text(encoding="utf-8")))
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        parser.exit(2, f"Audit input error: {error}\n")
    result_json = json.dumps(result, indent=2, sort_keys=True) + "\n"
    result_markdown = markdown(result)
    if args.json_out:
        args.json_out.parent.mkdir(parents=True, exist_ok=True)
        args.json_out.write_text(result_json, encoding="utf-8")
    if args.markdown_out:
        args.markdown_out.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_out.write_text(result_markdown, encoding="utf-8")
    print(result_markdown)
    if args.strict and not result["strict_equivalence_passed"]:
        print("\nStrict equivalence gate FAILS: full parity is not established.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
