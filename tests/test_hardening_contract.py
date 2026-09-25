from __future__ import annotations

from io import BytesIO
import json
import math
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._backend import InMemoryBackend  # noqa: E402
from destiny._errors import BackendCallError, UnsupportedTitleError  # noqa: E402
from destiny.net.client import ClientTickerInterface, TickErrorHandlerInterface, Ticker  # noqa: E402
from destiny.net.server import (  # noqa: E402
    Actions,
    BevyNetworkInterface,
    decode_carbon_value,
    encode_carbon_value,
)


def add_ball(
    park,
    ball_id,
    *,
    x=0.0,
    radius=1.0,
    velocity=(0.0, 0.0, 0.0),
    is_global=False,
    is_interactive=True,
):
    return park.AddBall(
        ball_id,
        5.0,
        radius,
        500.0,
        True,
        is_global,
        True,
        is_interactive,
        False,
        x,
        0.0,
        0.0,
        *velocity,
        0.25,
        1.0,
    )


class HardeningContractTests(unittest.TestCase):
    def setUp(self) -> None:
        destiny.settings.Reset()
        destiny.use_in_memory_backend()

    def tearDown(self) -> None:
        destiny.settings.Reset()
        destiny.clear_backend()

    def test_exact_space_damping_and_first_tick_bubble_assignment(self) -> None:
        park = destiny.Ballpark()
        ball = add_ball(park, 1, velocity=(10.0, 0.0, 0.0))
        add_ball(park, 2, x=2.0)
        park.tickInterval = 100.0
        park.friction = 5.0
        self.assertEqual(park.GetBallIdsInRange(1, 10.0), [])

        park.Evolve()
        x = (5.0 / ball.mass / ball.Agility) * 0.1
        expected_velocity = 10.0 * math.exp(-x)
        expected_position = 10.0 * (-math.expm1(-x) / x) * 0.1
        self.assertAlmostEqual(ball.vx, expected_velocity)
        self.assertAlmostEqual(ball.x, expected_position)
        self.assertEqual((ball.oldBubbleId, ball.newBubbleId), (-1, 0))
        self.assertEqual(park.GetBallIdsInRange(1, 10.0), [2])

    def test_delayed_removal_and_readd_cancellation(self) -> None:
        park = destiny.Ballpark()
        add_ball(park, 7)
        park.RemoveBall(7, 2)
        self.assertTrue(park.HasBall(7))
        park.Evolve()
        self.assertTrue(park.HasBall(7))

        replacement = add_ball(park, 7, x=9.0)
        self.assertEqual(replacement.x, 9.0)
        park.Evolve()
        self.assertTrue(park.HasBall(7))

        park.RemoveBall(7, 1)
        park.Evolve()
        self.assertFalse(park.HasBall(7))
        park.RemoveBall(404, 10)  # Missing-ball removal is a no-op.

    def test_proximity_sensor_replacement_owner_radius_and_cloak(self) -> None:
        park = destiny.Ballpark(True)
        owner = add_ball(park, 1, radius=2.0)
        candidate = add_ball(park, 2, x=6.0, radius=1.0)
        park.tickInterval = 100.0
        park.friction = 0.0
        park.Evolve()  # Assign compatibility bubble zero.

        self.assertEqual(owner.AddProximitySensor(0.0, 0.1), None)
        self.assertEqual(owner.AddProximitySensor(3.0, 0.1), None)
        park.Evolve()
        events = park.DrainProximityEvents()
        self.assertEqual(
            [(event["owner_id"], event["other_id"], event["entering"]) for event in events],
            [(1, 2, True)],
        )

        self.assertTrue(candidate.isMassive)
        park.CloakBall(2, destiny.DSTNORMALCLOAK, 10.0)
        self.assertFalse(candidate.isMassive)
        self.assertNotIn(2, park.GetBallIdsInRange(1, 20.0))
        park.UncloakBall(2)
        self.assertTrue(candidate.isMassive)

    def test_snapshot_schema_is_strict_duplicate_safe_and_atomic(self) -> None:
        park = destiny.Ballpark()
        add_ball(park, 1, x=12.0)
        stream = BytesIO()
        park.WriteFullStateToStream(stream)
        payload = json.loads(stream.getvalue())
        payload["balls"][0]["unexpected"] = True

        with self.assertRaises(BackendCallError):
            park.ReadFullStateFromStream(BytesIO(json.dumps(payload).encode()))
        self.assertEqual(park.GetBall(1).x, 12.0)

        original = stream.getvalue().decode("utf-8")
        duplicate = original.replace(
            '{"format":"destiny-bevy-compat-state-v3"',
            '{"format":"destiny-bevy-compat-state-v3","format":"destiny-bevy-compat-state-v3"',
            1,
        )
        with self.assertRaises(BackendCallError):
            park.ReadFullStateFromStream(BytesIO(duplicate.encode()))
        self.assertEqual(park.GetBall(1).x, 12.0)

    def test_partial_snapshot_modes_preserve_or_replace_child_shapes(self) -> None:
        source = destiny.Ballpark()
        incoming = add_ball(source, 1)
        incoming.AddMiniBall(1.0, 0.0, 0.0, 0.5)
        data = BytesIO()
        source.WriteFullStateToStream(data)

        preserve = destiny.Ballpark()
        existing = add_ball(preserve, 1)
        existing.AddMiniBall(9.0, 0.0, 0.0, 0.5)
        preserve.ReadFullStateFromStream(data, 1)
        result = BytesIO()
        preserve.WriteFullStateToStream(result)
        self.assertEqual(json.loads(result.getvalue())["balls"][0]["minis"][0]["position"][0], 9.0)

        replace = destiny.Ballpark()
        old = add_ball(replace, 1)
        old.AddMiniBall(9.0, 0.0, 0.0, 0.5)
        replace.ReadFullStateFromStream(data, 2)
        result = BytesIO()
        replace.WriteFullStateToStream(result)
        self.assertEqual(json.loads(result.getvalue())["balls"][0]["minis"][0]["position"][0], 1.0)

    def test_source_filtered_snapshot_respects_cloak_and_global_partition(self) -> None:
        park = destiny.Ballpark(True)
        add_ball(park, 1)
        add_ball(park, 2, x=2.0)
        add_ball(park, 3, x=3.0, is_global=True)
        park.Evolve()
        park.CloakBall(2, destiny.DSTGMCLOAK)
        stream = BytesIO()
        park.WriteFullStateToStream(stream, 1)
        ids = [row["id"] for row in json.loads(stream.getvalue())["balls"]]
        self.assertEqual(ids, [1, 3])

    def test_query_missing_scan_and_large_numeric_policies(self) -> None:
        park = destiny.Ballpark()
        self.assertIsNone(park.GetCenterDist(1, 2))
        self.assertIsNone(park.GetSurfaceDist(1, 2))
        add_ball(park, 1)
        add_ball(park, -2, x=2.0)
        add_ball(park, 3, x=3.0)
        add_ball(park, 4, x=-3.0)
        park.Evolve()
        self.assertEqual(park.ScanCone(1, math.pi / 2, 10.0, 1.0, 0.0, 0.0), [3])
        with self.assertRaises(ValueError):
            park.SetBallPosition(1, 10**1000, 0.0, 0.0)

    def test_extreme_finite_numeric_operations_are_safe_and_atomic(self) -> None:
        limit = sys.float_info.max
        park = destiny.Ballpark()
        first = add_ball(park, 1)
        add_ball(park, 2, x=3.0)
        park.Evolve()

        park.SetBallRotation(1, limit, limit, limit, limit)
        self.assertTrue(all(math.isfinite(value) for value in (first.rx, first.ry, first.rz, first.rw)))
        self.assertAlmostEqual(first.rw, 0.5)

        angular_before = (first.wx, first.wy, first.wz)
        with self.assertRaises(OverflowError):
            first.ApplyImpulsiveForceAtPosition(
                (limit, limit, limit),
                (limit, -limit, limit),
            )
        self.assertEqual((first.wx, first.wy, first.wz), angular_before)

        with self.assertRaises(ValueError):
            park.SetBallRadius(2, limit)
        with self.assertRaises(OverflowError):
            park.GetBallIdsInCapsule(1, limit, limit, limit, 1.0)

        with self.assertRaises(ValueError):
            park.SetBallPosition(1, limit, 0.0, 0.0)

        atomic = destiny.Ballpark()
        moving = add_ball(atomic, 10, x=1.0e100)
        atomic.friction = 0.0
        atomic.tickInterval = 2000.0
        atomic.SetMaxSpeed(10, 1.0e100)
        atomic.SetBallVelocity(10, 1.0e100, 0.0, 0.0)
        with self.assertRaises(OverflowError):
            atomic.Evolve()
        self.assertEqual(atomic.currentTime, 0)
        self.assertEqual(moving.x, 1.0e100)

    def test_network_codec_is_versioned_bounded_and_binary_safe(self) -> None:
        value = [1, ("event", (b"\x00\xff", {"ok": True}))]
        encoded = encode_carbon_value(value)
        self.assertEqual(decode_carbon_value(encoded), [1, ("event", (b"\x00\xff", {"ok": True}))])
        with self.assertRaises(ValueError):
            encode_carbon_value({"$destiny_bevy_bytes_v1": "collision"})
        with self.assertRaises(OverflowError):
            encode_carbon_value(2**63)

        backend = destiny.get_backend()
        park = destiny.Ballpark()
        interface = BevyNetworkInterface(backend, park)
        interface.singlecast([(7, "DoDestinyUpdate", value)])
        outbox = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        envelope = outbox["singlecasts"][0]
        self.assertEqual(envelope["mode"], "singlecast")
        self.assertEqual(BevyNetworkInterface.decode_envelope(envelope)[0][0], 7)
        with self.assertRaises(ValueError):
            BevyNetworkInterface.decode_envelope({**envelope, "extra": 1})

    def test_unsupported_actions_are_rejected_before_queueing(self) -> None:
        park = destiny.Ballpark(True)
        actions = Actions(park)
        with self.assertRaises(UnsupportedTitleError):
            actions.go_to_point(99, 1.0, 2.0, 3.0)
        self.assertEqual(actions.flush_history(), [])
        with self.assertRaises(RuntimeError):
            actions.set_ball_angular_velocity(99, 1.0, 2.0, 3.0)

    def test_retired_backend_lives_until_last_park_closes(self) -> None:
        class TrackingBackend(InMemoryBackend):
            def __init__(self):
                super().__init__()
                self.close_count = 0

            def close(self):
                self.close_count += 1
                super().close()

        first_backend = TrackingBackend()
        destiny.set_backend(first_backend)
        park = destiny.Ballpark()
        ball = add_ball(park, 1, x=4.0)

        second_backend = TrackingBackend()
        destiny.set_backend(second_backend)
        self.assertEqual(first_backend.close_count, 0)
        self.assertEqual(ball.x, 4.0)
        park.close()
        self.assertEqual(first_backend.close_count, 1)
        with self.assertRaises(BackendCallError):
            _ = ball.x

    def test_start_driver_and_close_are_per_park(self) -> None:
        park = destiny.Ballpark()
        park.tickInterval = 10.0
        park.Start()
        thread = park._driver_thread
        self.assertIsNotNone(thread)
        self.assertTrue(thread.is_alive())
        park.Pause()
        self.assertFalse(park.isRunning)
        self.assertIsNone(park._driver_thread)
        park.close()
        with self.assertRaises(BackendCallError):
            park.Evolve()


