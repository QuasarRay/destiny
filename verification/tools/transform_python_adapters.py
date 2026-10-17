#!/usr/bin/env python3
"""Recorded, source-anchored AST transformations for representation differences.

Rules contain typed operations and source references, never Python code strings.
Only selected expression spans are replaced; surrounding user code is retained.
"""
import argparse
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
RULES = ROOT / "verification/migration/python_adapter_rules.json"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_original(revision, path):
    return subprocess.check_output(["git", "--no-replace-objects", "show", revision + ":" + path], cwd=ROOT)


def offset(source, line, column):
    # AST column offsets are UTF-8 byte offsets, including non-ASCII comments.
    lines = source.splitlines(keepends=True)
    return sum(len(part) for part in lines[:line - 1]) + column


def float32_bits(value, decode=False):
    key = ast.Subscript(value=copy.deepcopy(value), slice=ast.Constant("source_f32_bits_v1"), ctx=ast.Load())
    if decode:
        packed = ast.Call(func=ast.Attribute(value=key, attr="to_bytes", ctx=ast.Load()), args=[ast.Constant(4), ast.Constant("big")], keywords=[])
        decoded = ast.Call(func=ast.Attribute(value=ast.Name(id="struct", ctx=ast.Load()), attr="unpack", ctx=ast.Load()), args=[ast.Constant("!f"), packed], keywords=[])
        return ast.Subscript(value=decoded, slice=ast.Constant(0), ctx=ast.Load())
    packed = ast.Call(func=ast.Attribute(value=ast.Name(id="struct", ctx=ast.Load()), attr="pack", ctx=ast.Load()), args=[ast.Constant("!f"), copy.deepcopy(value)], keywords=[])
    word = ast.Call(func=ast.Attribute(value=ast.Name(id="int", ctx=ast.Load()), attr="from_bytes", ctx=ast.Load()), args=[packed, ast.Constant("big")], keywords=[])
    return ast.Dict(keys=[ast.Constant("source_f32_bits_v1")], values=[word])


