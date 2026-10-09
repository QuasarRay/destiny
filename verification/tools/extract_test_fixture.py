#!/usr/bin/env python3
"""Extract a literal original-test fixture without executing the source file."""

import argparse
import ast
import hashlib
import json
from pathlib import Path
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--symbol", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    commit = subprocess.check_output(
        ["git", "-C", str(args.repository), "rev-parse", "HEAD"], text=True
    ).strip()
    source = subprocess.check_output(
        ["git", "-C", str(args.repository), "show", f"{commit}:{args.source}"]
    )
    candidates = [
        node.value
        for node in ast.parse(source).body
        if isinstance(node, ast.Assign)
        and any(isinstance(target, ast.Name) and target.id == args.symbol for target in node.targets)
    ]
    if len(candidates) != 1:
        parser.error(f"expected one top-level literal assignment for {args.symbol}")
    fixture = {
        "source": args.source,
        "commit": commit,
        "source_sha256": hashlib.sha256(source).hexdigest(),
        "symbol": args.symbol,
        "value": ast.literal_eval(candidates[0]),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
