#!/usr/bin/env python3
"""Fail closed unless vendored verifier gitlinks match recorded immutable commits/trees."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "verification/vendor/upstreams.json"


def git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(
        ["git", "-C", str(cwd), *args],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def main() -> int:
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    checked = []
    for name, spec in data["upstreams"].items():
        path = spec["path"]
        entry = git("ls-tree", "HEAD", "--", path)
        parts = entry.split()
        if len(parts) < 4 or parts[0] != "160000" or parts[1] != "commit":
            raise ValueError(f"{path} is not an immutable gitlink")
        if parts[2] != spec["commit"]:
            raise ValueError(f"{path} gitlink {parts[2]} != recorded {spec['commit']}")

        checkout = ROOT / path
        if not checkout.is_dir():
            raise ValueError(f"{path} is not materialized; run git submodule update --init {path}")
        head = git("rev-parse", "HEAD", cwd=checkout)
        tree = git("rev-parse", "HEAD^{tree}", cwd=checkout)
        if head != spec["commit"]:
            raise ValueError(f"{path} checkout {head} != recorded {spec['commit']}")
        if tree != spec["tree"]:
            raise ValueError(f"{path} tree {tree} != recorded {spec['tree']}")
        if git("status", "--porcelain", "--untracked-files=all", cwd=checkout):
            raise ValueError(f"{path} has local modifications")
        for license_path in spec["license_paths"]:
            if not (checkout / license_path).is_file():
                raise ValueError(f"{path} missing recorded license {license_path}")
        checked.append({"name": name, "commit": head, "tree": tree})

    print(json.dumps({
        "status": "VENDOR_PINS_VALIDATED",
        "checked": checked,
        "proof_credit": False,
        "note": "Source provenance only; this does not execute either verifier.",
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
