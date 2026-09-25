#!/usr/bin/env python3
"""Report original Destiny tests represented by Kani and Verus proof markers."""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

FILE_RE = re.compile(r'file!\("([^"]+)"\s*=>\s*\[(.*?)\]\)', re.S)
STRING_RE = re.compile(r'"([^"]+)"')
MARKER_PREFIX = "// original-test: "


def catalog(root: pathlib.Path) -> set[tuple[str, str]]:
    text = (root / "verification/spec/src/catalog.rs").read_text(encoding="utf-8")
    result: set[tuple[str, str]] = set()
    for source, body in FILE_RE.findall(text):
        for test in STRING_RE.findall(body):
            result.add((source, test))
    return result


def markers(root: pathlib.Path, relative: str) -> set[tuple[str, str]]:
    base = root / relative
    result: set[tuple[str, str]] = set()
    if not base.exists():
        return result

    for path in base.rglob("*.rs"):
        for raw_line in path.read_text(encoding="utf-8").splitlines():
            line = raw_line.strip()
            if not line.startswith(MARKER_PREFIX):
                continue

            source_test = line[len(MARKER_PREFIX):]
            source, separator, test = source_test.rpartition("::")
            if not separator or not source or not test:
                raise ValueError(
                    f"malformed original-test marker in {path}: {raw_line!r}"
                )
            result.add((source, test))

    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).parents[2],
    )
    parser.add_argument("--strict-both", action="store_true")
    args = parser.parse_args()

    all_tests = catalog(args.root)
    kani = markers(args.root, "verification/kani")
    verus = markers(args.root, "verification/verus")

    unknown = (kani | verus) - all_tests
    if unknown:
        print("proof markers not present in original catalog:", file=sys.stderr)
        for source, test in sorted(unknown):
            print(f"  {source}::{test}", file=sys.stderr)
        return 2

    both = all_tests & kani & verus
    print(f"original obligations: {len(all_tests)}")
    print(f"Kani-marked:          {len(all_tests & kani)}")
    print(f"Verus-marked:         {len(all_tests & verus)}")
    print(f"marked in both:       {len(both)}")
    print(f"missing both:         {len(all_tests - (kani | verus))}")

    if args.strict_both and both != all_tests:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
