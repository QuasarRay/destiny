#!/usr/bin/env python3
"""Execute actual C++ Destiny headers against the Rust port (finite fixtures).

This does not prove universal equivalence; it is one differential test corpus.
"""
from __future__ import annotations
import math
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "target/cpp-equivalence-oracle"


def run(args: list[str]) -> str:
    result = subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True)
    return result.stdout


def parse(text: str) -> dict[str, tuple[float, ...]]:
    cases = {}
    for line in text.splitlines():
        tokens = line.split()
        if len(tokens) != 5:
            raise ValueError(f"Unexpected oracle line: {line!r}")
        name = tokens[0]
        if name in cases:
            raise ValueError(f"Duplicate oracle case {name}")
        if name == "key_hash":
            cases[name] = (int(tokens[1]), 0, 0, 0)
        else:
            cases[name] = tuple(float(x) for x in tokens[1:])
    return cases


def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    run(["c++", "-std=c++17", "-O0", "-ffp-contract=off",
         "-I.", "tools/cpp_destiny_oracle.cpp", "-o", str(OUT)])
    cpp = parse(run([str(OUT)]))
    rust = parse(run(["cargo", "run", "--quiet", "--locked",
                      "-p", "destiny-rs", "--example", "cpp_oracle"]))
    if not cpp or cpp.keys() != rust.keys():
        raise AssertionError(f"Oracle cases differ: C++={sorted(cpp)} Rust={sorted(rust)}")
    failures = []
    for case in cpp:
        for i, (a, b) in enumerate(zip(cpp[case], rust[case])):
            if case == "key_hash":
                if a != b:
                    failures.append((case, i, a, b, "exact integer hash mismatch"))
            elif not math.isfinite(a) or not math.isfinite(b):
                failures.append((case, i, a, b, "non-finite output"))
            elif not math.isclose(a, b, rel_tol=1e-13, abs_tol=1e-13):
                failures.append((case, i, a, b, "numeric mismatch"))
    print(f"Compared {len(cpp)} original-C++/Rust arithmetic fixtures")
    for item in failures:
        print("FAILED:", *item)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
