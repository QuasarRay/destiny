#!/usr/bin/env python3
"""Generate implementation status from the shipped facade and Rust handlers.

Mapping verdicts describe stock ecosystem equivalence. This generator keeps
that question separate from implementation reachability: Python surfaces are
resolved with static object inspection, while native surfaces are extracted
from the Rust dispatch functions that actually accept Destiny titles.
"""

from __future__ import annotations

from collections import Counter
import importlib
import inspect
import json
from pathlib import Path
import re
import sys
from types import ModuleType
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry" / "title_mapping.json"
ANNOTATIONS = ROOT / "registry" / "implementation_annotations.json"
RUNTIME = ROOT / "src" / "runtime.rs"
OUTPUT = ROOT / "registry" / "implementation_status.json"

STATUS_DEFINITIONS = {
    "implemented-conformant": (
        "Reachable implementation with the tested legacy call/result contract for this title."
    ),
    "implemented-with-accepted-difference": (
        "Reachable native implementation with a material difference documented in COMPATIBILITY.md."
    ),
    "implemented-state-only": (
        "Reachable state field or call whose controller effect is intentionally not implemented."
    ),
    "facade-only": (
        "A statically reachable Python/Carbon object or protocol surface with no Rust dispatch claim."
    ),
    "unsupported": "No shipped Python object path or Rust Destiny dispatch handler was found.",
}


def _static_member(value: Any, name: str) -> Any:
    """Resolve one member without invoking descriptors or dynamic fallbacks."""

    try:
        return inspect.getattr_static(value, name)
    except AttributeError:
        if not isinstance(value, ModuleType):
            raise
        return importlib.import_module(f"{value.__name__}.{name}")


def python_reachable_titles(titles: set[str]) -> set[str]:
    """Return registry titles resolvable through shipped Python objects."""

    python_root = str(ROOT / "python")
    inserted = not sys.path or sys.path[0] != python_root
    if inserted:
        sys.path.insert(0, python_root)
    try:
        destiny = importlib.import_module("destiny")
        reachable: set[str] = set()
        for title in sorted(titles):
            parts = title.split(".")
            if not parts or parts[0] != "destiny":
                continue
            current: Any = destiny
            try:
                for part in parts[1:]:
                    current = _static_member(current, part)
            except (AttributeError, ImportError, ModuleNotFoundError):
                continue
            reachable.add(title)
        return reachable
    finally:
        if inserted:
            sys.path.pop(0)


def _function_body(source: str, name: str) -> str:
    """Extract a Rust function body while ignoring braces in strings/comments."""

    match = re.search(rf"\bfn\s+{re.escape(name)}\s*\(", source)
    if match is None:
        raise RuntimeError(f"required Rust handler {name!r} was not found")
    start = source.find("{", match.end())
    if start < 0:
        raise RuntimeError(f"Rust handler {name!r} has no body")

    depth = 0
    index = start
    state = "code"
    block_depth = 0
    while index < len(source):
        char = source[index]
        following = source[index + 1] if index + 1 < len(source) else ""
        if state == "line-comment":
            if char == "\n":
                state = "code"
        elif state == "block-comment":
            if char == "/" and following == "*":
                block_depth += 1
                index += 1
            elif char == "*" and following == "/":
                block_depth -= 1
                index += 1
                if block_depth == 0:
                    state = "code"
        elif state == "string":
            if char == "\\":
                index += 1
            elif char == '"':
                state = "code"
        elif state == "code":
            if char == "/" and following == "/":
                state = "line-comment"
                index += 1
            elif char == "/" and following == "*":
                state = "block-comment"
                block_depth = 1
                index += 1
            elif char == '"':
                state = "string"
            elif char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
                if depth == 0:
                    return source[start + 1:index]
        index += 1
    raise RuntimeError(f"Rust handler {name!r} has an unterminated body")


def _identifier_literals(body: str) -> set[str]:
    return set(re.findall(r'"([A-Za-z_][A-Za-z0-9_.:]*)"', body))


