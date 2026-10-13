#!/usr/bin/env python3
"""Differential observation of the declared CPython decimal-rounding partition.

This does not establish a universal theorem or native Destiny correspondence.
The Rust library is built from the actual locked specification manifest. The
oracle executes the host's Python round rather than duplicating the Rust rule.
"""
import argparse
import ast
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import platform
import random
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
REVISION = "ef738406b61ad06bae2f402857e464735ab106a2"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    build = subprocess.run(
        ["cargo", "build", "--manifest-path", "verification/spec/Cargo.toml",
         "--locked", "--offline", "--message-format=json"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    artifacts = []
    for line in build.stdout.splitlines():
        event = json.loads(line)
        if event.get("reason") == "compiler-artifact" and event["target"]["name"] == "destiny_original_spec":
            artifacts.extend(Path(p) for p in event["filenames"] if p.endswith(".rlib"))
    if len(artifacts) != 1:
        raise RuntimeError("Expected exactly the current specification library artifact")
    library = artifacts[0]
    dependencies = library.parent / "deps" if (library.parent / "deps").is_dir() else library.parent
    source = r'''
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line=line.expect("corpus input");
        let mut columns=line.split_whitespace();
        let bits:u64=columns.next().expect("bits").parse().expect("uint64");
        let places:u32=columns.next().expect("places").parse().expect("uint32");
        match destiny_original_spec::python_decimal_round_key(f64::from_bits(bits),places) {
            Some(key)=>println!("{key}"), None=>println!("OUTSIDE_SCOPE"),
        }
    }
}
'''
    values = [0.0, -0.0, math.ulp(0.0), -math.ulp(0.0), sys.float_info.min,
              -sys.float_info.min, 2.675, -2.675, 2.625, -2.625,
              67_108_864.0, -67_108_864.0]
    for path in ("test_stop.py", "test_goto.py"):
        original = subprocess.run(
            ["git", "show", f"{REVISION}:python/destiny/test/ballpark/evolve/{path}"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
        values.extend(n.value for n in ast.walk(ast.parse(original))
                      if isinstance(n, ast.Constant) and isinstance(n.value, float))
    corpus = []
    rng = random.Random(0xD3571)
    for places in range(7):
        selected = list(values)
        for integer in (0, 1, 2, 3, 9, 12, 34_567, 6_700_000):
            boundary = (integer + 0.5) / 10**places
            for sign in (-1, 1):
                value = sign * boundary
                selected.extend((math.nextafter(value, -math.inf), value,
                                 math.nextafter(value, math.inf)))
        for _ in range(256):
            # Draw bit patterns, including normal and subnormal magnitudes.
            bits = rng.getrandbits(52) | (rng.randrange(1050) << 52) | (rng.randrange(2) << 63)
            selected.append(struct.unpack("!d", struct.pack("!Q", bits))[0])
        selected.extend((math.nan, math.inf, -math.inf, 67_108_865.0))
        for value in selected:
            bits = struct.unpack("!Q", struct.pack("!d", value))[0]
            expected = None
            if math.isfinite(value) and abs(value) <= 67_108_864.0:
                # The Python numeric result has <=6 fractional places in this
                # partition. Its shortest decimal representation recovers its
                # unique decimal integer; no custom tie rule is used here.
                decimal_key = Decimal(str(round(value, places))) * 10**places
                if decimal_key != decimal_key.to_integral_value():
                    raise RuntimeError("Python oracle left the reviewed decimal-key partition")
                expected = str(int(decimal_key))
            corpus.append({"bits": bits, "places": places, "expected": expected})
    with tempfile.TemporaryDirectory(prefix="destiny-rounding-", dir=ROOT / ".aegis/destiny") as tmp:
        tmp = Path(tmp)
        entry, binary = tmp / "rounding.rs", tmp / "rounding"
        entry.write_text(source)
        subprocess.run(
            ["rustc", "--edition=2024", str(entry), "--extern",
             f"destiny_original_spec={library}", "-L", f"dependency={dependencies}",
             "-o", str(binary)], cwd=ROOT, check=True,
        )
        observed = subprocess.run([str(binary)], check=True, capture_output=True, text=True,
                                  input="".join(f"{r['bits']} {r['places']}\n" for r in corpus)).stdout.splitlines()
    if len(observed) != len(corpus):
        raise RuntimeError("Incomplete Rust oracle output")
    failures = []
    for record, actual in zip(corpus, observed):
        record["actual"] = None if actual == "OUTSIDE_SCOPE" else actual
        if record["actual"] != record["expected"]:
            failures.append(record)
    report = {
        "schema": 1, "status": "FAILED" if failures else "OBSERVED_SUCCESS",
        "claim": "finite differential corpus only; no formal proof credit",
        "python": platform.python_version(), "python_executable": sys.executable,
        "spec_source_sha256": hashlib.sha256((ROOT / "verification/spec/src/python_assertions.rs").read_bytes()).hexdigest(),
        "partition": "finite abs(value)<=2^26; places 0..6; CPython correct decimal conversion path",
        "count": len(corpus), "failures": failures, "corpus": corpus,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in ("status", "python", "count", "claim")}))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
