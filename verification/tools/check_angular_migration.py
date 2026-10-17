#!/usr/bin/env python3
"""Compile unchanged Original angular methods and retain independent bit results.

The scaffold supplies original field names/types, an in-park environment and
the unused external Vector3 declaration. It does not host Blue. Original
macros, Vector3d arithmetic, angular method and Modulo2pi execute unchanged.
"""
from __future__ import annotations

import argparse
import ctypes
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import random
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".aegis/destiny/migration/angular-oracle"
REVISION = "114e89fa584f0e1a81eb0a679aabf41af8870072"
sys.path.insert(0, str(ROOT / ".agents/infra"))
from agentinfra.process import run_process

FIELDS = dict(zip(
    ("yaw", "pitch", "roll", "old_yaw", "old_pitch", "old_roll", "roll_speed", "old_roll_speed", "yaw_delta"),
    ("mNewYaw", "mNewPitch", "mNewRoll", "mOldYaw", "mOldPitch", "mOldRoll", "mNewRollSpeed", "mOldRollSpeed", "mYawDelta"),
))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bits(value):
    return struct.pack(">d", value).hex()


def build():
    STATE.mkdir(parents=True, exist_ok=True)
    source = subprocess.check_output(["git", "show", REVISION + ":src/Ball.cpp"], cwd=ROOT)
    header = subprocess.check_output(["git", "show", REVISION + ":src/Vector3d.h"], cwd=ROOT)
    (STATE / "Vector3d.h").write_bytes(header)
    lines = source.decode().splitlines(keepends=True)
    regions = [(10, 16), (335, 466), (777, 807), (1885, 1898)]
    selected = {start: "".join(lines[start - 1:end]) for start, end in regions}
    mappings = "\n".join(f"ball.{cpp} = state[{index}];" for index, cpp in enumerate(FIELDS.values()))
    outputs = "\n".join(f"state[{index}] = ball.{cpp};" for index, cpp in enumerate(FIELDS.values()))
    declarations = "\n".join(f"double {cpp} = 0.0;" for cpp in FIELDS.values())
    scaffold = ("#include <algorithm>\n#include <cmath>\n#include <cstdlib>\n#include <cstdint>\n#include <vector>\n"
                "struct Vector3 { float x,y,z; Vector3(float,float,float); };\n#include \"Vector3d.h\"\n"
                + selected[10] + "\n"
                "enum { DSTBALL_WARP=3, DSTBALL_MISSILE=5 };\n"
                "struct Ballpark { long mCurrentTime; double mFriction, dt, mRollAcceleration, mRollDecay, mRollSpeedAcceleration, mRollSpeedDecay; };\n"
                "void Modulo2pi(double&, double&);\nstruct Ball {\n"
                "Ballpark* mPark; double mMass; Vector3d mNewVel, mNewPos, mGoto; int mMode; long mEffectStamp; std::vector<int64_t> mCollisions;\n"
                + declarations + "\nvoid CalculateYawPitchRoll(bool); bool IsWarpish(); bool IsAligningForWarp(); bool IsWarping();\n"
                "void RegisterBallNotInPark() { std::abort(); }\n};\n"
                + selected[335] + selected[777] + selected[1885] + "\n"
                "extern \"C\" void destiny_oracle_angles(double* state, const double* input, int mode, long stamp, long now, int collided, int snap) {\n"
                "static_assert(sizeof(double)==8);\n"
                "Ballpark park{now,input[10],input[11],input[12],input[13],input[14],input[15]};\nBall ball; ball.mPark=&park;\n"
                "ball.mMass=input[0]; ball.mNewVel=Vector3d(input[1],input[2],input[3]);\n"
                "ball.mNewPos=Vector3d(input[4],input[5],input[6]); ball.mGoto=Vector3d(input[7],input[8],input[9]);\n"
                "ball.mMode=mode; ball.mEffectStamp=stamp; if(collided) ball.mCollisions.push_back(1);\n"
                + mappings + "\nball.CalculateYawPitchRoll(snap!=0);\n" + outputs + "\n}\n")
    path = STATE / "angular_oracle.cpp"
    path.write_text(scaffold)
    library = STATE / "liboriginal_angular.so"
    command = ["clang++", "-std=c++17", "-O2", "-shared", "-fPIC", "-fno-fast-math", "-ffp-contract=off", str(path), "-o", str(library)]
    execution = run_process(command, cwd=ROOT, env={"PATH": os.environ["PATH"]}, timeout=45)
    receipt = {"schema": 1, "original_commit": REVISION, "source_sha256": sha(source), "header_sha256": sha(header),
               "adapter_sha256": sha(path.read_bytes()), "regions": regions, "execution": asdict(execution),
               "scope": "In-park angular methods on the host C++ ABI; no original Ball layout/Blue ABI or whole-engine claim. Callback trap is outside the selected in-park domain.",
               "macro_provenance": "Unchanged Ball.cpp lines 10-16, including MIN, ABS and exact pi literals.",
               "long_bytes": ctypes.sizeof(ctypes.c_long), "binary64_bytes": ctypes.sizeof(ctypes.c_double), "proof_credit": False}
    (STATE / "build.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if execution.returncode:
        raise RuntimeError(execution.stderr)
    receipt["library_sha256"] = sha(library.read_bytes())
    (STATE / "build.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt, library


def function(library):
    loaded = ctypes.CDLL(str(library))
    call = loaded.destiny_oracle_angles
    call.argtypes = [ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_double), ctypes.c_int, ctypes.c_long,
                     ctypes.c_long, ctypes.c_int, ctypes.c_int]
    call.restype = None
    return call


def expected(call, state, values, mode, stamp, now, collided, snap):
    storage = (ctypes.c_double * 9)(*state)
    call(storage, (ctypes.c_double * 16)(*values), mode, stamp, now, collided, snap)
    return list(storage)


def samples():
    receipt, library = build()
    call = function(library)
    rng = random.Random(114335)
    rows = []
    for index in range(136):
        state = [rng.uniform(-10.0, 10.0) for _ in range(9)] if index >= 8 else [0.0] * 9
        velocity = ([1.0, 2.0, 3.0] if index == 0 else [0.0] * 3 if index == 1 else
                    [1e-160, 0.0, 0.0] if index == 2 else [rng.uniform(-1000.0, 1000.0) for _ in range(3)])
        values = [10.0 ** rng.uniform(2, 12), *velocity, *[rng.uniform(-100.0, 100.0) for _ in range(6)],
                  1_000_000.0, (index % 4 + 1) * 0.25, 1.0, -0.5, -3.0, -0.9]
        if index == 3:
            values[0], values[10] = 1e-100, 1e308
        if index == 4:
            values[12:16] = [2.5, -0.75, -4.25, -0.2]
        mode = index % 13
        stamp, now, collided, snap = (-1 if index % 3 == 0 else 2), 10, bool(index % 2), index < 3 or index % 5 == 0
        output = expected(call, state, values, mode, stamp, now, collided, snap)
        rows.append({"state_bits": dict(zip(FIELDS, map(bits, state))), "input": values, "mode": mode,
                     "effect_stamp": stamp, "current_time": now, "has_collisions": collided, "snap": snap,
                     "expected_bits": dict(zip(FIELDS, map(bits, output)))})
    report = {"schema": 1, "status": "ORIGINAL_CPP_OBSERVATIONS", "original_commit": REVISION,
              "oracle": receipt, "checker_sha256": sha(Path(__file__).read_bytes()), "seed": 114335, "rows": rows,
              "scope": "136 deterministic angular transitions, all 13 source mode numbers, snap/non-snap, prior state, varying dt/roll, proper/alignment warp and extreme friction. Not a proof or complete world oracle.",
              "proof_credit": False}
    path = ROOT / "verification/migration/angular_oracle_samples.json"
    path.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"oracle": str(library), "observations": len(rows), "report": str(path), "proof_credit": False}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["samples"])
    parser.parse_args()
    samples()
