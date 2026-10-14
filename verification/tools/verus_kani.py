#!/usr/bin/env python3
"""Kani-assisted, Verus-default verification for a deliberately small Rust subset.

Kani is used only for metadata/codegen discovery through autoharness list, which
upstream defines as only-codegen. The default path never accepts a Kani/CBMC
verification result. Supported source is structurally parsed and lowered into
Verus assertions; unsupported syntax is rejected.
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
KANI_VERSION = "0.68.0"
VERUS_VERSION = "0.2026.09.20.aef82ed"
KANI_COMMIT = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862"
VERUS_COMMIT = "aef82eda71838deef4a8cd0260fda6c9504be9b9"
SCHEMA = 1
SUPPORTED_TYPES = {
    "bool", "u8", "u16", "u32", "u64", "usize",
    "i8", "i16", "i32", "i64", "isize",
}
TOKEN_RE = re.compile(
    r"""
    (?P<ws>\s+)
  | (?P<linecomment>//[^\n]*)
  | (?P<blockcomment>/\*.*?\*/)
  | (?P<arrow>->)
  | (?P<and>&&)
  | (?P<or>\|\|)
  | (?P<eq>==)
  | (?P<ne>!=)
  | (?P<le><=)
  | (?P<ge>>=)
  | (?P<range>\.\.)
  | (?P<number>0|[1-9][0-9]*)
  | (?P<ident>[A-Za-z_][A-Za-z0-9_]*)
  | (?P<punct>[(){}:;,!<>])
    """,
    re.X | re.S,
)
SUCCESS_RE = re.compile(r"verification results::?\s*(\d+) verified, (\d+) errors")


class Unsupported(ValueError):
    """Source is outside the declared semantic bridge."""


@dataclasses.dataclass(frozen=True)
class Token:
    kind: str
    value: str
    line: int
    col: int


@dataclasses.dataclass(frozen=True)
class Expr:
    text: str


@dataclasses.dataclass(frozen=True)
class AssertStmt:
    expr: Expr
    line: int


@dataclasses.dataclass(frozen=True)
class IfStmt:
    cond: Expr
    then_body: tuple["Stmt", ...]
    else_body: tuple["Stmt", ...]
    line: int


@dataclasses.dataclass(frozen=True)
class ForStmt:
    start: int
    end: int
    body: tuple["Stmt", ...]
    line: int


Stmt = AssertStmt | IfStmt | ForStmt


@dataclasses.dataclass(frozen=True)
class Function:
    name: str
    params: tuple[tuple[str, str], ...]
    body: tuple[Stmt, ...]
    line: int
    source: str


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def strict_json(text: str):
    def unique(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"duplicate JSON key: {key}")
            result[key] = value
        return result

    def reject_constant(value):
        raise ValueError(f"non-finite JSON constant: {value}")

    return json.loads(
        text, object_pairs_hook=unique, parse_constant=reject_constant
    )


def tokenize(text: str) -> list[Token]:
    output: list[Token] = []
    pos = 0
    line = 1
    col = 1
    while pos < len(text):
        match = TOKEN_RE.match(text, pos)
        if not match:
            snippet = text[pos:pos + 30].splitlines()[0]
            raise Unsupported(f"unsupported token at {line}:{col}: {snippet!r}")
        value = match.group(0)
        kind = match.lastgroup or ""
        if kind not in {"ws", "linecomment", "blockcomment"}:
            output.append(Token(kind, value, line, col))
        newline_count = value.count("\n")
        if newline_count:
            line += newline_count
            col = len(value.rsplit("\n", 1)[-1]) + 1
        else:
            col += len(value)
        pos = match.end()
    output.append(Token("eof", "", line, col))
    return output


class Parser:
    def __init__(self, tokens: Sequence[Token], source: str):
        self.tokens = tokens
        self.index = 0
        self.source = source
        self.allowed_names: set[str] = set()

    def current(self) -> Token:
        return self.tokens[self.index]

    def take(self, value: str | None = None, kind: str | None = None) -> Token:
        token = self.current()
        if value is not None and token.value != value:
            self.fail(f"expected {value!r}, found {token.value!r}")
        if kind is not None and token.kind != kind:
            self.fail(f"expected {kind}, found {token.value!r}")
        self.index += 1
        return token

    def accept(self, value: str) -> bool:
        if self.current().value == value:
            self.index += 1
            return True
        return False

    def fail(self, message: str) -> None:
        token = self.current()
        raise Unsupported(f"{self.source}:{token.line}:{token.col}: {message}")

    def parse_file(self) -> list[Function]:
        functions: list[Function] = []
        while self.current().kind != "eof":
            self.accept("pub")
            if self.current().value != "fn":
                self.fail("only free functions are supported at top level")
            functions.append(self.parse_function())
        return functions

    def parse_function(self) -> Function:
        start = self.take("fn")
        name = self.take(kind="ident").value
        if self.current().value == "<":
            self.fail("generic functions are unsupported")
        self.take("(")
        params: list[tuple[str, str]] = []
        if self.current().value != ")":
            while True:
                parameter = self.take(kind="ident").value
                if parameter == "_":
                    self.fail("unnamed parameters are unsupported")
                self.take(":")
                type_name = self.take(kind="ident").value
                if type_name not in SUPPORTED_TYPES:
                    self.fail(f"unsupported parameter type {type_name!r}")
                params.append((parameter, type_name))
                if not self.accept(","):
                    break
                if self.current().value == ")":
                    break
        self.take(")")
        if self.accept("->"):
            if self.accept("("):
                self.take(")")
            else:
                self.fail("only unit return type is supported")
        previous_names = self.allowed_names
        self.allowed_names = {parameter for parameter, _ in params}
        body = self.parse_block()
        self.allowed_names = previous_names
        return Function(name, tuple(params), body, start.line, self.source)

    def parse_block(self) -> tuple[Stmt, ...]:
        self.take("{")
        statements: list[Stmt] = []
        while self.current().value != "}":
            if self.current().kind == "eof":
                self.fail("unterminated block")
            statements.append(self.parse_statement())
        self.take("}")
        return tuple(statements)

    def parse_statement(self) -> Stmt:
        token = self.current()
        if token.value == "assert":
            self.take("assert")
            self.take("!")
            self.take("(")
            expression = self.parse_expression()
            self.take(")")
            self.take(";")
            return AssertStmt(expression, token.line)
        if token.value == "if":
            self.take("if")
            condition = self.parse_expression()
            then_body = self.parse_block()
            else_body: tuple[Stmt, ...] = ()
            if self.accept("else"):
                if self.current().value == "if":
                    self.fail("else-if is not in the initial subset; use nested if")
                else_body = self.parse_block()
            return IfStmt(condition, then_body, else_body, token.line)
        if token.value == "for":
            self.take("for")
            loop_variable = self.take(kind="ident").value
            if loop_variable != "_":
                self.fail("bounded loop variable must be '_' in the initial subset")
            self.take("in")
            lower = int(self.take(kind="number").value)
            self.take("..")
            upper = int(self.take(kind="number").value)
            if upper < lower:
                self.fail("range upper bound is smaller than lower bound")
            if upper - lower > 32:
                self.fail("literal loop bound exceeds initial maximum of 32 iterations")
            body = self.parse_block()
            return ForStmt(lower, upper, body, token.line)
        self.fail("supported statements are assert!, if/else, and literal for-ranges")
        raise AssertionError("unreachable")

    def parse_expression(self) -> Expr:
        return Expr(self.parse_or())

    def parse_or(self) -> str:
        left = self.parse_and()
        while self.current().value == "||":
            self.take("||")
            left = f"({left} || {self.parse_and()})"
        return left

    def parse_and(self) -> str:
        left = self.parse_comparison()
        while self.current().value == "&&":
            self.take("&&")
            left = f"({left} && {self.parse_comparison()})"
        return left

    def parse_comparison(self) -> str:
        left = self.parse_unary()
        if self.current().value in {"==", "!=", "<", "<=", ">", ">="}:
            operator = self.take().value
            right = self.parse_unary()
            left = f"({left} {operator} {right})"
            if self.current().value in {"==", "!=", "<", "<=", ">", ">="}:
                self.fail("chained comparisons are unsupported")
        return left

    def parse_unary(self) -> str:
        if self.accept("!"):
            return f"!({self.parse_unary()})"
        return self.parse_atom()

    def parse_atom(self) -> str:
        token = self.current()
        if token.value == "(":
            self.take("(")
            inner = self.parse_or()
            self.take(")")
            return f"({inner})"
        if token.kind == "number":
            return self.take(kind="number").value
        if token.kind == "ident":
            name = self.take(kind="ident").value
            if name in {"true", "false"}:
                return name
            if name not in self.allowed_names:
                self.fail(f"expression identifier {name!r} is not a function parameter")
            return name
        self.fail(f"unsupported expression atom {token.value!r}")
        raise AssertionError("unreachable")


def parse_source(path: Path) -> list[Function]:
    return Parser(tokenize(path.read_text()), str(path)).parse_file()


def emit_statement(statement: Stmt, indent: str, source: str, output: list[str]) -> None:
    if isinstance(statement, AssertStmt):
        output.append(f"{indent}// source {source}:{statement.line}")
        output.append(f"{indent}assert({statement.expr.text});")
        return
    if isinstance(statement, IfStmt):
        output.append(f"{indent}// source {source}:{statement.line}")
        output.append(f"{indent}if {statement.cond.text} {{")
        for child in statement.then_body:
            emit_statement(child, indent + "    ", source, output)
        if statement.else_body:
            output.append(f"{indent}}} else {{")
            for child in statement.else_body:
                emit_statement(child, indent + "    ", source, output)
        output.append(f"{indent}}}")
        return
    if isinstance(statement, ForStmt):
        output.append(
            f"{indent}// source {source}:{statement.line}; exact literal range "
            f"{statement.start}..{statement.end} unrolled by translator"
        )
        for _ in range(statement.start, statement.end):
            for child in statement.body:
                emit_statement(child, indent, source, output)
        return
    raise TypeError(statement)


def lower_function(function: Function, source_path: Path, project_root: Path) -> str:
    try:
        relative = source_path.resolve().relative_to(project_root.resolve()).as_posix()
    except ValueError:
        relative = source_path.resolve().as_posix()
    params = ", ".join(f"{name}: {type_name}" for name, type_name in function.params)
    output = ["use vstd::prelude::*;", "", "verus! {", f"fn {function.name}({params}) {{"]
    for statement in function.body:
        emit_statement(statement, "    ", relative, output)
    output.extend(["}", "}", ""])
    return "\n".join(output)


def flatten_strings(value) -> list[str]:
    output: list[str] = []
    if isinstance(value, str):
        output.append(value)
    elif isinstance(value, list):
        for item in value:
            output.extend(flatten_strings(item))
    elif isinstance(value, dict):
        for item in value.values():
            output.extend(flatten_strings(item))
    return output


def run(command: Sequence[str], cwd: Path, timeout: int) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(command), cwd=cwd, text=True, capture_output=True, timeout=timeout,
        check=False, env=os.environ.copy()
    )


def tool_version(command: Sequence[str]) -> str:
    try:
        result = subprocess.run(
            list(command), text=True, capture_output=True, timeout=20, check=False
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return f"UNAVAILABLE: {error}"
    return (result.stdout + result.stderr).strip()


def discover(
    manifest: Path,
    kani_cargo: str,
    includes: Sequence[str],
    excludes: Sequence[str],
    timeout: int,
) -> dict:
    manifest = manifest.resolve()
    with tempfile.TemporaryDirectory(prefix="destiny-kani-list-") as temp_name:
        temp = Path(temp_name)
        command = [
            kani_cargo, "kani", "autoharness", "-Z", "autoharness",
            "--manifest-path", str(manifest),
            "--target-dir", str(temp / "target"),
            "--list", "--format", "json", "--quiet",
        ]
        for pattern in includes:
            command += ["--include-pattern", pattern]
        for pattern in excludes:
            command += ["--exclude-pattern", pattern]
        result = run(command, temp, timeout)
        list_path = temp / "kani-list.json"
        if result.returncode != 0:
            return {
                "status": "infrastructure_error",
                "stage": "kani_metadata_codegen",
                "command": command,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }
        if not list_path.is_file():
            return {
                "status": "malformed_output",
                "stage": "kani_metadata_codegen",
                "reason": "Kani did not produce kani-list.json",
                "command": command,
                "stdout": result.stdout,
                "stderr": result.stderr,
            }
        try:
            data = strict_json(list_path.read_text())
        except (json.JSONDecodeError, ValueError) as error:
            return {"status": "malformed_output", "reason": f"invalid kani-list.json: {error}"}
        if data.get("kani-version") != KANI_VERSION or data.get("file-version") != "0.1":
            return {
                "status": "mismatched_tool_or_schema",
                "observed_kani_version": data.get("kani-version"),
                "observed_file_version": data.get("file-version"),
            }
        listed_harnesses = sorted(
            set(flatten_strings(data.get("standard-harnesses", {})))
            | set(flatten_strings(data.get("contract-harnesses", {})))
        )
        listed_harnesses = [
            name for name in listed_harnesses
            if "/" not in name and "\\" not in name and name != "NONE"
        ]
        chosen: set[str] = set()
        skipped = []
        metadata_files = []
        for metadata_path in sorted((temp / "target").rglob("*.kani-metadata.json")):
            try:
                metadata = strict_json(metadata_path.read_text())
            except (json.JSONDecodeError, ValueError) as error:
                return {
                    "status": "malformed_output",
                    "reason": f"invalid Kani metadata {metadata_path.name}: {error}",
                }
            auto = metadata.get("autoharness_md")
            if not auto:
                continue
            metadata_files.append(metadata_path.name)
            chosen.update(auto.get("chosen", []))
            for function, reason in auto.get("skipped", {}).items():
                skipped.append({
                    "crate": metadata.get("crate_name"),
                    "function": function,
                    "reason": reason,
                })
        selected = sorted(chosen) if metadata_files else listed_harnesses
        kani_driver = shutil.which("cargo-kani")
        return {
            "status": "discovered" if selected else "empty_selection",
            "stage": "kani_metadata_codegen",
            "engine": "kani_metadata_only",
            "cbmc_verification": False,
            "command": command,
            "selected": selected,
            "listed_harnesses": listed_harnesses,
            "skipped": sorted(skipped, key=lambda row: (str(row["crate"]), row["function"])),
            "metadata_files": metadata_files,
            "totals": data.get("totals", {}),
            "kani_driver_path": kani_driver,
            "kani_driver_sha256": (
                sha256_file(Path(kani_driver)) if kani_driver and Path(kani_driver).is_file()
                else None
            ),
            "stdout_sha256": sha256_bytes(result.stdout.encode()),
            "stderr_sha256": sha256_bytes(result.stderr.encode()),
        }


def find_source_files(manifest: Path) -> list[Path]:
    source = manifest.parent / "src"
    return sorted(source.rglob("*.rs")) if source.is_dir() else []


def source_for_function(manifest: Path, requested: str) -> tuple[Path, Function]:
    matches: list[tuple[Path, Function]] = []
    failures: list[str] = []
    for path in find_source_files(manifest):
        try:
            functions = parse_source(path)
        except Unsupported as error:
            failures.append(str(error))
            continue
        for function in functions:
            if requested == function.name or requested.endswith("::" + function.name):
                matches.append((path, function))
    if len(matches) != 1:
        detail = "; ".join(failures[:3])
        raise Unsupported(f"function {requested!r} is not uniquely supported; {detail}")
    return matches[0]


def import_checked_output():
    infra = ROOT / ".agents" / "infra"
    if not infra.is_dir():
        raise RuntimeError(
            "maintained controller unavailable: materialize .agents at its pinned gitlink "
            "and run verification/tools/bootstrap_controller.py"
        )
    sys.path.insert(0, str(infra))
    from agentinfra.destiny_reuse import checked_output
    return checked_output


def verify_one(
    function: Function,
    source_path: Path,
    manifest: Path,
    verus: Path,
    output_root: Path,
    timeout: int,
) -> dict:
    generated_text = lower_function(function, source_path, manifest.parent)
    tracked_inputs = {
        "source": source_path,
        "manifest": manifest,
        "translator": Path(__file__),
        "capabilities": ROOT / "verification" / "automation" / "capabilities.json",
        "vendor_manifest": ROOT / "verification" / "vendor" / "manifest.json",
    }
    input_sha256 = {name: sha256_file(path) for name, path in tracked_inputs.items()}
    identity = {
        "schema": SCHEMA,
        "engine": "verus",
        "verus_commit": VERUS_COMMIT,
        "verus_sha256": sha256_file(verus),
        "kani_metadata_commit": KANI_COMMIT,
        "source_file": str(source_path.resolve()),
        "source_sha256": input_sha256["source"],
        "input_sha256": input_sha256,
        "function": function.name,
        "generated_sha256": sha256_bytes(generated_text.encode()),
        "translator_sha256": input_sha256["translator"],
        "assumptions": [],
        "bounds": [{"kind": "literal_for_unroll", "max_iterations": 32}],
        "trusted_boundaries": [
            "Kani autoharness eligibility/source metadata is reused but Kani/CBMC verification is not.",
            "Rust-to-Verus translation correctness is limited to the declared parsed subset.",
            "Verus and its pinned solver/toolchain remain trusted according to their own model.",
        ],
    }
    key = sha256_bytes(json.dumps(identity, sort_keys=True, separators=(",", ":")).encode())
    destination = output_root / key
    destination.mkdir(parents=True, exist_ok=True)
    generated = destination / f"{function.name}.rs"
    generated.write_text(generated_text)
    command = [str(verus), "--crate-type=lib", str(generated)]
    try:
        result = run(command, ROOT, timeout)
    except subprocess.TimeoutExpired as error:
        row = {
            **identity, "key": key, "status": "timeout", "command": command,
            "stdout": error.stdout or "", "stderr": error.stderr or "",
        }
        (destination / "evidence.json").write_text(json.dumps(row, indent=2) + "\n")
        return row
    stdout = result.stdout
    stderr = result.stderr
    summaries = SUCCESS_RE.findall(stdout + "\n" + stderr)
    status = "proof_failure"
    if result.returncode == 0:
        try:
            checked_output = import_checked_output()
            proxy = SimpleNamespace(
                returncode=result.returncode, timed_out=False,
                stdout_truncated=False, stderr_truncated=False,
                stdout=stdout, stderr=stderr,
            )
            obligation = {"method": "verus", "entry": str(generated), "expected_checks": 1}
            status = "checked_valid" if checked_output(obligation, proxy) else "malformed_or_unexpected_success"
        except (RuntimeError, ValueError, ImportError) as error:
            status = "infrastructure_error"
            stderr += f"\nstrict evidence parser unavailable: {error}\n"
    elif not summaries:
        status = "proof_failure" if "error" in (stdout + stderr).lower() else "infrastructure_error"
    try:
        post_input_sha256 = {
            name: sha256_file(path) for name, path in tracked_inputs.items()
        }
        generated_after = sha256_file(generated)
    except OSError as error:
        post_input_sha256 = {"error": str(error)}
        generated_after = None
    if post_input_sha256 != input_sha256 or generated_after != identity["generated_sha256"]:
        status = "stale_input"

    row = {
        **identity,
        "key": key,
        "status": status,
        "post_input_sha256": post_input_sha256,
        "generated_after_sha256": generated_after,
        "command": command,
        "returncode": result.returncode,
        "summary": summaries,
        "stdout_sha256": sha256_bytes(stdout.encode()),
        "stderr_sha256": sha256_bytes(stderr.encode()),
        "generated": str(generated),
    }
    (destination / "stdout.txt").write_text(stdout)
    (destination / "stderr.txt").write_text(stderr)
    (destination / "evidence.json").write_text(json.dumps(row, indent=2) + "\n")
    return row


def command_doctor(args) -> int:
    vendor = ROOT / "verification" / "vendor" / "manifest.json"
    info = {
        "schema": SCHEMA,
        "default_engine": "verus",
        "verus_expected": VERUS_VERSION,
        "verus_observed": tool_version([args.verus, "--version"]) if args.verus else "not supplied",
        "kani_expected": KANI_VERSION,
        "kani_observed": tool_version([args.kani_cargo, "kani", "--version"]),
        "controller_present": (ROOT / ".agents" / "infra").is_dir(),
        "vendor_manifest_present": vendor.is_file(),
        "cbmc_used_by_default_path": False,
    }
    print(json.dumps(info, indent=2))
    return 0 if info["controller_present"] and info["vendor_manifest_present"] else 1


def command_discover(args) -> int:
    result = discover(
        Path(args.manifest_path), args.kani_cargo, args.include, args.exclude, args.timeout
    )
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "discovered" else 1


def command_verify(args) -> int:
    if args.engine != "verus":
        print(json.dumps({
            "status": "unsupported_mode",
            "reason": (
                "Kani verification remains an explicit reference backend in Destiny's "
                "existing Kani lanes; this command never uses it as fallback."
            ),
        }, indent=2))
        return 2
    if not args.verus:
        print(json.dumps({"status": "infrastructure_error", "reason": "--verus is required"}, indent=2))
        return 2
    manifest = Path(args.manifest_path).resolve()
    discovery = discover(manifest, args.kani_cargo, args.include, args.exclude, args.timeout)
    if discovery.get("status") != "discovered":
        print(json.dumps({"status": "incomplete", "discovery": discovery}, indent=2))
        return 1
    selected = discovery["selected"]
    if args.function:
        selected = [
            name for name in selected
            if name == args.function or name.endswith("::" + args.function)
        ]
        if len(selected) != 1:
            print(json.dumps({
                "status": "unsupported_or_unselected",
                "function": args.function,
                "selected": discovery["selected"],
            }, indent=2))
            return 1
    output_root = ROOT / ".aegis" / "destiny" / "verus-kani"
    results = []
    for name in selected:
        try:
            source, function = source_for_function(manifest, name)
        except Unsupported as error:
            results.append({"function": name, "status": "unsupported", "reason": str(error)})
            continue
        results.append(
            verify_one(function, source, manifest, Path(args.verus).resolve(), output_root, args.timeout)
        )
    if not results:
        status = "empty_selection"
    elif any(
        result["status"] in {
            "unsupported", "infrastructure_error", "timeout", "malformed_or_unexpected_success"
        }
        for result in results
    ):
        status = "incomplete"
    elif any(result["status"] == "proof_failure" for result in results):
        status = "refuted_or_unproved"
    elif all(result["status"] == "checked_valid" for result in results):
        status = "checked_valid"
    else:
        status = "incomplete"
    report = {
        "schema": SCHEMA,
        "status": status,
        "default_engine": "verus",
        "discovery_engine": "kani_metadata_only",
        "cbmc_verification": False,
        "manifest": str(manifest),
        "selected": selected,
        "skipped": discovery.get("skipped", []),
        "discovery": {
            key: discovery.get(key)
            for key in (
                "engine", "stage", "cbmc_verification", "listed_harnesses",
                "metadata_files", "totals", "kani_driver_path",
                "kani_driver_sha256", "stdout_sha256", "stderr_sha256"
            )
        },
        "results": results,
    }
    print(json.dumps(report, indent=2))
    return 0 if status == "checked_valid" else 1


def command_explain(args) -> int:
    capability_path = ROOT / "verification" / "automation" / "capabilities.json"
    data = json.loads(capability_path.read_text())
    if args.capability:
        rows = [row for row in data["capabilities"] if row["id"] == args.capability]
        if not rows:
            print(json.dumps({"status": "unknown_capability", "id": args.capability}, indent=2))
            return 1
        print(json.dumps(rows[0], indent=2))
    else:
        print(json.dumps(data, indent=2))
    return 0


def command_replay(args) -> int:
    evidence = json.loads(Path(args.evidence).read_text())
    status = evidence.get("status")
    if status == "proof_failure":
        print(json.dumps({
            "status": "non_replayable_proof_failure",
            "reason": "A Verus proof failure is not a concrete source-level counterexample. No trace is fabricated.",
            "evidence": str(Path(args.evidence).resolve()),
        }, indent=2))
        return 1
    print(json.dumps({
        "status": "not_applicable",
        "reason": f"replay requires a validated concrete witness; evidence status is {status!r}",
    }, indent=2))
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)

    doctor = subcommands.add_parser("doctor")
    doctor.add_argument("--verus")
    doctor.add_argument("--kani-cargo", default="cargo")
    doctor.set_defaults(func=command_doctor)

    discovery = subcommands.add_parser("discover")
    discovery.add_argument("--manifest-path", required=True)
    discovery.add_argument("--kani-cargo", default="cargo")
    discovery.add_argument("--include", action="append", default=[])
    discovery.add_argument("--exclude", action="append", default=[])
    discovery.add_argument("--timeout", type=int, default=120)
    discovery.set_defaults(func=command_discover)

    verify = subcommands.add_parser("verify")
    verify.add_argument("--manifest-path", required=True)
    verify.add_argument("--function")
    verify.add_argument("--engine", choices=["verus", "kani"], default="verus")
    verify.add_argument("--verus")
    verify.add_argument("--kani-cargo", default="cargo")
    verify.add_argument("--include", action="append", default=[])
    verify.add_argument("--exclude", action="append", default=[])
    verify.add_argument("--timeout", type=int, default=120)
    verify.set_defaults(func=command_verify)

    explain = subcommands.add_parser("explain")
    explain.add_argument("capability", nargs="?")
    explain.set_defaults(func=command_explain)

    replay = subcommands.add_parser("replay")
    replay.add_argument("--evidence", required=True)
    replay.set_defaults(func=command_replay)

    args = parser.parse_args()
    try:
        return args.func(args)
    except (
        Unsupported, OSError, json.JSONDecodeError, subprocess.SubprocessError
    ) as error:
        print(json.dumps({"status": "infrastructure_error", "reason": str(error)}, indent=2))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
