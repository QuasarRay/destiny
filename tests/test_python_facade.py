from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._errors import UnsupportedTitleError  # noqa: E402


def add_ball(park: destiny.Ballpark, ball_id: int, *, x: float, radius: float = 2.0):
    # Exact old-style Destiny AddBall argument order from Thunkers.cpp.
    return park.AddBall(
        ball_id,
        10.0,
        radius,
        100.0,
        True,
        False,
        True,
        True,
        False,
        x,
        2.0,
        3.0,
        4.0,
        5.0,
        6.0,
        0.5,
        0.75,
    )


class PythonFacadeContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.backend = destiny.use_in_memory_backend()

    def tearDown(self) -> None:
        destiny.settings.Reset()
        destiny.clear_backend()

    def test_public_root_surface_and_enums_exist(self) -> None:
        expected = {
            "Ball",
            "Ballpark",
            "Capsule",
            "ClientBall",
            "MiniBall",
            "MiniBox",
            "MiniCapsule",
            "OrientedBox",
            "SettingsConfiguration",
            "DstBallMode",
            "DstConstants",
            "DstEventType",
            "GetBoxCenter",
            "settings",
        }
        self.assertLessEqual(expected, set(dir(destiny)))
        self.assertEqual(destiny.DstBallMode.DSTBALL_RIGID, 11)
        self.assertEqual(destiny.DstEventType.DST_COLLISION, 6)
        self.assertEqual(destiny.DstConstants.DSTLOCALBALLS, -1073741824)

    def test_all_declared_python_types_are_importable(self) -> None:
        from destiny.net.client import BaseTicker as ClientBaseTicker
        from destiny.net.client import ClientTickerInterface, TickErrorHandlerInterface, Ticker as ClientTicker
        from destiny.net.server import (
            Actions,
            BallInfoInterface,
            BaseTicker as ServerBaseTicker,
            BubbleUpdater,
            CharacterInterestsInterface,
            ClientInterestsInterface,
            NetworkInterface,
            ParkUpdateBatcher,
            Ticker as ServerTicker,
        )

        for exported in (
            ClientBaseTicker,
            ClientTicker,
            ClientTickerInterface,
            TickErrorHandlerInterface,
            Actions,
            BallInfoInterface,
            ServerBaseTicker,
            BubbleUpdater,
            CharacterInterestsInterface,
            ClientInterestsInterface,
            NetworkInterface,
            ParkUpdateBatcher,
            ServerTicker,
        ):
            self.assertIsInstance(exported.__name__, str)

    def test_ballpark_and_ball_direct_equivalents_round_trip(self) -> None:
        park = destiny.Ballpark(True)
        ball = add_ball(park, 101, x=1.0)

        self.assertIsInstance(ball, destiny.Ball)
        self.assertEqual(ball.id, 101)
        self.assertEqual((ball.x, ball.y, ball.z), (1.0, 2.0, 3.0))
        self.assertEqual((ball.vx, ball.vy, ball.vz), (4.0, 5.0, 6.0))
        self.assertEqual(ball.mass, 10.0)
        self.assertEqual(ball.radius, 2.0)
        self.assertEqual(ball.maxVelocity, 100.0)

        ball.x = 9.5
        ball.mass = 12.0
        park.SetBallVelocity(101, -1.0, -2.0, -3.0)
        park.SetBallAngularVelocity(101, 0.1, 0.2, 0.3)
        park.SetBallRotation(101, 0.0, 0.0, 0.0, 1.0)
        park.SetMaxSpeed(101, 55.0)
        park.SetMaxAngularSpeed(101, 2.5)

        self.assertEqual(ball.x, 9.5)
        self.assertEqual(ball.mass, 12.0)
        self.assertEqual((ball.vx, ball.vy, ball.vz), (-1.0, -2.0, -3.0))
        # Rotation replacement invalidates the former angular derivative.
        self.assertEqual((ball.wx, ball.wy, ball.wz), (0.0, 0.0, 0.0))
        for actual, expected in zip((ball.rx, ball.ry, ball.rz, ball.rw), (0.0, 0.0, 0.0, 1.0)):
            self.assertAlmostEqual(actual, expected)
        self.assertEqual(ball.maxVelocity, 55.0)
        self.assertEqual(ball.maxAngularVelocity, 2.5)

        self.assertEqual(park.time, 0)
        park.AdjustTimes(2)
        park.AdjustTimes(3)
        self.assertEqual(park.time, 5)

        other = add_ball(park, 102, x=19.5, radius=2.0)
        park.ego = ball.id
        self.assertEqual(park.GetCenterDist(ball.id, ball.id), 0.0)
        self.assertAlmostEqual(park.GetCenterDist(ball.id, other.id), 10.0)
        self.assertAlmostEqual(park.GetSurfaceDist(ball.id, other.id), 6.0)

    def test_ballpark_lifecycle_and_surface_distance(self) -> None:
        park = destiny.Ballpark()
        first = add_ball(park, 1, x=0.0, radius=2.0)
        second = add_ball(park, 2, x=10.0, radius=3.0)

        self.assertFalse(park.isRunning)
        self.assertEqual(park.tickInterval, 1000.0)
        self.assertEqual(park.friction, 1_000_000.0)
        park.Start()
        self.assertTrue(park.isRunning)
        park.Pause()
        self.assertFalse(park.isRunning)
        park.Start()
        self.assertTrue(park.isRunning)

        park.tickInterval = 25
        park.friction = 0.15
        self.assertEqual(park.tickInterval, 25)
        self.assertAlmostEqual(park.friction, 0.15)
        self.assertAlmostEqual(park.GetSurfaceDist(first.id, second.id), 5.0)
        self.assertTrue(park.HasBall(2))
        park.RemoveBall(2)
        self.assertFalse(park.HasBall(2))

    def test_quaternion_and_euler_fields_share_one_rotation(self) -> None:
        park = destiny.Ballpark()
        ball = add_ball(park, 1, x=0.0)
        park.SetBallRotation(1, math.sin(math.pi / 8), 0.0, 0.0, math.cos(math.pi / 8))
        self.assertAlmostEqual(ball.roll, math.pi / 4)
        with self.assertRaises(AttributeError):
            ball.roll = 0.0
        norm = math.sqrt(ball.rx**2 + ball.ry**2 + ball.rz**2 + ball.rw**2)
        self.assertAlmostEqual(norm, 1.0)

    def test_unsupported_titles_fail_explicitly(self) -> None:
        park = destiny.Ballpark()
        add_ball(park, 1, x=0.0)
        with self.assertRaises(UnsupportedTitleError) as caught:
            park.Orbit(1, 2, 1000.0)
        self.assertEqual(caught.exception.canonical_title, "destiny.Ballpark.Orbit")
        self.assertEqual(caught.exception.verdict, "NONE")

    def test_settings_are_carbon_compatible_value_objects(self) -> None:
        defaults = destiny.settings.GetDefault()
        self.assertEqual(defaults.collisionMaxIterations, 20)
        self.assertFalse(defaults.useIterativeCollision)

        changed = destiny.SettingsConfiguration()
        changed.collisionMaxIterations = 32
        changed.useIterativeCollision = True
        destiny.settings.Apply(changed)
        current = destiny.settings.Get()
        self.assertEqual(current.collisionMaxIterations, 32)
        self.assertTrue(current.useIterativeCollision)

        destiny.settings.Reset()
        self.assertEqual(destiny.settings.Get().collisionMaxIterations, 20)
        invalid = destiny.SettingsConfiguration(collisionMaxIterations=0)
        with self.assertRaises(ValueError):
            destiny.settings.Apply(invalid)

    def test_get_box_center_and_client_history_helper(self) -> None:
        from destiny.net.client import merge_state_into_history

        actual = destiny.GetBoxCenter((2, 1, -1, 0))
        # Level 2 has 491,520 m cells in the legacy Destiny partition.
        for value, expected in zip(actual, (245760.0, -245760.0, 245760.0)):
            self.assertAlmostEqual(value, expected)
        history: list = []
        merge_state_into_history([(12, "a"), (10, "b")], history, True)
        self.assertEqual(history, [[[(10, "b")], True], [[(12, "a")], True]])


if __name__ == "__main__":
    unittest.main()
