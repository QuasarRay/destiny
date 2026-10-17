#!/usr/bin/env python3
"""Generate exact expected traces from original C++; check the native Bevy path.

This is runtime evidence for the selected finite query scenarios, not a proof or
a substitute for full original-engine/configuration/platform acceptance.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "python"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bits(value):
    return struct.pack(">d", value).hex()


def check(library, oracle, output, baseline=False):
    import destiny
    from destiny._backend import NativeBackend
    scenarios_path = ROOT / "verification/migration/geometry_scenarios.json"
    scenarios = json.loads(scenarios_path.read_text())
    oracle_library = ctypes.CDLL(str(oracle))
    length = oracle_library.destiny_oracle_vector_length
    length.argtypes = [ctypes.c_double] * 3
    length.restype = ctypes.c_double
    source_record = json.loads((ROOT / "verification/migration/compiler_geometry.json").read_text())
    header = ROOT / ".aegis/destiny/migration/original" / source_record["original_revisions"][-1] / "src/Vector3d.h"
    if sha(header) != source_record["source_sha256"]:
        raise ValueError("original oracle header drift")
    random_inputs = scenarios["random_vectors"]
    rng = random.Random(random_inputs["seed"])
    vectors = list(scenarios["fixed_vectors"])
    vectors.extend([[rng.uniform(random_inputs["minimum"], random_inputs["maximum"]) for _ in range(3)]
                    for _ in range(random_inputs["count"])])
    if not vectors:
        raise ValueError("empty oracle selection")
    backend = NativeBackend(library)
    native_calls = [0]
    for entry in ("dbc_runtime_call", "dbc_runtime_call_with_mode_callback", "dbc_runtime_call_with_event_callback"):
        if hasattr(backend._library, entry):
            function = getattr(backend._library, entry)
            def counted(*args, _function=function):
                native_calls[0] += 1
                return _function(*args)
            setattr(backend._library, entry, counted)
    rows = []
    park = destiny.Ballpark(isMaster=False, _backend=backend)
    try:
        for ball_id in (1, 2):
            park.AddBall(ball_id, 1.0, 0.0, 0.0, False, False, False, False, False,
                         0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0)
        park.ego = 2
        ball = park.GetBall(1)
        for vector in vectors:
            park.SetBallPosition(1, *vector)
            expected_center = length(*vector)
            for radius in scenarios["radii"]:
                park.SetBallRadius(1, radius)
                expected_surface = expected_center - radius - 0.0
                observations = {
                    "GetCenterDist": (park.GetCenterDist(1, 2), expected_center),
                    "GetSurfaceDist": (park.GetSurfaceDist(1, 2), expected_surface),
                    "ClientBall.centerDist": (ball.centerDist, expected_center),
                    "ClientBall.surfaceDist": (ball.surfaceDist, max(expected_surface, 0.0)),
                }
                for operation, (actual, expected) in observations.items():
                    rows.append({"input_bits": [bits(v) for v in vector], "radius_bits": bits(radius),
                                 "operation": operation, "expected_bits": bits(expected),
                                 "observed_bits": bits(actual), "passed": bits(actual) == bits(expected)})
        if park.GetCenterDist(1, 999) is not None or park.GetSurfaceDist(1, 999) is not None:
            raise AssertionError("missing-ID result changed")
        park.ego = 0
        if ball.centerDist != 0.0 or ball.surfaceDist != 0.0:
            raise AssertionError("missing-ego result changed")
    finally:
        park.close()
        backend.close()
    failures = [row for row in rows if not row["passed"]]
    if native_calls[0] == 0:
        raise AssertionError("no actual native dispatch executed")
    report = {"schema": 1, "status": "BASELINE_MISMATCH_OBSERVED" if baseline and failures else "FAILED" if failures else "PASSED_SCOPED",
              "proof_credit": False, "original_commit": source_record["original_revisions"][-1],
              "original_header_sha256": source_record["source_sha256"], "oracle_sha256": sha(oracle),
              "library": str(library), "library_sha256": sha(library), "scenarios_sha256": sha(scenarios_path),
              "checker_sha256": sha(Path(__file__)), "runtime_source_sha256": sha(ROOT / "src/runtime.rs"),
              "generated_adapter_sha256": sha(ROOT / "src/generated_geometry.rs"),
              "reused_source_sha256": {p: sha(ROOT / p) for p in ["verification/spec/src/geometry.rs", "verification/spec/src/model.rs"]},
              "vectors": len(vectors), "comparisons": len(rows), "failures": len(failures), "native_calls": native_calls[0],
              "rows": rows, "scope": scenarios["scope"],
              "baseline_library_freshness": "Historical pre-transformation binary; only negative-control evidence" if baseline else "Built for current implementation before execution"}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({k: report[k] for k in ("status", "vectors", "comparisons", "failures", "native_calls", "proof_credit")}))
    return 0 if (baseline and failures) or (not baseline and not failures) else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--baseline", action="store_true")
    args = parser.parse_args()
    return check(args.library.resolve(), args.oracle.resolve(), args.report.resolve(), args.baseline)


if __name__ == "__main__":
    sys.exit(main())
