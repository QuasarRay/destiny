#!/usr/bin/env python3
"""Generate deterministic SHA-256 coverage for the distributable source tree."""

from __future__ import annotations

import hashlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "ARTIFACT_MANIFEST.md"
EXCLUDED_DIRECTORIES = {
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "__pycache__",
    "build",
    "dist",
    "target",
}


def distributable_files() -> list[Path]:
    files = []
    for path in ROOT.rglob("*"):
        if not path.is_file() or path == OUTPUT:
            continue
        if any(part in EXCLUDED_DIRECTORIES for part in path.relative_to(ROOT).parts):
            continue
        if path.suffix in {".pyc", ".pyo"}:
            continue
        files.append(path)
    return sorted(files, key=lambda path: path.relative_to(ROOT).as_posix())


def main() -> None:
    rows = []
    for path in distributable_files():
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        rows.append((path.relative_to(ROOT).as_posix(), digest, path.stat().st_size))

    lines = [
        "# Artifact integrity manifest",
        "",
        "Generated from the v4 delivery tree. This file excludes itself, transient",
        "build/cache directories, and Python bytecode. Paths are relative to the",
        "project root.",
        "",
        f"Covered files: **{len(rows)}**",
        "",
        "| Path | Bytes | SHA-256 |",
        "|---|---:|---|",
    ]
    lines.extend(f"| `{path}` | {size} | `{digest}` |" for path, digest, size in rows)
    OUTPUT.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
