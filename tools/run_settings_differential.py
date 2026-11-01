#!/usr/bin/env python3
"""Run the original Settings.cpp implementation against Destiny-rs Settings.

The mini Blue ABI only substitutes the unavailable CCP runtime machinery.
Source: original src/Settings.cpp, not a reimplementation of its methods.
This is bounded differential evidence, not a universal semantics proof.
"""
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
EXE = ROOT / "target/cpp-settings-oracle"

def execute(cmd):
    return subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True).stdout

def parse(output):
    cases = {}
    for line in output.strip().splitlines():
        fields = line.split()
        if len(fields) != 6:
            raise ValueError("Expected case name plus five settings fields: " + line)
        name, *values = fields
        if name in cases:
            raise ValueError("Duplicate settings fixture: " + name)
        cases[name] = tuple(int(v) for v in values)
    return cases

def main():
    EXE.parent.mkdir(parents=True, exist_ok=True)
    execute(["c++", "-std=c++17", "-O0", "-I.", "tools/cpp_settings_oracle.cpp", "-o", str(EXE)])
    cpp = parse(execute([str(EXE)]))
    rust = parse(execute(["cargo", "run", "--quiet", "--locked", "-p", "destiny-rs",
                          "--example", "settings_cpp_oracle"]))
    if cpp != rust:
        print("C++:", cpp)
        print("Rust:", rust)
        raise AssertionError("Original Settings.cpp behavior differs from Rust")
    print("Compared", len(cpp), "original Settings.cpp / Rust configuration scenarios")
    return 0

if __name__ == "__main__":
    sys.exit(main())