def transform_pass(source, rules):
    tree = ast.parse(source)
    edits = []
    for rule in rules:
        if rule["kind"] in {"register_park_method", "extend_snapshot_optional_fields"}:
            if rule["kind"] == "register_park_method":
                matches = [node.value.args[0] for node in tree.body if isinstance(node, ast.Assign)
                           and any(isinstance(target, ast.Name) and target.id == "_SUPPORTED_DYNAMIC_PARK_METHODS" for target in node.targets)]
            else:
                matches = [node for node in ast.walk(tree) if isinstance(node, ast.Set)
                           and {item.value for item in node.elts if isinstance(item, ast.Constant)} == {"formations", "local_counter", "local_high_counter"}]
            if len(matches) != 1 or not isinstance(matches[0], ast.Set):
                raise ValueError("registration/optional field selection changed")
            target = matches[0]
            replacement = copy.deepcopy(target)
            replacement.elts.extend(ast.Constant(value) for value in rule["values"])
            edits.append((offset(source, target.lineno, target.col_offset), offset(source, target.end_lineno, target.end_col_offset), ast.unparse(ast.fix_missing_locations(replacement)).encode()))
            continue
        if rule["kind"] == "selected_product_return":
            if rule["fields"] != ["number", "tuple_vector3"]:
                raise ValueError("unsupported product representation")
            scope = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "_park_method")
            scope = next(node for node in scope.body if isinstance(node, ast.FunctionDef) and node.name == "method")
            target = next(node for node in scope.body if isinstance(node, ast.If))
            value = ast.Name(id="result", ctx=ast.Load())
            name = ast.Name(id="name", ctx=ast.Load())
            def decode(decoder, index):
                return ast.Call(func=ast.Name(id=decoder, ctx=ast.Load()), args=[ast.Subscript(value=copy.deepcopy(value), slice=ast.Constant(index), ctx=ast.Load()), copy.deepcopy(name)], keywords=[])
            product = ast.Tuple(elts=[decode("_backend_number", 0), ast.Call(func=ast.Name(id="tuple", ctx=ast.Load()), args=[decode("_backend_vector", 1)], keywords=[])], ctx=ast.Load())
            valid = ast.BoolOp(op=ast.And(), values=[
                ast.Call(func=ast.Name(id="isinstance", ctx=ast.Load()), args=[copy.deepcopy(value), ast.Name(id="list", ctx=ast.Load())], keywords=[]),
                ast.Compare(left=ast.Call(func=ast.Name(id="len", ctx=ast.Load()), args=[copy.deepcopy(value)], keywords=[]), ops=[ast.Eq()], comparators=[ast.Constant(2)])])
            error = ast.Raise(exc=ast.Call(func=ast.Name(id="BackendCallError", ctx=ast.Load()), args=[ast.Constant("backend returned an invalid box product")], keywords=[ast.keyword(arg="code", value=ast.Constant("invalid_backend_response"))]), cause=None)
            branch = ast.If(test=ast.Compare(left=name, ops=[ast.Eq()], comparators=[ast.Constant(rule["operation"])]), body=[
                ast.If(test=ast.Compare(left=copy.deepcopy(value), ops=[ast.Is()], comparators=[ast.Constant(None)]), body=[ast.Return(value=ast.Constant(None))], orelse=[]),
                ast.If(test=ast.UnaryOp(op=ast.Not(), operand=valid), body=[error], orelse=[]),
                ast.Return(value=product)], orelse=[])
            at=offset(source, target.lineno, 0)
            indentation=" "*target.col_offset
            rendered="\n".join(indentation+line for line in ast.unparse(ast.fix_missing_locations(branch)).splitlines())+"\n"
            edits.append((at,at,rendered.encode()))
            continue
        if rule["kind"] == "add_writable_ball_property":
            matches = [node.iter for node in ast.walk(tree) if isinstance(node, ast.For)
                       and isinstance(node.target, ast.Name) and node.target.id == "_field_name"
                       and isinstance(node.iter, ast.Tuple)
                       and any(isinstance(value, ast.Constant) and value.value == "Agility" for value in node.iter.elts)]
            if len(matches) != 1:
                raise ValueError("writable property registration changed")
            target = matches[0]
            replacement = copy.deepcopy(target)
            replacement.elts.append(ast.Constant(rule["operation"]))
            edits.append((offset(source, target.lineno, target.col_offset), offset(source, target.end_lineno, target.end_col_offset), ast.unparse(ast.fix_missing_locations(replacement)).encode()))
            continue
        nodes = tree.body
        for name in rule["scope"]:
            matched = [node for node in nodes if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name == name]
            if len(matched) != 1:
                raise ValueError("ambiguous or missing scope: " + "::".join(rule["scope"]))
            scope = matched[0]
            nodes = scope.body
        predicate = ast.Compare(left=ast.Name(id=rule["selector"], ctx=ast.Load()), ops=[ast.Eq()], comparators=[ast.Constant(rule["operation"])])
        if rule["kind"] in {"selected_return_shape", "selected_float32_return"}:
            matches = [node for node in ast.walk(scope) if isinstance(node, ast.Call)
                       and isinstance(node.func, ast.Name) and node.func.id == rule["decoder"]]
        else:
            matches = [node for node in ast.walk(scope) if isinstance(node, ast.Call)
                       and isinstance(node.func, ast.Attribute) and node.func.attr == "_invoke"
                       and isinstance(node.func.value, ast.Name) and node.func.value.id == "self"]
        if len(matches) != 1:
            raise ValueError("unreviewed invocation shape")
        invocation = matches[0]
        if rule["kind"] == "selected_return_shape":
            if rule["shape"] != "tuple" or rule["arity"] != 3:
                raise ValueError("unreviewed return representation")
            if not any(isinstance(node, ast.Return) and node.value is invocation for node in ast.walk(scope)):
                raise ValueError("expected decoded return value")
            target = invocation
            replacement = ast.IfExp(test=predicate, body=ast.Call(func=ast.Name(id="tuple", ctx=ast.Load()), args=[copy.deepcopy(target)], keywords=[]), orelse=copy.deepcopy(target))
        elif rule["kind"] == "selected_bool_integer_input":
            target = invocation.args[-1]
            if not isinstance(target, ast.Name) or target.id != rule["argument"]:
                raise ValueError("unreviewed scalar argument")
            check = ast.Call(func=ast.Name(id="isinstance", ctx=ast.Load()), args=[copy.deepcopy(target), ast.Name(id="bool", ctx=ast.Load())], keywords=[])
            replacement = ast.IfExp(test=ast.BoolOp(op=ast.And(), values=[predicate, check]),
                                    body=ast.Call(func=ast.Name(id="int", ctx=ast.Load()), args=[copy.deepcopy(target)], keywords=[]), orelse=copy.deepcopy(target))
        elif rule["kind"] == "selected_float32_return":
            target = invocation
            value = ast.Name(id="value", ctx=ast.Load())
            word = ast.Subscript(value=copy.deepcopy(value), slice=ast.Constant("source_f32_bits_v1"), ctx=ast.Load())
            checks = [predicate,
                ast.Call(func=ast.Name(id="isinstance", ctx=ast.Load()), args=[copy.deepcopy(value), ast.Name(id="dict", ctx=ast.Load())], keywords=[]),
                ast.Compare(left=ast.Call(func=ast.Name(id="set", ctx=ast.Load()), args=[copy.deepcopy(value)], keywords=[]), ops=[ast.Eq()], comparators=[ast.Set(elts=[ast.Constant("source_f32_bits_v1")])]),
                ast.Compare(left=ast.Call(func=ast.Name(id="type", ctx=ast.Load()), args=[copy.deepcopy(word)], keywords=[]), ops=[ast.Is()], comparators=[ast.Name(id="int", ctx=ast.Load())]),
                ast.Compare(left=ast.Constant(0), ops=[ast.LtE(), ast.LtE()], comparators=[copy.deepcopy(word), ast.Constant(0xffffffff)])]
            replacement = ast.IfExp(test=ast.BoolOp(op=ast.And(), values=checks), body=float32_bits(value, decode=True), orelse=copy.deepcopy(target))
        elif rule["kind"] == "selected_float32_argument":
            target = invocation.args[-1]
            value = ast.Name(id=rule["argument"], ctx=ast.Load())
            finite = ast.Call(func=ast.Attribute(value=ast.Name(id="math", ctx=ast.Load()), attr="isfinite", ctx=ast.Load()), args=[copy.deepcopy(value)], keywords=[])
            encoded = ast.IfExp(test=finite, body=ast.Call(func=ast.Name(id="float", ctx=ast.Load()), args=[copy.deepcopy(value)], keywords=[]), orelse=float32_bits(value))
            replacement = ast.IfExp(test=predicate, body=encoded, orelse=copy.deepcopy(target))
        else:
            raise ValueError("unsupported adapter rule")
        edits.append((offset(source, target.lineno, target.col_offset), offset(source, target.end_lineno, target.end_col_offset), ast.unparse(ast.fix_missing_locations(replacement)).encode()))
    result = source
    for start, end, replacement in sorted(edits, reverse=True):
        result = result[:start] + replacement + result[end:]
    ast.parse(result)
    return result