def rust_reachable_titles(titles: set[str]) -> set[str]:
    """Derive native reachability from the Rust dispatch implementation."""

    source = RUNTIME.read_text(encoding="utf-8")
    functions = {
        name: _identifier_literals(_function_body(source, name))
        for name in (
            "dispatch",
            "dispatch_ball",
            "get_ball_property",
            "set_ball_property",
            "dispatch_park",
            "park_call_arities",
        )
    }
    exact = {value for value in functions["dispatch"] if value.startswith("destiny.")}
    ball_members = (
        functions["dispatch_ball"]
        | functions["get_ball_property"]
        | functions["set_ball_property"]
    )
    park_members = functions["dispatch_park"] | functions["park_call_arities"]

    reachable: set[str] = set()
    for title in titles:
        member = title.rsplit(".", 1)[-1]
        if title in exact:
            reachable.add(title)
        elif title.startswith(("destiny.Ball.", "destiny.ClientBall.")):
            if member in ball_members:
                reachable.add(title)
        elif title.startswith("destiny.Ballpark.") and member in park_members:
            reachable.add(title)

    # Class roots are reachable when a native descendant is reachable. This
    # avoids maintaining another hand-written class coverage list.
    for title in titles:
        prefix = f"{title}."
        if any(candidate.startswith(prefix) for candidate in reachable):
            reachable.add(title)
    return reachable


def _load_annotations(titles: set[str]) -> dict[str, dict[str, str]]:
    payload = json.loads(ANNOTATIONS.read_text(encoding="utf-8"))
    if payload.get("schema_version") != 1 or set(payload) != {"schema_version", "titles"}:
        raise ValueError("implementation annotations have an invalid schema")
    annotations = payload["titles"]
    if not isinstance(annotations, dict) or not set(annotations).issubset(titles):
        raise ValueError("implementation annotations contain unknown titles")
    for title, annotation in annotations.items():
        if not isinstance(annotation, dict) or set(annotation) != {"status", "note"}:
            raise ValueError(f"invalid implementation annotation for {title}")
        if annotation.get("status") not in STATUS_DEFINITIONS:
            raise ValueError(f"invalid implementation status annotation for {title}")
        if not isinstance(annotation.get("note"), str) or not annotation["note"]:
            raise ValueError(f"missing implementation annotation note for {title}")
    return annotations


def generate_payload() -> dict[str, Any]:
    source = json.loads(REGISTRY.read_text(encoding="utf-8"))
    mappings = source["records"]
    titles = {mapping["canonical_title"] for mapping in mappings}
    if len(titles) != len(mappings):
        raise ValueError("mapping registry contains duplicate canonical titles")

    python_titles = python_reachable_titles(titles)
    rust_titles = rust_reachable_titles(titles)
    annotations = _load_annotations(titles)
    records = []
    for mapping in mappings:
        title = mapping["canonical_title"]
        surfaces = []
        if title in python_titles:
            surfaces.append("python")
        if title in rust_titles:
            surfaces.extend(("rust-native", "c-abi"))

        if title in rust_titles:
            status = "implemented-with-accepted-difference"
            note = (
                "A Rust Destiny dispatch handler is reachable through the stable C ABI; "
                "the documented Bevy/Avian lifecycle, precision, or snapshot differences apply."
            )
        elif title in python_titles:
            status = "facade-only"
            note = (
                "The shipped Python/Carbon object path is statically reachable; no Rust Destiny "
                "dispatch handler is claimed for this title."
            )
        else:
            status = "unsupported"
            note = (
                "No shipped Python object path or Rust Destiny dispatch handler was found; "
                "registry-aware access fails explicitly where a dynamic fallback exists."
            )

        annotation = annotations.get(title)
        if annotation is not None:
            status = annotation["status"]
            note = annotation["note"]
            if status != "unsupported" and not surfaces:
                raise ValueError(f"annotation claims unreachable implementation {title}")

        records.append(
            {
                "title_id": mapping["title_id"],
                "canonical_title": title,
                "mapping_verdict": mapping["verdict"],
                "implementation_status": status,
                "surfaces": surfaces,
                "note": note,
            }
        )

    counts = Counter(record["implementation_status"] for record in records)
    return {
        "schema_version": 2,
        "mapping_source": "registry/title_mapping.json",
        "annotation_source": "registry/implementation_annotations.json",
        "mapping_and_implementation_are_independent": True,
        "reachability_evidence": {
            "python": "inspect.getattr_static over the shipped destiny package; descriptors are not invoked",
            "rust-native": "identifier extraction from the Rust dispatch, ball, property, and park handlers",
            "c-abi": "the stable generic C ABI reaches every Rust-native Destiny dispatch handler",
        },
        "status_definitions": STATUS_DEFINITIONS,
        "counts": dict(sorted(counts.items())),
        "records": records,
    }


def main() -> None:
    OUTPUT.write_text(
        json.dumps(generate_payload(), indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
