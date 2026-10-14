#!/usr/bin/env python3
"""Extend the reviewed source comparison using local immutable Git objects.

Line regions record the definitions actually inspected for this implementation.
Hash equality alone is not semantic review, compiler equivalence or proof.
No original code is executed and no reference is fetched.
"""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]
REVISIONS=("ef738406b61ad06bae2f402857e464735ab106a2","114e89fa584f0e1a81eb0a679aabf41af8870072")
REGIONS={
    "src/Settings.cpp": [[1,65]],
    "src/Settings.h": [[1,49]],
    "src/Settings_Blue.cpp": [[1,66]],
    "src/Ballpark.cpp": [[59,66],[85,150],[332,403],[421,1022],[1023,1350],[1398,1513],[1988,2713],[2745,3020],[3013,3036],
        [3041,3105],[3105,3206],[3265,3314],[3320,3502],[3568,3640],[3716,3788],[3790,4071],[4072,4418],[4471,4555],[4556,4650],[4716,4781],[4887,4994],[5026,5069],[5137,5154],
        [4852,4883],[5156,5221],[5223,5389],[5415,5585],[5471,5591],[5820,5954],[5896,5961],[5956,6060],[6154,6214],[6230,6286],[6278,6350],[6429,6618],[6667,6728],[6730,6768]],
    "src/Partition.cpp": [[20,161],[182,238],[279,537]],
    "src/Partition.h": [[36,110],[181,195]],
    "src/Box.cpp": [[18,109],[155,208],[216,277]],
    "src/Box.h": [[13,73]],
    "src/MapOfBalls.h": [[10,135]],
    "src/Ball.cpp": [[22,160],[244,296],[335,815],[812,881],[905,941],[1499,1632],
        [1663,1875],[1876,1923],[2098,2238],[2354,2548]],
    "src/Ball.h": [[20,23],[70,79],[121,184],[340,364],[415,422]],
    "src/Ball_Blue.cpp": [[42,65],[308,326],[546,586]],
    "src/Thunkers.cpp": [[66,187],[304,324],[356,430],[855,975],[1121,1250],[1330,1800],[1822,1879],[1948,2043],
        [2044,2077],[2083,2260],[2260,2377],[2379,2505],[2509,2894],[2897,3381]],
    "src/Capsule.cpp": [[1,35],[197,581]],
    "src/OrientedBox.cpp": [[1,291]],
    "src/StaticCollidable.cpp": [[1,63]],
    "src/CollisionBallProperties.cpp": [[1,260]],
    "src/AABB.cpp": [[57,63]],
    "src/IDstConstants.h": [[1,82]],
    "python/destiny/test/helpers.py": [[1,288]],
    "python/destiny/test/net/client/helpers.py": [[1,47]],
    "python/destiny/net/client/_ticker.py": [[1,340]],
    "python/destiny/net/client/_baseticker.py": [[1,130]],
    "python/destiny/_util/settings.py": [[1,31]],
    "python/destiny/_util/decometaclass.py": [[1,74]],
    "python/destiny/net/server/_parkupdatebatcher.py": [[1,343]],
    "python/destiny/net/server/_actions.py": [[311,383],[678,708]],
    "python/destiny/test/net/server/helpers.py": [[17,230]],
    "python/destiny/net/server/_bubbleupdater.py": [[11,54]],
    "python/destiny/net/const.py": [[1,8]],
}

def read(revision,path):
    return subprocess.run(["git","--no-replace-objects","show",revision+":"+path],
                          cwd=ROOT,capture_output=True,check=True).stdout

