#!/usr/bin/env python3
"""Run the unchanged original formation-slot tests against a selected backend."""

import argparse
from pathlib import Path
import sys
import unittest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original-root", type=Path, required=True)
    parser.add_argument("--library", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root / "python"))
    import destiny
    from destiny._backend import InMemoryBackend, NativeBackend

    # Import the upstream tests/helpers through the rewritten Destiny package.
    # Test methods and assertions are left unchanged.
    destiny.__path__.append(str(args.original_root.resolve() / "python" / "destiny"))
    backend = NativeBackend(str(args.library.resolve())) if args.library else InMemoryBackend()
    destiny.set_backend(backend)
    names = [
        "destiny.test.test_ball.BallTest.test_reserve_formation_slot",
        "destiny.test.test_ball.BallTest.test_slots_are_reserved_in_incremental_order",
        "destiny.test.test_ball.BallTest.test_reserving_too_many_formation_slots_fails",
        "destiny.test.test_ball.BallTest.test_free_formation_slot",
        "destiny.test.ballpark.test_getters_and_setters.TestSetBallFormation.test_valid_formation_gets_set",
        "destiny.test.ballpark.test_getters_and_setters.TestSetBallFormation.test_formation_out_of_range_does_not_get_set",
    ]
    try:
        result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromNames(names))
        return 0 if result.wasSuccessful() else 1
    finally:
        backend.close()


if __name__ == "__main__":
    raise SystemExit(main())
