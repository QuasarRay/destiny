#!/usr/bin/env python3
"""Generate deterministic SHA-256 coverage for the distributable source tree."""

from __future__ import annotations

import hashlib
import argparse
import os
from pathlib import Path
import tarfile
import tomllib


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
    ".aegis",
}


def distributable_files() -> list[Path]:
    settings = tomllib.loads((ROOT / "pyproject.toml").read_text())["tool"]["scikit-build"]
    patterns = settings.get("sdist", {}).get("exclude", [])
    excluded_roots = {pattern[:-3] for pattern in patterns if pattern.endswith("/**")}
    excluded_files = {pattern for pattern in patterns if "/" not in pattern}
    files = []
    for directory, subdirectories, names in os.walk(ROOT):
        relative = Path(directory).relative_to(ROOT)
        subdirectories[:] = [name for name in subdirectories
                             if name not in EXCLUDED_DIRECTORIES
                             and (relative / name).as_posix() not in excluded_roots]
        for name in names:
            path = Path(directory) / name
            if path == OUTPUT or path.relative_to(ROOT).as_posix() in excluded_files:
                continue
            if path.is_symlink() or path.suffix in {".pyc", ".pyo"}:
                continue
            files.append(path)
    return sorted(files, key=lambda path: path.relative_to(ROOT).as_posix())


def archive_rows(archive: Path) -> list[tuple[str, str, int]]:
    rows, seen, roots = [], set(), set()
    with tarfile.open(archive, "r:gz") as source:
        for member in source.getmembers():
            path = Path(member.name)
            if path.is_absolute() or ".." in path.parts or not path.parts:
                raise ValueError("unsafe source archive path")
            roots.add(path.parts[0])
            if member.isdir():
                continue
            if not member.isfile() or len(path.parts) < 2:
                raise ValueError("source archive contains a link or unsupported entry")
            relative = Path(*path.parts[1:]).as_posix()
            if relative in seen:
                raise ValueError("duplicate source archive path")
            seen.add(relative)
            if relative == OUTPUT.name:
                continue
            data = source.extractfile(member)
            if data is None:
                raise ValueError("unreadable source archive entry")
            contents = data.read()
            rows.append((relative, hashlib.sha256(contents).hexdigest(), len(contents)))
    if len(roots) != 1 or not rows:
        raise ValueError("source archive must have one nonempty distribution root")
    return sorted(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, help="Hash the actual built source distribution")
    parser.add_argument("--check", action="store_true", help="Reject manifest drift without rewriting")
    args = parser.parse_args()
    if args.archive:
        rows = archive_rows(args.archive)
        scope = "the candidate source distribution"
    else:
        rows = [(path.relative_to(ROOT).as_posix(), hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_size)
                for path in distributable_files()]
        scope = "the local distributable source tree"

    lines = [
        "# Artifact integrity manifest",
        "",
        f"Generated from {scope}. This file excludes itself.",
        "Local controller/reference checkouts, user prompts, build/cache directories",
        "and Python bytecode are excluded by the packaging policy. Paths are relative",
        "to the distribution root. Hash coverage does not establish release readiness.",
        "",
        f"Covered files: **{len(rows)}**",
        "",
        "| Path | Bytes | SHA-256 |",
        "|---|---:|---|",
    ]
    lines.extend(f"| `{path}` | {size} | `{digest}` |" for path, digest, size in rows)
    result = "\n".join(lines) + "\n"
    if args.check:
        if OUTPUT.read_text() != result:
            raise SystemExit("artifact manifest drift")
    else:
        OUTPUT.write_text(result, encoding="utf-8")


if __name__ == "__main__":
    main()