def main():
    path=ROOT/"verification/cases/reference_review.json"
    document=json.loads(path.read_text())
    indexed={f["path"]:f for f in document["files"]}
    for source,regions in REGIONS.items():
        blobs={r:read(r,source) for r in REVISIONS}
        line_count = len(blobs[REVISIONS[0]].splitlines())
        if any(not 1 <= start <= end <= line_count for start,end in regions):
            raise ValueError(f"Review region outside {source}: {regions}, {line_count} lines")
        identical=len(set(blobs.values()))==1
        include_only=(not identical and len({b.replace(b'"stdafx.h"',b'"StdAfx.h"') for b in blobs.values()})==1)
        record=indexed.setdefault(source,{"path":source})
        record["sha256"]={r:hashlib.sha256(b).hexdigest() for r,b in blobs.items()}
        record["status"]="IDENTICAL" if identical else "INCLUDE_SPELLING_ONLY" if include_only else "DIFFERS"
        record["reviewed_regions_at_behavioral_revision"]=regions
        record["region_hashes"]=[{"start":start,"end":end,"sha256":hashlib.sha256(
            b"\n".join(blobs[REVISIONS[0]].splitlines()[start-1:end])+b"\n").hexdigest()} for start,end in regions]
        record["review_scope"]="Definitions used by the implemented concrete scenario projection; all other definitions and native correspondence remain unreviewed."
        if source=="src/Partition.h":
            record["difference_review"]=("The behavioral revision hashes Key using (ix<<3)+(iy<<2)+(iz<<1)+n; "
                "the CI revision introduces KeyHash combining each field. Clamping/equality and grid keys match, "
                "but unordered-map iteration and build/hasher correspondence are not established. The "
                "spherical component model uses exact key equality and the undirected neighbour graph, "
                "not an assumed native hash order. GetActiveBoxes list order is established only for "
                "zero/one cell; other translated tests observe sets or sorted membership through an "
                "explicit adapter. Arbitrary native enumeration order remains unimplemented.")
    document["files"]=list(indexed.values())
    document["numeric_dependency_review"]={
        "registry_baseline":"2027e0c9d34193f468438589c4e6bb25996e823d",
        "baseline_path":"versions/baseline.json",
        "baseline_sha256":hashlib.sha256(subprocess.run(["git","show",
            "2027e0c9d34193f468438589c4e6bb25996e823d:versions/baseline.json"],
            cwd=ROOT/"vendor/github.com/carbonengine/vcpkg-registry",capture_output=True,check=True).stdout).hexdigest(),
        "python":"3.12.9#4", "carbon_math":"1.2.1#0",
        "python_sum_algorithm":"https://github.com/python/cpython/blob/v3.12.9/Python/bltinmodule.c#L2393-L2477",
        "scope":"Three exact float objects and initial integer zero; compensated fast path ported. Interpreter/build/libm and object correspondence remain unproved. The URL is review provenance, not a downloaded artifact or measured source-file hash.",
        "carbon_rotation":"Exact Carbon math 1.2.1 commit 50f7c65241b9e22514fdd0a7a159e86322e4d53e and builtin-registry DirectXMath apr2025 commit 56206b1916646b29b37e1eac56e96e2008d6c71f reviewed. Linux SSE2 non-FMA rotation operations ported. Every retained artifact matches its Git blob ID; verification/cases/math_dependencies.json records actual SHA256 and licenses. Other target branches, compiler flags/rounding, Python list identity and native numerical correspondence remain unproved.",
    }
    document["implementation_review"]=("Previously reviewed geometry/collision/box/formation/settings/history "
        "definitions retained. Additional consumed partition/initialization/query/attribute-notification and "
        "batcher, collision/evolution/Orbit/callback/visibility/remote-follower and static partition "
        "definitions have explicit line-region hashes. NaN inflated radius affects comparisons, not the "
        "coordinate cast; finite-coordinate insertion is retained. Partition.h has a substantive hasher difference. "
        "File comparisons and region review do not establish native allocation, Python, compiler, math or "
        "production refinement. Missing include/Ball.h is retained as a historical unresolved path; the "
        "actual consumed definition is src/Ball.h.")
    document["generator_sha256"]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    path.write_text(json.dumps(document,indent=2,sort_keys=True)+"\n")
    print(json.dumps({"source_files":len(indexed),"region_reviews":len(REGIONS),"path":str(path)}))

if __name__=="__main__":main()
