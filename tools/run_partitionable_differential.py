#!/usr/bin/env python3
"""Compare the original C++ Partitionable constructor to Rust/Bevy.

This extracts the real source initializer list from Partitionable.cpp;
the C++ test harness supplies only a concrete derived witness because the
original Partitionable type has seven pure virtual methods. Does NOT prove
virtual-derived implementations or object-layout equivalence.
"""
from pathlib import Path
import math
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "target"
SOURCE = ROOT / "src/Partitionable.cpp"
TEMPLATE = ROOT / "tools/partitionable_constructor_oracle.cpp"
GENERATED = TARGET / "cpp-partitionable-constructor.generated.cpp"
BINARY = TARGET / "cpp-partitionable-constructor"

def call(command):
    return subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True).stdout

def parse(output):
    entries = {}
    for line in output.strip().splitlines():
        parts = line.split()
        if parts[0] in entries:
            raise ValueError(f"Duplicate fixture: {parts[0]}")
        entries[parts[0]] = tuple(float(v) for v in parts[1:])
    return entries

def main():
    cpp_source = SOURCE.read_text(encoding="utf-8")
    match = re.search(
        r"Partitionable::Partitionable\s*\(\s*\)\s*:"
        r"[\s\S]*?\{\s*\}", cpp_source
    )
    if not match:
        raise RuntimeError("Original Partitionable constructor body not found")
    template = TEMPLATE.read_text(encoding="utf-8")
    if template.count("// __ORIGINAL_PARTITIONABLE_CONSTRUCTOR__") != 1:
        raise RuntimeError("Missing unique source injection marker")
    TARGET.mkdir(parents=True, exist_ok=True)
    GENERATED.write_text(
        template.replace("// __ORIGINAL_PARTITIONABLE_CONSTRUCTOR__", match.group(0)),
        encoding="utf-8"
    )
    call(["c++", "-std=c++17", "-O0", "-I.", str(GENERATED), "-o", str(BINARY)])
    original = parse(call([str(BINARY)]))
    port = parse(call(["cargo", "run", "--locked", "--quiet", "-p", "destiny-rs",
                       "--example", "partitionable_constructor_oracle"]))
    if original.keys() != port.keys():
        raise AssertionError(f"Fixture set differs: {original.keys()} vs {port.keys()}")
    for name in original:
        if len(original[name]) != len(port[name]):
            raise AssertionError(f"Field count differs: {name}")
        for pos, (a, b) in enumerate(zip(original[name], port[name])):
            if not math.isfinite(a) or not math.isfinite(b) or a != b:
                raise AssertionError(
                    f"Constructor mismatch: {name}[{pos}], C++={a}, Rust={b}"
                )
    print(f"Verified {sum(len(v) for v in original.values())} original "
          f"Partitionable constructor field outputs in {len(original)} fixtures")
    return 0

if __name__ == "__main__":
    sys.exit(main())
