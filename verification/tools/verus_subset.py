#!/usr/bin/env python3
"""Verify a deliberately small Rust safety subset through Verus.

Kani is not executed here. Eligibility must come from the single normalized
compiler-autoharness artifact produced by run_kani_autoharness_discovery.py.
Accepted Rust is structurally parsed and lowered to Verus; everything outside
the declared grammar is rejected rather than approximated.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from types import SimpleNamespace
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
KANI_COMMIT = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862"
VERUS_COMMIT = "aef82eda71838deef4a8cd0260fda6c9504be9b9"
CAPABILITIES = ROOT / "verification/automation/capabilities.json"
SUPPORTED_TYPES = {
    "bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64",
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
    pass


class VerificationError(RuntimeError):
    pass


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
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def strict_json(path: Path):
    def unique(items):
        result = {}
        for key, value in items:
            if key in result:
                raise VerificationError(f"duplicate JSON key {key!r} in {path}")
            result[key] = value
        return result

    def reject_constant(value):
        raise VerificationError(f"non-finite JSON constant {value!r} in {path}")

    try:
        return json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=unique,
            parse_constant=reject_constant,
        )
    except OSError as error:
        raise VerificationError(f"cannot read {path}: {error}") from error


def tokenize(text: str) -> list[Token]:
    output = []
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
        newlines = value.count("\n")
        if newlines:
            line += newlines
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

    def fail(self, message: str) -> None:
        token = self.current()
        raise Unsupported(f"{self.source}:{token.line}:{token.col}: {message}")

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

    def parse_file(self) -> list[Function]:
        functions = []
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
        params = []
        if self.current().value != ")":
            while True:
                param = self.take(kind="ident").value
                if param == "_":
                    self.fail("unnamed parameters are unsupported")
                self.take(":")
                typ = self.take(kind="ident").value
                if typ not in SUPPORTED_TYPES:
                    self.fail(f"unsupported parameter type {typ!r}")
                params.append((param, typ))
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
        previous = self.allowed_names
        self.allowed_names = {name for name, _ in params}
        body = self.parse_block()
        self.allowed_names = previous
        return Function(name, tuple(params), body, start.line, self.source)

    def parse_block(self) -> tuple[Stmt, ...]:
        self.take("{")
        statements = []
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
            expr = self.parse_expression()
            self.take(")")
            self.take(";")
            return AssertStmt(expr, token.line)
        if token.value == "if":
            self.take("if")
            cond = self.parse_expression()
            then_body = self.parse_block()
            else_body: tuple[Stmt, ...] = ()
            if self.accept("else"):
                if self.current().value == "if":
                    self.fail("else-if is unsupported; use nested if")
                else_body = self.parse_block()
            return IfStmt(cond, then_body, else_body, token.line)
        if token.value == "for":
            self.take("for")
            variable = self.take(kind="ident").value
            if variable != "_":
                self.fail("literal loop variable must be '_' in the initial subset")
            self.take("in")
            lower = int(self.take(kind="number").value)
            self.take("..")
            upper = int(self.take(kind="number").value)
            if upper < lower:
                self.fail("range upper bound is smaller than lower bound")
            if upper - lower > 32:
                self.fail("literal loop bound exceeds 32 iterations")
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
    return Parser(tokenize(path.read_text(encoding="utf-8")), str(path)).parse_file()


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
            f"{statement.start}..{statement.end} exhaustively unrolled"
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
    params = ", ".join(f"{name}: {typ}" for name, typ in function.params)
    output = ["use vstd::prelude::*;", "", "verus! {", f"fn {function.name}({params}) {{"]
    for statement in function.body:
        emit_statement(statement, "    ", relative, output)
    output.extend(["}", "}", ""])
    return "\n".join(output)


def collect_assert_lines(statements: Sequence[Stmt]) -> list[int]:
    lines = []
    for statement in statements:
        if isinstance(statement, AssertStmt):
            lines.append(statement.line)
        elif isinstance(statement, IfStmt):
            lines.extend(collect_assert_lines(statement.then_body))
            lines.extend(collect_assert_lines(statement.else_body))
        elif isinstance(statement, ForStmt):
            lines.extend(collect_assert_lines(statement.body))
    return lines


def collect_loop_bounds(statements: Sequence[Stmt]) -> list[dict[str, int | str]]:
    bounds = []
    for statement in statements:
        if isinstance(statement, ForStmt):
            bounds.append({
                "kind": "literal_for_exhaustive_unroll",
                "start": statement.start,
                "end": statement.end,
                "iterations": statement.end - statement.start,
            })
            bounds.extend(collect_loop_bounds(statement.body))
        elif isinstance(statement, IfStmt):
            bounds.extend(collect_loop_bounds(statement.then_body))
            bounds.extend(collect_loop_bounds(statement.else_body))
    return bounds


def source_for_function(manifest: Path, requested: str) -> tuple[Path, Function]:
    source_root = manifest.parent / "src"
    matches = []
    failures = []
    for path in sorted(source_root.rglob("*.rs")):
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


def require_kani_selection(discovery_path: Path, requested: str) -> dict:
    discovery = strict_json(discovery_path)
    if discovery.get("default_engine") != "verus":
        raise VerificationError("discovery default engine is not Verus")
    if discovery.get("proof_credit") is not False:
        raise VerificationError("Kani discovery unexpectedly carries proof credit")
    if discovery.get("kani_source_commit") != KANI_COMMIT:
        raise VerificationError("stale Kani discovery commit")
    matches = [
        row for row in discovery.get("candidates", [])
        if row.get("function") == requested or str(row.get("function", "")).endswith("::" + requested)
    ]
    if len(matches) != 1:
        raise VerificationError(
            f"expected exactly one Kani discovery identity for {requested!r}, found {len(matches)}"
        )
    row = matches[0]
    if row.get("disposition") != "kani_eligible":
        raise Unsupported(
            f"Kani did not select {requested!r}: {row.get('disposition')}: {row.get('reason')}"
        )
    return row


def import_checked_output():
    infra = ROOT / ".agents/infra"
    if not infra.is_dir():
        raise VerificationError("maintained Aegis controller is not materialized")
    sys.path.insert(0, str(infra))
    from agentinfra.destiny_reuse import checked_output
    return checked_output


def run(command: Sequence[str], cwd: Path, timeout: int):
    return subprocess.run(
        list(command),
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
        env=os.environ.copy(),
    )


def verify(
    manifest: Path,
    discovery_path: Path,
    requested: str,
    verus: Path,
    output_root: Path,
    timeout: int,
) -> dict:
    manifest = manifest.resolve(strict=True)
    discovery_path = discovery_path.resolve(strict=True)
    verus = verus.resolve(strict=True)
    kani_row = require_kani_selection(discovery_path, requested)

    try:
        source_path, function = source_for_function(manifest, requested)
    except Unsupported as error:
        return {
            "schema": 1,
            "status": "unsupported",
            "engine": "verus",
            "default_engine": "verus",
            "function": requested,
            "reason": str(error),
            "kani_selection": kani_row,
            "proof_credit": False,
        }

    generated_text = lower_function(function, source_path, manifest.parent)
    tracked = {
        "source": source_path,
        "manifest": manifest,
        "discovery": discovery_path,
        "translator": Path(__file__),
        "capabilities": CAPABILITIES,
    }
    before = {name: sha256_file(path) for name, path in tracked.items()}
    identity = {
        "schema": 1,
        "engine": "verus",
        "default_engine": "verus",
        "verus_commit": VERUS_COMMIT,
        "verus_sha256": sha256_file(verus),
        "kani_metadata_commit": KANI_COMMIT,
        "kani_selection": kani_row,
        "source_file": str(source_path),
        "source_sha256": before["source"],
        "input_sha256": before,
        "function": function.name,
        "parameter_domains": [
            {"name": name, "rust_type": typ, "independent": True}
            for name, typ in function.params
        ],
        "source_assert_lines": collect_assert_lines(function.body),
        "generated_assert_count": generated_text.count("assert("),
        "generated_sha256": sha256_bytes(generated_text.encode()),
        "assumptions": [],
        "bounds": collect_loop_bounds(function.body),
        "target_scope": "bool_and_fixed_width_integers_only; usize/isize rejected",
        "trusted_boundaries": [
            "Kani compiler autoharness eligibility metadata selects the source function; Kani/CBMC proof results are not used.",
            "The structural Rust-to-Verus lowering is trusted only for the declared grammar and is separately regression-tested.",
            "Verus, its vstd contracts, Rust toolchain and solver remain verifier trust boundaries.",
        ],
    }
    key = sha256_bytes(json.dumps(identity, sort_keys=True, separators=(",", ":")).encode())
    destination = output_root / key
    destination.mkdir(parents=True, exist_ok=True)
    generated = destination / f"{function.name}.rs"
    generated.write_text(generated_text, encoding="utf-8")
    command = [str(verus), "--crate-type=lib", str(generated)]

    try:
        result = run(command, ROOT, timeout)
    except subprocess.TimeoutExpired as error:
        row = {
            **identity,
            "key": key,
            "status": "timeout",
            "command": command,
            "stdout": error.stdout or "",
            "stderr": error.stderr or "",
        }
        (destination / "evidence.json").write_text(json.dumps(row, indent=2) + "\n")
        return row

    stdout = result.stdout
    stderr = result.stderr
    summaries = SUCCESS_RE.findall(stdout + "\n" + stderr)
    if result.returncode == 0:
        try:
            checked_output = import_checked_output()
            proxy = SimpleNamespace(
                returncode=0,
                timed_out=False,
                stdout_truncated=False,
                stderr_truncated=False,
                stdout=stdout,
                stderr=stderr,
            )
            obligation = {
                "method": "verus",
                "entry": str(generated),
                "expected_checks": 1,
            }
            status = "checked_valid" if checked_output(obligation, proxy) else "malformed_output"
        except (ImportError, RuntimeError, ValueError, VerificationError) as error:
            status = "infrastructure_error"
            stderr += f"\nstrict evidence parser unavailable: {error}\n"
    elif summaries:
        status = "proof_failure"
    else:
        status = "infrastructure_error"

    try:
        after = {name: sha256_file(path) for name, path in tracked.items()}
        generated_after = sha256_file(generated)
    except OSError as error:
        after = {"error": str(error)}
        generated_after = None
    if after != before or generated_after != identity["generated_sha256"]:
        status = "stale_input"

    row = {
        **identity,
        "key": key,
        "status": status,
        "post_input_sha256": after,
        "generated_after_sha256": generated_after,
        "command": command,
        "returncode": result.returncode,
        "summary": summaries,
        "stdout_sha256": sha256_bytes(stdout.encode()),
        "stderr_sha256": sha256_bytes(stderr.encode()),
        "generated": str(generated),
    }
    (destination / "stdout.txt").write_text(stdout, encoding="utf-8")
    (destination / "stderr.txt").write_text(stderr, encoding="utf-8")
    (destination / "evidence.json").write_text(
        json.dumps(row, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return row


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    check = sub.add_parser("verify")
    check.add_argument("--manifest-path", type=Path, required=True)
    check.add_argument("--discovery", type=Path, required=True)
    check.add_argument("--function", required=True)
    check.add_argument("--verus", type=Path, required=True)
    check.add_argument(
        "--output-root",
        type=Path,
        default=ROOT / ".aegis/destiny/verus-kani-subset",
    )
    check.add_argument("--timeout", type=int, default=120)

    sub.add_parser("explain")
    args = parser.parse_args()

    if args.command == "explain":
        print(CAPABILITIES.read_text(encoding="utf-8"), end="")
        return 0

    output_root = args.output_root if args.output_root.is_absolute() else ROOT / args.output_root
    try:
        row = verify(
            args.manifest_path,
            args.discovery,
            args.function,
            args.verus,
            output_root,
            args.timeout,
        )
    except Unsupported as error:
        row = {
            "schema": 1,
            "status": "unsupported",
            "engine": "verus",
            "function": args.function,
            "reason": str(error),
            "proof_credit": False,
        }
    except (VerificationError, OSError, json.JSONDecodeError, subprocess.SubprocessError) as error:
        row = {
            "schema": 1,
            "status": "infrastructure_error",
            "engine": "verus",
            "function": args.function,
            "reason": str(error),
            "proof_credit": False,
        }

    print(json.dumps(row, indent=2, sort_keys=True))
    return 0 if row.get("status") == "checked_valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
