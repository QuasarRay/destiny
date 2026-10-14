#!/usr/bin/env python3
"""Apply and validate the pinned metadata-only patch to the vendored Kani checkout."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "verification/vendor/upstreams.json"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(checkout: Path, *args: str, check: bool = True) -> str:
    return subprocess.run(
        ["git", "-C", str(checkout), *args],
        check=check,
        capture_output=True,
        text=True,
    ).stdout.rstrip()


def main() -> int:
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    spec = data["upstreams"]["kani"]
    patch_spec = spec["patches"][0]
    checkout = ROOT / spec["path"]
    patch = ROOT / patch_spec["path"]

    if git(checkout, "rev-parse", "HEAD") != spec["commit"]:
        raise ValueError("Kani checkout is not at the recorded commit")
    if sha256(patch) != patch_spec["sha256"]:
        raise ValueError("Kani patch digest differs from the recorded digest")

    expected = patch_spec["postimage_git_blobs"]

    def postimages_match() -> bool:
        return all(
            (checkout / rel).is_file()
            and git(checkout, "hash-object", rel) == blob
            for rel, blob in expected.items()
        )

    if not postimages_match():
        if git(checkout, "status", "--porcelain", "--untracked-files=all"):
            raise ValueError("Kani checkout has unrelated changes; preserve them before patching")
        if git(checkout, "rev-parse", "HEAD^{tree}") != spec["tree"]:
            raise ValueError("Kani source tree does not match the recorded pristine tree")
        subprocess.run(
            ["git", "-C", str(checkout), "apply", "--check", str(patch)],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(checkout), "apply", str(patch)],
            check=True,
        )

    if not postimages_match():
        raise ValueError("Applied Kani patch does not match recorded postimages")

    changed = {
        line[3:]
        for line in git(checkout, "status", "--porcelain", "--untracked-files=all").splitlines()
        if line
    }
    if changed != set(expected):
        raise ValueError(f"Unexpected Kani vendor modifications: {sorted(changed)}")

    print(json.dumps({
        "status": "KANI_VENDOR_PATCH_VALIDATED",
        "base_commit": spec["commit"],
        "patch": patch_spec["id"],
        "files": sorted(expected),
        "proof_credit": False,
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
