from __future__ import annotations

import json
import math
import os
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._backend import InMemoryBackend, NativeBackend  # noqa: E402


NATIVE_LIBRARY = os.environ.get("DESTINY_BEVY_COMPAT_LIBRARY")


@unittest.skipUnless(NATIVE_LIBRARY, "native differential gate requires a built library")
class NativeDifferentialTests(unittest.TestCase):
    def setUp(self) -> None:
        self.native = NativeBackend(NATIVE_LIBRARY)
        self.contract = InMemoryBackend()

    def tearDown(self) -> None:
        self.native.close()
        self.contract.close()

    @staticmethod
    def run_scenario(backend):
        park = destiny.Ballpark(_backend=backend)
        try:
            first = park.AddBall(
                1, 5.0, 1.0, 100.0, True, False, True, True, False,
                0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.5, 1.0,
            )
            park.AddBall(
                2, 5.0, 1.0, 100.0, True, False, True, False, False,
                5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 1.0,
            )
            park.tickInterval = 100.0
            park.friction = 5.0
            park.Evolve()
            capture_time, capture = park.CaptureFullState()
            capture_payload = json.loads(capture)
            return {
                "current_time": park.currentTime,
                "position": first.x,
                "velocity": first.vx,
                "range": park.GetBallIdsInRange(1, 10.0),
                "box": tuple(park.GetBoxCenter(2, 1.0, -1.0, 0.0)),
                "membership": park.GetBubbleMembership(),
                "capture_time": capture_time,
                "capture_ids": [row["id"] for row in capture_payload["balls"]],
            }
        finally:
            park.close()

    def test_supported_motion_query_and_geometry_results_match(self) -> None:
        native = self.run_scenario(self.native)
        contract = self.run_scenario(self.contract)
        self.assertEqual(native["current_time"], contract["current_time"])
        self.assertEqual(native["range"], contract["range"])
        self.assertEqual(native["box"], contract["box"])
        self.assertEqual(native["membership"], contract["membership"])
        self.assertEqual(native["capture_time"], contract["capture_time"])
        self.assertEqual(native["capture_ids"], contract["capture_ids"])
        self.assertTrue(math.isclose(native["position"], contract["position"], rel_tol=1e-10))
        self.assertTrue(math.isclose(native["velocity"], contract["velocity"], rel_tol=1e-10))


if __name__ == "__main__":
    unittest.main()
