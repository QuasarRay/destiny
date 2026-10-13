#!/usr/bin/env python3
"""Export this campaign's controller changes without copying or resetting it.

The generated patch and postimage manifest apply only to the recorded Git base.
No checkout, fetch, installation or credential mutation occurs here.
"""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]
BASE="46f90f7dbc4f3db8a1a74ef9d8aa91fb4b679460"
EXTRA=("infra/agentinfra/coverage.py","infra/tests/test_coverage.py")

def git(*arguments,check=True):
    return subprocess.run(["git","-C",str(ROOT/".agents"),*arguments],
                          capture_output=True,check=check)

def main():
    if git("rev-parse","HEAD").stdout.decode().strip()!=BASE:raise ValueError("Controller base changed")
    patch=git("diff","--binary","--no-ext-diff",BASE).stdout
    paths=git("diff","--name-only",BASE).stdout.decode().splitlines()
    for name in EXTRA:
        p=ROOT/".agents"/name
        if git("ls-files","--error-unmatch",name,check=False).returncode==0:
            raise ValueError("Expected added controller file became tracked: "+name)
        # --no-index emits correct Git patch data without staging user files.
        result=subprocess.run(["git","diff","--no-index","--binary","--no-ext-diff","/dev/null",name],
                              cwd=ROOT/".agents",capture_output=True)
        if result.returncode!=1:raise ValueError("Cannot export added controller file: "+name)
        patch+=result.stdout
        paths.append(name)
    directory=ROOT/"verification/controller";directory.mkdir(parents=True,exist_ok=True)
    (directory/"destiny-controller.patch").write_bytes(patch)
    manifest={"schema":1,"repository":"https://github.com/QuasarRay/Aegis.git","base_commit":BASE,
              "patch_sha256":hashlib.sha256(patch).hexdigest(),
              "postimages":{name:hashlib.sha256((ROOT/".agents"/name).read_bytes()).hexdigest() for name in sorted(paths)},
              "upstream_license":"No LICENSE or COPYING file is present at this supplied controller pin. Preserve any upstream notices; this patch does not declare a license for upstream Aegis.",
              "scope":"Existing catalog/provenance/strict evidence/fixture-qualified conjunction hardening and focused regressions only; no proof result is exported."}
    (directory/"base.json").write_text(json.dumps(manifest,indent=2)+"\n")
    print(json.dumps({"base_commit":BASE,"changed_files":len(paths),"patch_bytes":len(patch)}))

if __name__=="__main__":main()
