#!/usr/bin/env python3
"""Apply/validate the exported patch in an existing exact Aegis checkout.

Does not clone, fetch or reset. For CI, first use actions/checkout with the
repository and commit from verification/controller/base.json. Existing unrelated
changes fail closed; an already identical postimage checkout is reused.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout",type=Path,default=ROOT/".agents")
    parser.add_argument("--verify-only",action="store_true")
    args=parser.parse_args();checkout=args.checkout.resolve(strict=True)
    manifest=json.loads((ROOT/"verification/controller/base.json").read_text())
    patch=ROOT/"verification/controller/destiny-controller.patch"
    if digest(patch)!=manifest["patch_sha256"]:raise ValueError("Controller patch digest differs")
    def git(*arguments):
        return subprocess.run(["git","-C",str(checkout),*arguments],capture_output=True,check=True)
    if git("rev-parse","HEAD").stdout.decode().strip()!=manifest["base_commit"]:
        raise ValueError("Controller checkout is not at the recorded exact base")
    for name in manifest["postimages"]:
        p=checkout/name
        if p.is_symlink() or not p.resolve().is_relative_to(checkout):raise ValueError("Controller postimage path escapes checkout")
    def matches():
        return all((checkout/name).is_file() and digest(checkout/name)==sha for name,sha in manifest["postimages"].items())
    if not matches():
        if args.verify_only:raise ValueError("Controller postimage differs")
        if git("status","--porcelain","--untracked-files=all").stdout:
            raise ValueError("Controller has existing changes; preserve them and use a clean exact-base checkout")
        git("apply","--check",str(patch));git("apply",str(patch))
        if not matches():raise ValueError("Applied patch does not match its exported postimages")
    allowed=set(manifest["postimages"])
    for line in git("status","--porcelain","--untracked-files=all").stdout.decode().splitlines():
        if line[3:] not in allowed:raise ValueError("Unbound additional controller change: "+line[3:])
    print(json.dumps({"status":"CONTROLLER_POSTIMAGE_VALIDATED","base_commit":manifest["base_commit"],
                      "files":len(allowed),"notice":"Source reproducibility only; no proof credit."}))

if __name__=="__main__":main()
