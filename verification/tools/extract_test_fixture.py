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
    parser.add_argument("--revision", help="Immutable source revision; defaults to the repository HEAD")
    parser.add_argument("--scope", help="Class/function scope separated by ::")
    parser.add_argument("--symbol", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    commit = subprocess.check_output(
        ["git", "-C", str(args.repository), "rev-parse", args.revision or "HEAD"], text=True
    ).strip()
    source = subprocess.check_output(
        ["git", "-C", str(args.repository), "show", f"{commit}:{args.source}"]
    )
    nodes = ast.parse(source).body
    if args.scope:
        for name in args.scope.split("::"):
            matches = [node for node in nodes if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name]
            if len(matches) != 1:
                parser.error(f"missing or ambiguous scope {args.scope}")
            nodes = matches[0].body
    candidates = [
        node.value
        for node in nodes
        if isinstance(node, ast.Assign)
        and any(isinstance(target, ast.Name) and target.id == args.symbol for target in node.targets)
    ]
    if len(candidates) != 1:
        parser.error(f"expected one literal assignment in the selected scope for {args.symbol}")
    fixture = {
        "source": args.source,
        "commit": commit,
        "source_sha256": hashlib.sha256(source).hexdigest(),
        "symbol": args.symbol,
        "value": ast.literal_eval(candidates[0]),
    }
    if args.scope:
        fixture["scope"] = args.scope
        fixture["source_span"] = [candidates[0].lineno, candidates[0].end_lineno]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
