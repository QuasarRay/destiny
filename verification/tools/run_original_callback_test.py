#!/usr/bin/env python3
"""Run the unchanged pinned original callback scenario on a real native library.

The selected source and helper must match the behavioral Git objects. This is
runtime boundary evidence; it does not discharge Verus/Kani proof obligations.
"""
import argparse
import hashlib
from pathlib import Path
import subprocess
import sys
import unittest

ROOT=Path(__file__).resolve().parents[2]
PIN='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/ballpark/evolve/test_add.py'
HELPER='python/destiny/test/net/client/helpers.py'

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--original-root',required=True,type=Path)
    parser.add_argument('--library',required=True,type=Path)
    args=parser.parse_args()
    for source in (SOURCE,HELPER):
        expected=subprocess.run(['git','--no-replace-objects','show',PIN+':'+source],cwd=ROOT,
                                check=True,capture_output=True).stdout
        actual=(args.original_root/source).read_bytes()
        if actual!=expected:raise ValueError('Original source differs: '+source)
        print(source,hashlib.sha256(actual).hexdigest())
    if not args.library.is_file():raise ValueError('Real native library is required')
    sys.path.insert(0,str(ROOT/'python'))
    import destiny
    from destiny._backend import NativeBackend
    destiny.__path__.append(str(args.original_root.resolve()/'python/destiny'))
    backend=NativeBackend(args.library.resolve())
    destiny.set_backend(backend)
    try:
        name='destiny.test.ballpark.evolve.test_add.TestAdd.test_balls_can_not_be_added_during_evolve'
        suite=unittest.defaultTestLoader.loadTestsFromName(name)
        if suite.countTestCases()!=1:raise ValueError('Expected exactly one original case')
        result=unittest.TextTestRunner(verbosity=2).run(suite)
        return 0 if result.wasSuccessful() and result.testsRun==1 and not result.skipped else 1
    finally:backend.close()

if __name__=='__main__':raise SystemExit(main())
