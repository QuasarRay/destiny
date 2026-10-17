#!/usr/bin/env python3
"""Independent unchanged Original missile numeric/state-field observations.

The reference supplies real scalar fields and Vector3d, then executes source
field initialization, EvolveMissile, GotoThrust, time-factor and integration.
Admission, callbacks, collisions, Blue conversion/layout and platforms remain
separate obligations; the selected numeric domain does not replace them.
"""
import ctypes
import hashlib
import json
from pathlib import Path
import random
import struct
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from tools.migrate_destiny import LATEST, STATE, blob, checked, save


def digest(data):
    return hashlib.sha256(data).hexdigest()


def rotation_samples(directory):
    """Run source launch arithmetic with its real pinned Carbon/DirectX math."""
    dependency_root = ROOT / ".aegis/destiny/math-sources"
    math_receipt = json.loads((dependency_root / "latest/source-receipt.json").read_text())
    math = ROOT / math_receipt["source_root"]
    if math_receipt["commit"] != "540edbfbb779a8f5616e9bbdfbf595c564af5a2c":
        raise ValueError("unreviewed Carbon math source")
    archive = dependency_root / "directxmath-apr2025.tar.gz"
    archive_hash = "c7d3b107180b269c5c4e823fa51d96a316dc35cace3cb13f030022d9096c9465e8a770559419176692b047574fd67c96d8527abd8817998264a149eee0b88c9d"
    if not archive.exists():
        checked(["curl", "--fail", "--silent", "--show-error", "--location", "--max-time", "60", "https://codeload.github.com/microsoft/DirectXMath/tar.gz/refs/tags/apr2025", "--output", str(archive)], "world-missile-directx-source", 70)
    if hashlib.sha512(archive.read_bytes()).hexdigest() != archive_hash:
        raise ValueError("DirectXMath archive does not match original pinned vcpkg recipe")
    directx = dependency_root / "DirectXMath-apr2025"
    if not directx.exists():
        with tarfile.open(archive) as source:
            source.extractall(dependency_root, filter="data")
    sal = directx / "Inc/sal.h"
    sal_hash = "8085f67bfa4ce01ae89461cadf72454a9552fde3f08b2dcc3de36b9830e29ce7a6192800f8a5cb2a66af9637be0017e85719826a4cfdade508ae97f319e0ee8e"
    if not sal.exists():
        checked(["curl", "--fail", "--silent", "--show-error", "--location", "--max-time", "60", "https://raw.githubusercontent.com/dotnet/runtime/v9.0.2/src/coreclr/pal/inc/rt/sal.h", "--output", str(sal)], "world-missile-sal-source", 70)
    if hashlib.sha512(sal.read_bytes()).hexdigest() != sal_hash:
        raise ValueError("SAL header does not match original pinned vcpkg recipe")
    # Verify the previously reviewed Git blobs against the matching release.
    dependencies = json.loads((ROOT / "verification/cases/math_dependencies.json").read_text())
    for row in dependencies["files"]:
        if row["project"] == "directxmath":
            # The hash-pinned release archive exports .h/.inl with CRLF;
            # the previously verified Git blobs use LF. Record the relation
            # while retaining and compiling the archive's exact bytes.
            raw = (directx / row["path"]).read_bytes()
            if digest(raw.replace(b"\r\n", b"\n")) != row["sha256"]:
                raise ValueError("DirectXMath release/source identity mismatch")
    def region(raw, start, end):
        return b"".join(raw.splitlines(keepends=True)[start-1:end]).decode()
    ball = blob(LATEST, "src/Ball.cpp")
    launch = blob(LATEST, "src/Thunkers.cpp")
    quaternion = (math / "src/Quaternion.cpp").read_bytes()
    source = '''#include "CcpMath.h"
#include "Vector3d.h"
bool g_useDynamicalOrientation=false;
struct Ball {
 Quaternion mNewRot; double mNewYaw,mNewPitch,mNewRoll;
 Vector3d mNewPos,mNewVel; float mMaxVel; bool isCloaked;
 void GetRotatedVector(Vector3&);
};
''' + region(quaternion, 60, 82) + region(ball, 2191, 2203) + '''
extern "C" void destiny_original_stationary_launch(const double* input,double* output) {
 Ball launcher{}; launcher.mNewYaw=input[0]; launcher.mNewPitch=input[1]; launcher.mNewRoll=input[2]; launcher.mMaxVel=float(input[3]);
 Ball* launcherBall=&launcher; Ball* targetBall=nullptr; int aimedLaunch=0;
''' + region(launch, 898, 926) + '''
 output[0]=v0.x; output[1]=v0.y; output[2]=v0.z;
 Vector3 vector(0.0f,0.0f,-1.0f); launcher.GetRotatedVector(vector);
 output[3]=vector.x; output[4]=vector.y; output[5]=vector.z;
}
'''
    path = directory / "launch_rotation_oracle.cpp"
    library = directory / "liboriginal_launch_rotation.so"
    path.write_text(source)
    checked(["clang++", "-std=c++17", "-O2", "-shared", "-fPIC", "-msse2", "-mno-sse3", "-mno-avx", "-mno-fma", "-fno-fast-math", "-ffp-contract=off", "-I"+str(math / "include"), "-I"+str(directx / "Inc"), str(path), "-o", str(library)], "world-missile-launch-oracle-build", 60)
    loaded = ctypes.CDLL(str(library))
    call = loaded.destiny_original_stationary_launch
    call.argtypes = [ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_double)]
    call.restype = None
    rng = random.Random(114920)
    inputs = [[0.0,0.0,0.0,10.0], [1.5707963267948966,0.0,0.0,150.0], [0.3,-0.9,1.3,300.0]]
    inputs.extend([[rng.uniform(-3.1,3.1) for _ in range(3)]+[rng.uniform(0,500)] for _ in range(15)])
    rows = []
    for values in inputs:
        output = (ctypes.c_double*6)()
        call((ctypes.c_double*4)(*values), output)
        rows.append({"input":values, "expected_bits":[struct.pack(">d",value).hex() for value in output]})
    return {"rows":rows,"seed":114920,"math_commit":math_receipt["commit"],"directx_commit":"56206b1916646b29b37e1eac56e96e2008d6c71f",
            "vcpkg_baseline":"242ac6932a9cd2607332969129e9572516f28d65", "archive_sha512":archive_hash,"sal_sha512":sal_hash,
            "reviewed_git_blob_relation":"CRLF archive exports equal the reviewed LF Git blobs after only CRLF-to-LF conversion. Exact compiled archive-file bytes are hashed below.",
            "math_source_sha256":{str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in sorted([*(math/"include").glob("*.h"),math/"src/Quaternion.cpp"])},
            "directx_source_sha256":{str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in sorted((directx/"Inc").glob("*")) if p.is_file()},
            "quaternion_source_sha256":digest(quaternion),"quaternion_region":[60,82],"ball_source_sha256":digest(ball),"ball_region":[2191,2203],
            "launch_source_sha256":digest(launch),"launch_region":[898,926],"scaffold_sha256":digest(path.read_bytes()),"library_sha256":digest(library.read_bytes()),
            "execution_receipt":".aegis/destiny/migration/world-missile-launch-oracle-build-execution.json",
            "execution_sha256":digest((STATE/"world-missile-launch-oracle-build-execution.json").read_bytes()),
            "scope":"Original stationary non-aimed numeric launch with real Carbon 1.2.2 and pinned SSE2 non-FMA DirectX math. Admission, callbacks, original Blue/layout and other platform branches remain separate."}


