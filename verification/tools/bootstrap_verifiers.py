#!/usr/bin/env python3
"""Materialize and verify the pinned Verus/Kani source submodules.

Network access is used only with --init. --check is intentionally offline and
fails if the source trees, nested Kani submodules, or Git object identities do
not match verification/vendor-lock.json.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / "verification/vendor-lock.json"


def git(*args: str, cwd: Path = ROOT) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode:
        raise RuntimeError(
            f"git {' '.join(args)} failed in {cwd}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def verify_repo(path: Path, commit: str, tree: str) -> None:
    if not (path / ".git").exists() and not (path / ".git").is_file():
        raise RuntimeError(f"missing materialized submodule: {path.relative_to(ROOT)}")
    actual_commit = git("rev-parse", "HEAD", cwd=path)
    actual_tree = git("rev-parse", "HEAD^{tree}", cwd=path)
    if actual_commit != commit:
        raise RuntimeError(
            f"{path.relative_to(ROOT)} commit mismatch: {actual_commit} != {commit}"
        )
    if actual_tree != tree:
        raise RuntimeError(
            f"{path.relative_to(ROOT)} tree mismatch: {actual_tree} != {tree}"
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--init", action="store_true", help="materialize pinned submodules")
    parser.add_argument("--check", action="store_true", help="verify the offline source identity")
    args = parser.parse_args()
    if not args.init and not args.check:
        args.check = True

    lock = json.loads(LOCK.read_text(encoding="utf-8"))

    if args.init:
        git(
            "submodule",
            "update",
            "--init",
            "--recursive",
            "--depth",
            "1",
            "--filter=blob:none",
            "third_party/verus",
            "third_party/kani",
        )

    verify_repo(
        ROOT / "third_party/verus",
        lock["verus"]["commit"],
        lock["verus"]["tree"],
    )
    verify_repo(
        ROOT / "third_party/kani",
        lock["kani"]["commit"],
        lock["kani"]["tree"],
    )

    kani = ROOT / "third_party/kani"
    for rel, expected in lock["kani"]["recursive_submodules"].items():
        path = kani / rel
        if not (path / ".git").exists() and not (path / ".git").is_file():
            raise RuntimeError(f"missing recursive Kani submodule: {rel}")
        actual = git("rev-parse", "HEAD", cwd=path)
        if actual != expected:
            raise RuntimeError(f"Kani submodule {rel} mismatch: {actual} != {expected}")

    print(
        json.dumps(
            {
                "status": "PINNED_SOURCE_OK",
                "verus": lock["verus"]["commit"],
                "kani": lock["kani"]["commit"],
                "offline_check": True,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, KeyError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "INCOMPLETE", "reason": str(error)}, sort_keys=True))
        raise SystemExit(1)
