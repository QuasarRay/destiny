#!/usr/bin/env python3
"""Prepare reviewed, existing Copy-value scenarios for strict Kani execution.

Equality keeps exactly the same operands and PartialEq relation. The direct
boolean form avoids unreachable Debug-formatting assertions in assert_eq's
panic-only path. This is deliberately limited to the reviewed proof files.
"""
import json
from pathlib import Path
import re


def direct_equalities(text):
    while (match := re.search(r'assert_eq!\(', text)) is not None:
        start = match.end()
        depth, end = 1, start
        separators = []
        while depth:
            char = text[end]
            if char in "([{":
                depth += 1
            elif char in ")]}":
                depth -= 1
            elif char == "," and depth == 1:
                separators.append(end)
            # Reviewed arguments contain no string/debug messages or literals.
            assert char != '"', "unreviewed equality argument"
            end += 1
        assert 1 <= len(separators) <= 2
        left = text[start:separators[0]].strip()
        right = text[separators[0] + 1:separators[1] if len(separators) == 2 else end - 1].strip()
        if len(separators) == 2:
            assert not text[separators[1] + 1:end - 1].strip()
        text = text[:match.start()] + f"assert!({left} == {right})" + text[end:]
    return text


def main():
    root = Path(__file__).resolve().parents[2]
    inventory = json.loads((root / "verification/cases/inventory.json").read_text())
    cases = {f"{c['source']}::{c['test']}": c['id'] for c in inventory["cases"] if c["source"].endswith(".cpp")}
    geometry = root / "verification/kani/src/geometry_contracts.rs"
    text = geometry.read_text()
    pattern = re.compile(r'// original-test: ([^\n]+)\n(?:[^\n]*\n)*?fn (\w+)\(\) \{(.*?)^\}', re.M | re.S)
    matches = list(pattern.finditer(text))
    assert len(matches) == 36, "the reviewed geometry inventory changed"
    for match in reversed(matches):
        identity = cases[match[1]]
        body = direct_equalities(match[3])
        if "kani::cover!" not in body:
            body += '    #[cfg(kani)]\n    kani::cover!(true, "exact original geometry scenario completed");\n'
        replacement = (f"// original-test: {match[1]}\n// original-case: {identity}\n"
                       f"#[cfg_attr(kani, kani::proof)]\n#[cfg_attr(not(kani), test)]\nfn {match[2]}() {{{body}}}")
        text = text[:match.start()] + replacement + text[match.end():]
    geometry.write_text(text)
    for name in ("formation_refinement.rs", "formation_contracts.rs"):
        path = root / "verification/kani/src" / name
        path.write_text(direct_equalities(path.read_text()))
    print("Prepared 36 reviewed geometry scenarios and direct formation equalities; no proof was run.")


if __name__ == "__main__":
    main()