def main():
    directory = STATE / "missile-oracle"
    directory.mkdir(parents=True, exist_ok=True)
    source = blob(LATEST, "src/Ballpark.cpp")
    header = blob(LATEST, "src/Vector3d.h")
    lines = source.decode().splitlines(keepends=True)
    regions = [(751, 776), (972, 1015), (1398, 1426), (3552, 3552), (3857, 3868), (4958, 4966), (4968, 4983)]
    selected = {start: "".join(lines[start-1:end]) for start, end in regions}
    (directory / "Vector3d.h").write_bytes(header)
    scaffold = '''#include <cmath>
#include <cstdint>
#include <cstring>
#include <map>
struct Vector3 { float x,y,z; Vector3(float,float,float); };
#include "Vector3d.h"
using ID=int64_t;
enum { DSTBALL_MISSILE=5 };
struct Ball {
 Vector3d mNewPos,mNewVel,mGoto; Ball* mFollowPtr;
 double mMass,mTimeFactor; float mAgility,mMaxVel,mRadius,mFollowRange,mSpeedFraction,mRotAgility;
 long mEffectStamp; ID mFollowId,mOwnerId; int mMode;
};
struct Ballpark {
 double mFriction,dt; long mCurrentTime; int mTickInterval; std::map<ID,Ball*> mBalls;
 Vector3d EvolveMissile(Ball*);
 Vector3d GotoThrust(const Ball*,const Vector3d&,bool missile=false);
 void SetBallTimeFactor(Ball*);
 void SetBallAngularAgility(const ID&,float);
 void Integrate(Vector3d&,Vector3d&,const Vector3d&,double,double,double,double);
 void SourceFieldInitialization(Ball* missile,Ball* target,const ID& dstId,const ID& ownerId) {
''' + selected[3857] + '''
 }
};
''' + selected[751] + selected[972] + selected[1398] + selected[4958] + selected[4968] + '''
extern "C" void destiny_original_missile(const double* input,double* output,long elapsed,int tick_ms) {
 Ballpark park; park.mFriction=input[17]; park.dt=double(tick_ms)/1000.0; park.mCurrentTime=0; park.mTickInterval=tick_ms;
 Ball ball{},target{};
 ball.mNewPos=Vector3d(input[0],input[1],input[2]); ball.mNewVel=Vector3d(input[3],input[4],input[5]);
 target.mNewPos=Vector3d(input[6],input[7],input[8]); target.mNewVel=Vector3d(input[9],input[10],input[11]);
 ball.mMass=input[12]; ball.mAgility=float(input[13]); ball.mMaxVel=float(input[14]);
 ball.mRadius=float(input[15]); target.mRadius=float(input[16]);
 park.SourceFieldInitialization(&ball,&target,2,3); ball.mEffectStamp=-elapsed; ball.mMode=DSTBALL_MISSILE;
 park.SetBallTimeFactor(&ball);
 Vector3d a=park.EvolveMissile(&ball),p=ball.mNewPos,v=ball.mNewVel;
 park.Integrate(p,v,a,ball.mMass*ball.mAgility,park.mFriction,ball.mTimeFactor,park.dt);
 const double values[]={a.x,a.y,a.z,ball.mGoto.x,ball.mGoto.y,ball.mGoto.z,p.x,p.y,p.z,v.x,v.y,v.z};
 for(int i=0;i<12;i++) output[i]=values[i];
}
extern "C" uint32_t destiny_original_raw_rotational(uint32_t bits) {
 Ballpark park; Ball ball{}; std::memcpy(&ball.mRotAgility,&bits,4); park.mBalls[1]=&ball;
 park.SetBallAngularAgility(1,ball.mRotAgility);
 uint32_t result; std::memcpy(&result,&ball.mRotAgility,4); return result;
}
extern "C" uint32_t destiny_original_constructor_rotational(uint32_t bits) {
 Ball storage{}; Ball* ball=&storage; float rotAgility; std::memcpy(&rotAgility,&bits,4);
''' + selected[3552] + '''
 uint32_t result; std::memcpy(&result,&ball->mRotAgility,4); return result;
}
'''
    path = directory / "missile_oracle.cpp"
    library = directory / "liboriginal_missile.so"
    path.write_text(scaffold)
    checked(["clang++", "-std=c++17", "-O2", "-shared", "-fPIC", "-fno-fast-math", "-ffp-contract=off", str(path), "-o", str(library)], "world-missile-oracle-build", 60)
    version = checked(["clang++", "--version"], "world-missile-oracle-compiler")
    loaded = ctypes.CDLL(str(library))
    call = loaded.destiny_original_missile
    call.argtypes = [ctypes.POINTER(ctypes.c_double), ctypes.POINTER(ctypes.c_double), ctypes.c_long, ctypes.c_int]
    call.restype = None
    scalar = loaded.destiny_original_raw_rotational
    scalar.argtypes = [ctypes.c_uint32]
    scalar.restype = ctypes.c_uint32
    constructor = loaded.destiny_original_constructor_rotational
    constructor.argtypes = [ctypes.c_uint32]
    constructor.restype = ctypes.c_uint32
    rng = random.Random(114972)
    rows = []
    for index in range(42):
        position = [rng.uniform(-1000,1000) for _ in range(3)]
        velocity = [rng.uniform(-300,300) for _ in range(3)]
        target = [rng.uniform(-10000,10000) for _ in range(3)]
        target_velocity = [rng.uniform(-30,30) for _ in range(3)]
        if index == 0:
            position,velocity,target,target_velocity = [0.0]*3,[1.0,0.0,0.0],[1e6,0.0,0.0],[0.0]*3
        if index == 1:
            target = position.copy()  # late-phase zero-distance branch
        if index == 2:
            velocity = [0.0]*3
        mass,agility,maximum = 10.0**rng.uniform(3,8), rng.uniform(0.01,1.5),rng.uniform(30,500)
        radius,target_radius,friction = rng.uniform(0.1,5),rng.uniform(0.1,5),10.0**rng.uniform(2,7)
        tick_ms = [200,400,800,1000,1600,1][index%6]
        elapsed = [0,5,1,2,4,800][index%6]
        values = [*position,*velocity,*target,*target_velocity,mass,agility,maximum,radius,target_radius,friction]
        output = (ctypes.c_double*12)()
        call((ctypes.c_double*18)(*values),output,elapsed,tick_ms)
        rows.append({"input":values,"elapsed_ticks":elapsed,"tick_ms":tick_ms,
                     "expected_bits":[struct.pack(">d",value).hex() for value in output]})
    words = [0,0x80000000,1,0x80000001,0x3dcccccd,0xbdcccccd,0x3f000000,0xbf000000,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc01234,0x7f801234,0xffc04321]
    scalars = [{"input_bits":word,"expected_bits":scalar(word),"constructor_expected_bits":constructor(word)} for word in words]
    rotations = rotation_samples(directory)
    report = {"schema":1,"original_commit":LATEST,"seed":114972,"source_sha256":digest(source),
              "header_sha256":digest(header),"source_regions":regions,"scaffold_sha256":digest(path.read_bytes()),
              "library_sha256":digest(library.read_bytes()),"compiler":version,"checker_sha256":digest(Path(__file__).read_bytes()),
              "long_bytes":ctypes.sizeof(ctypes.c_long),"int_bytes":ctypes.sizeof(ctypes.c_int),
              "execution_receipt":".aegis/destiny/migration/world-missile-oracle-build-execution.json",
              "execution_sha256":digest((STATE/"world-missile-oracle-build-execution.json").read_bytes()),
              "rows":rows,"scalar_rows":scalars,"rotation":rotations,"proof_credit":False,
              "scope":"42 collision-free post-admission numeric transitions; source field-initialization assignments execute before mode is supplied at the EvolveMissile boundary. Covers 800ms equality, initial/late phases, zero distance, variable integer tick interval/friction/dt and 15 raw f32 field states. No admission/callback, Blue conversion/layout, collision or platform-matrix proof."}
    save(ROOT/"verification/migration/missile_oracle_samples.json",report)
    print(json.dumps({"status":"ORIGINAL_CPP_OBSERVATIONS","motion_cases":len(rows),"output_lanes":len(rows)*12,"scalar_cases":len(scalars),"rotation_cases":len(rotations["rows"]),"proof_credit":False}))


if __name__ == "__main__":
    main()
