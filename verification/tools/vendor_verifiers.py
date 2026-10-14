#!/usr/bin/env python3
"""Validate or explicitly materialize the immutable Verus and Kani gitlinks."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "verification" / "vendor" / "manifest.json"

def run(*argv: str, cwd: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(argv, cwd=cwd, text=True, capture_output=True)
    if check and result.returncode:
        raise RuntimeError(f"{' '.join(argv)} failed: {result.stderr.strip()}")
    return result

def gitlink(path: str) -> str:
    fields = run("git", "ls-tree", "HEAD", "--", path).stdout.strip().split()
    if len(fields) < 3 or fields[0] != "160000" or fields[1] != "commit":
        raise RuntimeError(f"{path} is not an immutable gitlink in HEAD")
    return fields[2]

def materialized(path: Path) -> bool:
    return path.is_dir() and run(
        "git", "rev-parse", "--is-inside-work-tree", cwd=path, check=False
    ).returncode == 0

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--materialize", action="store_true",
                      help="explicitly fetch/init only the two pinned verifier submodules")
    mode.add_argument("--verify-only", action="store_true",
                      help="require materialized source and validate it without fetching")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text())
    rows = []
    for dep in manifest["upstreams"]:
        actual = gitlink(dep["path"])
        if actual != dep["commit"]:
            raise RuntimeError(f"{dep['path']} gitlink {actual} != pinned {dep['commit']}")
        path = ROOT / dep["path"]
        if args.materialize:
            run("git", "submodule", "update", "--init", "--depth", "1", "--", dep["path"])
        present = materialized(path)
        if args.verify_only and not present:
            raise RuntimeError(f"{dep['path']} is not materialized")
        row = {"name": dep["name"], "path": dep["path"], "gitlink": actual,
               "materialized": present}
        if present:
            head = run("git", "rev-parse", "HEAD", cwd=path).stdout.strip()
            tree = run("git", "rev-parse", "HEAD^{tree}", cwd=path).stdout.strip()
            dirty = run("git", "status", "--porcelain", "--untracked-files=all", cwd=path).stdout
            if head != dep["commit"]:
                raise RuntimeError(f"{dep['path']} HEAD {head} != pin {dep['commit']}")
            if tree != dep["tree"]:
                raise RuntimeError(f"{dep['path']} tree {tree} != pin {dep['tree']}")
            if dirty:
                raise RuntimeError(f"{dep['path']} has local modifications")
            row["tree"] = tree
        rows.append(row)
    print(json.dumps({"schema": 1, "status": "VENDOR_IDENTITIES_VALIDATED",
                      "upstreams": rows, "proof_credit": False}, indent=2))
    return 0

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, KeyError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "INVALID_OR_UNAVAILABLE", "reason": str(error)}))
        raise SystemExit(1)
