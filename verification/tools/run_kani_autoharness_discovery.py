#!/usr/bin/env python3
"""Run patched vendored Kani autoharness discovery while hard-failing backend tool execution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--kani-driver",
        type=Path,
        default=ROOT / "third_party/kani/target/kani/bin/cargo-kani",
    )
    parser.add_argument(
        "--manifest-path",
        type=Path,
        default=ROOT / "verification/spec/Cargo.toml",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / ".aegis/destiny/kani-autoharness-discovery",
    )
    args = parser.parse_args()

    driver = args.kani_driver.resolve(strict=True)
    manifest = args.manifest_path.resolve(strict=True)
    out = args.output_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)
    forbidden_log = out / "forbidden-backend-tools.log"
    if forbidden_log.exists():
        forbidden_log.unlink()

    with tempfile.TemporaryDirectory(prefix="destiny-no-cbmc-") as tmp:
        blocker = Path(tmp)
        for name in ("cbmc", "goto-cc", "goto-instrument", "goto-analyzer"):
            path = blocker / name
            path.write_text(
                "#!/bin/sh\necho " + name + " >> " + repr(str(forbidden_log)) + "\nexit 97\n",
                encoding="utf-8",
            )
            path.chmod(path.stat().st_mode | stat.S_IXUSR)

        env = os.environ.copy()
        env["PATH"] = str(blocker) + os.pathsep + env.get("PATH", "")
        cmd = [
            str(driver),
            "autoharness",
            "-Z",
            "autoharness",
            "--list",
            "--format",
            "json",
            "--quiet",
            "--manifest-path",
            str(manifest),
            "--target-dir",
            str(out / "kani-target"),
        ]
        result = subprocess.run(cmd, cwd=out, env=env, text=True, capture_output=True)

    (out / "command.json").write_text(json.dumps(cmd, indent=2) + "\n", encoding="utf-8")
    (out / "stdout.txt").write_text(result.stdout, encoding="utf-8")
    (out / "stderr.txt").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise SystemExit(
            f"Kani metadata-only discovery failed with exit {result.returncode}; see {out}"
        )
    if forbidden_log.exists() and forbidden_log.read_text(encoding="utf-8").strip():
        raise SystemExit("A forbidden CBMC/GOTO backend tool was invoked")

    metadata = sorted((out / "kani-target").rglob("*.kani_metadata.json"))
    if not metadata:
        raise SystemExit("Kani produced no .kani_metadata.json artifacts")

    combined = []
    for path in metadata:
        payload = json.loads(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise SystemExit(f"Malformed Kani metadata object: {path}")
        combined.append(payload)
    combined_path = out / "kani-metadata.json"
    combined_path.write_text(
        json.dumps(combined, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    receipt = {
        "status": "KANI_AUTOHARNESS_DISCOVERY_ONLY",
        "proof_credit": False,
        "manifest": str(manifest.relative_to(ROOT)),
        "driver": str(driver),
        "combined_metadata": {
            "path": str(combined_path.relative_to(ROOT)),
            "sha256": digest(combined_path),
        },
        "metadata": [
            {
                "path": str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path),
                "sha256": digest(path),
            }
            for path in metadata
        ],
        "list_json": str((out / "kani-list.json").relative_to(ROOT))
        if (out / "kani-list.json").is_file()
        else None,
        "backend_tools_forbidden": ["cbmc", "goto-cc", "goto-instrument", "goto-analyzer"],
    }
    (out / "receipt.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(receipt, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
