#!/usr/bin/env python3
"""Diagnose whether the retained literal Kani failure also occurs in C/CBMC.

The C expression is emitted from Clang's typed IR; expected bits come from the
independent original-header oracle. No assumptions or relaxed comparisons.
"""
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".aegis/destiny/migration"
sys.path.insert(0, str(ROOT / ".agents/infra"))
from agentinfra.destiny import environment, load_profile, reserve_attempt
from agentinfra.process import run_process


def c_expression(node):
    if node["machine_type"] != "double":
        raise ValueError("unsupported diagnostic machine type")
    if node["op"] == "field":
        return "vector[" + str({"x": 0, "y": 1, "z": 2}[node["name"]]) + "]"
    if node["op"] == "binary" and node["operator"] in ("+", "-", "*", "/"):
        return "(" + c_expression(node["left"]) + " " + node["operator"] + " " + c_expression(node["right"]) + ")"
    if node["op"] == "sqrt":
        return "sqrt(" + c_expression(node["value"]) + ")"
    raise ValueError("unsupported diagnostic expression")


def main():
    kani = json.loads((STATE / "generated-geometry-kani-direct.json").read_text())
    failure = re.search(r"Failed Checks:.*?f64::from_bits\((\d+)\)", kani["execution"]["stdout"], re.S)
    if not failure:
        raise ValueError("no retained literal failure to investigate")
    samples = json.loads((ROOT / "verification/migration/geometry_oracle_samples.json").read_text())
    sample = next(row for row in samples["rows"] if int(row["input_bits"][0], 16) == int(failure[1]))
    unit = json.loads((ROOT / "verification/migration/geometry_semantics.json").read_text())
    statements = []
    for index, word in enumerate(sample["input_bits"]):
        statements.extend([f"uint64_t lane{index} = UINT64_C({int(word, 16)});", f"memcpy(&vector[{index}], &lane{index}, sizeof(double));"])
    expression = c_expression(unit["functions"][0]["expression"])
    text = ("/* Generated diagnostic from Clang IR and original-header oracle. */\n"
            "#include <assert.h>\n#include <stdint.h>\n#include <string.h>\n#include <math.h>\n"
            "int main(void) {\n_Static_assert(sizeof(double)==8, \"binary64 required\");\n"
            "double vector[3];\n" + "\n".join(statements) + "\n"
            + "double result = " + expression + ";\nuint64_t observed; memcpy(&observed, &result, sizeof(result));\n"
            + f"assert(observed == UINT64_C({int(sample['expected_bits'], 16)}));\nreturn 0;\n}}\n")
    path = STATE / "subnormal-sqrt-diagnostic.c"
    path.write_text(text)
    cbmc = Path.home() / ".kani/kani-0.68.0/bin/cbmc"
    command = [str(cbmc), str(path), "--function", "main", "--floatbv", "--bounds-check", "--pointer-check",
               "--signed-overflow-check", "--unwind", "128", "--unwinding-assertions", "--trace"]
    key = hashlib.sha256((text + "\n" + " ".join(command)).encode()).hexdigest()
    if not reserve_attempt(ROOT, load_profile(), key, 60, "Diagnose the retained concrete sqrt discrepancy in C/CBMC without changing source semantics or comparisons."):
        raise RuntimeError("diagnostic attempt budget exhausted")
    env = environment(ROOT, False)
    native_build = run_process(["clang", "-std=c11", "-O0", "-ffp-contract=off", str(path), "-lm", "-o", str(STATE / "subnormal-sqrt-native")], cwd=ROOT, env=env, timeout=30)
    if native_build.returncode:
        raise RuntimeError(native_build.stderr)
    native = run_process([str(STATE / "subnormal-sqrt-native")], cwd=ROOT, env=env, timeout=10)
    result = run_process(command, cwd=ROOT, env=env, timeout=60, capture_limit=2_500_000)
    report = {"schema": 1, "question": "Does the exact original-oracle literal failure also appear in C/CBMC?", "proof_credit": False,
              "source_sha256": hashlib.sha256(text.encode()).hexdigest(), "input_bits": sample["input_bits"], "expected_bits": sample["expected_bits"],
              "native_build": asdict(native_build), "native_execution": asdict(native), "cbmc_execution": asdict(result),
              "scope": "One concrete original-header oracle input; unwind 128 with assertions retained. This diagnostic is not whole-feature proof."}
    (STATE / "subnormal-sqrt-diagnostic.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"native_exit": native.returncode, "cbmc_exit": result.returncode, "cbmc_timed_out": result.timed_out,
                      "stdout_tail": result.stdout[-2500:], "proof_credit": False}))


if __name__ == "__main__":
    main()