class _ErrorHandler(TickErrorHandlerInterface):
    def __init__(self):
        self.fatal = 0
        self.recoverable = 0

    def on_fatal_desync(self):
        self.fatal += 1

    def on_recoverable_desync(self):
        self.recoverable += 1


class _ClientInterface(ClientTickerInterface):
    def on_set_state(self):
        pass

    def on_ballpark_local_action(self, func_name, args):
        pass

    def should_log_actions(self):
        return False

    def get_ball_destruction_delay(self, ball, is_terminal=False):
        return 0

    def get_ball_destruction_delays(self, ball_ids, is_release=False):
        return {ball_id: 0 for ball_id in ball_ids}

    def clean_up_after_ball_removal(self, ball_id, ball, is_terminal=False):
        pass

    def clean_up_after_multiple_ball_removal(self, ball_ids, is_release=False):
        pass


class ClientFailurePolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        destiny.use_in_memory_backend()

    def tearDown(self) -> None:
        destiny.clear_backend()

    def test_failed_client_action_invalidates_state_and_reports_fatal_desync(self) -> None:
        park = destiny.Ballpark()
        handler = _ErrorHandler()
        ticker = Ticker(handler, _ClientInterface())
        ticker.set_ballpark(park)
        ticker._state_is_valid = True
        with self.assertLogs("destiny.net.client", level="ERROR"):
            ticker._real_flush_state([(0, ("DefinitelyNotAnAction", ()))])
        self.assertFalse(ticker._state_is_valid)
        self.assertEqual(handler.fatal, 1)


if __name__ == "__main__":
    unittest.main()
