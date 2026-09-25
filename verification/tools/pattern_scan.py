#!/usr/bin/env python3
"""Find repeated verification-relevant implementation patterns.

This is intentionally lexical: it is a regression detector, not a proof.  Its
output feeds the Kani metaverification harnesses, which prove that a reusable
macro/helper preserves the behavior of a representative handwritten pattern.
"""

from __future__ import annotations

import argparse
import collections
import json
import pathlib
import re

DEFAULT_PATHS = (
    "src/runtime.rs",
    "src/network.rs",
    "src/ffi.rs",
    "python/destiny/_facade.py",
    "python/destiny/_backend.py",
)

IGNORED = {"{", "}", "});", ");", "else {"}


def normalize(line: str) -> str:
    return re.sub(r"\s+", " ", line.strip())


def scan(root: pathlib.Path) -> list[dict[str, object]]:
    counts: collections.Counter[str] = collections.Counter()
    locations: dict[str, list[str]] = collections.defaultdict(list)

    for relative in DEFAULT_PATHS:
        path = root / relative
        if not path.exists():
            continue
        for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = normalize(raw)
            if len(line) < 24 or line in IGNORED or line.startswith(("//", "#")):
                continue
            counts[line] += 1
            if len(locations[line]) < 8:
                locations[line].append(f"{relative}:{number}")

    return [
        {"count": count, "pattern": pattern, "locations": locations[pattern]}
        for pattern, count in counts.most_common()
        if count >= 4
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).parents[2])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    report = scan(args.root)
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        for row in report:
            print(f"{row['count']:>4}  {row['pattern']}")
            for location in row["locations"][:3]:
                print(f"      {location}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