def transform(source, rules):
    # Reparse each narrow pass so disjoint scalar rules compose at the same
    # argument without overlapping byte edits or replacing earlier adapters.
    for rule in rules:
        source = transform_pass(source, [rule])
    return source


def generate(check=False):
    document = json.loads(RULES.read_text())
    bindings = {}
    for rule in document["rules"]:
        source = read_original(document["original_commit"], rule["source"])
        lines = source.decode().splitlines()
        start, end = rule["source_span"]
        region = "\n".join(lines[start - 1:end])
        if not all(evidence in region for evidence in rule["source_evidence"]):
            raise ValueError("original source no longer establishes the adapter rule")
        bindings[rule["source"]] = sha(source)
    target = ROOT / document["target"]
    base = read_original(document["base_commit"], document["target"])
    expected = transform(base, document["rules"])
    current = target.read_bytes()
    receipt_path = ROOT / "verification/migration/python_transformations.json"
    previous_output = json.loads(receipt_path.read_text()).get("output_sha256") if receipt_path.exists() else None
    if current != base and current != expected and sha(current) != previous_output:
        raise ValueError("target has independent edits; preserve and incorporate them before generation")
    if check and current != expected:
        raise ValueError("generated Python adapter drift")
    if not check:
        target.write_bytes(expected)
    receipt = {"schema": 1, "rule_version": "python-representation-ast-v1", "rules_sha256": sha(RULES.read_bytes()),
               "generator_sha256": sha(Path(__file__).read_bytes()), "base_commit": document["base_commit"],
               "target": document["target"], "base_sha256": sha(base), "output_sha256": sha(expected),
               "original_source_sha256": bindings, "original_commit": document["original_commit"],
               "regenerate": "python verification/tools/transform_python_adapters.py", "no_opaque_code_fields": True}
    (ROOT / "verification/migration/python_transformations.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"status": "DRIFT_CHECK_PASSED" if check else "GENERATED", "rules": len(document["rules"]), "target": document["target"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        generate(args.check)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(json.dumps({"status": "INCOMPLETE", "reason": str(error)}), file=sys.stderr)
        sys.exit(1)
