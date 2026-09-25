from __future__ import annotations

from array import array
from gc import collect
from io import BytesIO
import json
from pathlib import Path
import sys
from threading import Event
import time
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._backend import InMemoryBackend  # noqa: E402
from destiny._errors import BackendCallError  # noqa: E402
from destiny._facade import (  # noqa: E402
    _decode_snapshot_response,
    _read_stream,
    _write_stream,
)
from destiny._util.signal import Signal  # noqa: E402
from destiny.net import _codec as carbon_codec  # noqa: E402
from destiny.net._codec import I64_MAX  # noqa: E402
from destiny.net.client import ClientTickerInterface, TickErrorHandlerInterface, Ticker  # noqa: E402
from destiny.net.server import BevyNetworkInterface  # noqa: E402


def add_ball(park, ball_id, *, x=0.0, y=0.0, radius=1.0):
    return park.AddBall(
        ball_id, 5.0, radius, 500.0, True, False, True, True, False,
        x, y, 0.0, 0.0, 0.0, 0.0, 0.25, 1.0,
    )


def snapshot_payload(park):
    stream = BytesIO()
    park.WriteFullStateToStream(stream)
    return json.loads(stream.getvalue())


class _Errors(TickErrorHandlerInterface):
    def __init__(self):
        self.fatal = 0
        self.recoverable = 0

    def on_fatal_desync(self):
        self.fatal += 1

    def on_recoverable_desync(self):
        self.recoverable += 1


class _Client(ClientTickerInterface):
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


class V3RegressionTests(unittest.TestCase):
    def setUp(self):
        destiny.settings.Reset()
        destiny.use_in_memory_backend()

    def tearDown(self):
        destiny.settings.Reset()
        destiny.clear_backend()

    def test_failed_set_state_preserves_live_state_and_invalidates_ticker(self):
        park = destiny.Ballpark()
        ball = add_ball(park, 1, x=12.0)
        park.ego = 1
        errors = _Errors()
        ticker = Ticker(errors, _Client())
        ticker.set_ballpark(park)
        ticker._state_is_valid = True

        with self.assertLogs("destiny.net.client", level="ERROR"):
            ticker._real_flush_state([(0, ("SetState", (b"not a snapshot", 99)))])

        self.assertTrue(park.HasBall(1))
        self.assertEqual(ball.x, 12.0)
        self.assertEqual(park.ego, 1)
        self.assertFalse(ticker._state_is_valid)
        self.assertEqual(errors.fatal, 1)

    def test_partial_one_rewind_restores_time_and_removes_future_entities(self):
        park = destiny.Ballpark()
        add_ball(park, 1)
        park.Evolve()
        stamp, checkpoint = park.CaptureFullState()
        add_ball(park, 2, x=2.0)
        park.Evolve()
        self.assertGreater(park.currentTime, stamp)

        park.ReadFullStateFromStream(BytesIO(checkpoint), 1)

        self.assertEqual(park.currentTime, stamp)
        self.assertTrue(park.HasBall(1))
        self.assertFalse(park.HasBall(2))

    def test_pending_removal_round_trips_as_authoritative_lifecycle(self):
        source = destiny.Ballpark()
        add_ball(source, 7)
        source.RemoveBall(7, 2)
        state = BytesIO()
        source.WriteFullStateToStream(state)
        self.assertEqual(snapshot_payload(source)["park"]["pending_removals"][0]["ball_id"], 7)

        restored = destiny.Ballpark()
        restored.ReadFullStateFromStream(state)
        self.assertTrue(restored.HasBall(7))
        restored.Evolve()
        self.assertTrue(restored.HasBall(7))
        restored.Evolve()
        self.assertFalse(restored.HasBall(7))

    def test_partial_two_merges_pending_lifecycle_only_at_the_same_tick(self):
        source = destiny.Ballpark()
        add_ball(source, 8)
        source.RemoveBall(8, 2)
        state = BytesIO()
        source.WriteBallsToStream([8], state)

        target = destiny.Ballpark()
        target.ReadFullStateFromStream(state, 2)
        target.Evolve()
        self.assertTrue(target.HasBall(8))
        target.Evolve()
        self.assertFalse(target.HasBall(8))

        later = destiny.Ballpark()
        add_ball(later, 9)
        later.Evolve()
        future_state = BytesIO()
        later.WriteBallsToStream([9], future_state)
        unsynchronized = destiny.Ballpark()
        with self.assertRaisesRegex(BackendCallError, "current simulation tick"):
            unsynchronized.ReadFullStateFromStream(future_state, 2)
        self.assertFalse(unsynchronized.HasBall(9))

    def test_cloak_sensor_transitions_and_all_spatial_queries_filter_cloak(self):
        park = destiny.Ballpark(True)
        add_ball(park, 1)
        target = add_ball(park, 2, x=2.0)
        target.AddProximitySensor(10.0)
        park.Evolve()
        park.CloakBall(2, destiny.DSTNORMALCLOAK, 20.0)

        self.assertEqual(park.GetBallIdsInCapsule(1, 10.0, 0.0, 0.0, 1.0), [])
        self.assertEqual(park.GetBallIdsInCone(1, 10.0, 0.0, 0.0, 0.5), [])
        self.assertEqual(
            park.GetBallIdsInRangeOfTriangle(1, 10.0, 0.0, 0.0, 0.0, 10.0, 0.0, 1.0),
            [],
        )
        park.CloakBall(2, destiny.DSTGMCLOAK)
        sensors = snapshot_payload(park)["balls"][1]["sensors"]
        self.assertEqual(len(sensors), 1)
        self.assertFalse(any(sensor["cloak_sensor"] for sensor in sensors))
        park.UncloakBall(2)
        self.assertFalse(any(sensor["cloak_sensor"] for sensor in snapshot_payload(park)["balls"][1]["sensors"]))

    def test_exact_cone_and_degenerate_triangle_policies(self):
        park = destiny.Ballpark()
        add_ball(park, 1)
        add_ball(park, 2, x=-0.25, y=0.5, radius=0.1)
        park.Evolve()
        self.assertEqual(park.GetBallIdsInCone(1, 1.0, 0.0, 0.0, 2.0), [])
        with self.assertRaises(ValueError):
            park.GetBallIdsInRangeOfTriangle(1, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0)

    def test_both_get_box_center_entry_points_are_reachable_and_equivalent(self):
        park = destiny.Ballpark()
        expected = destiny.GetBoxCenter((2, 1.0, -1.0, 0.0))
        self.assertEqual(tuple(park.GetBoxCenter(2, 1.0, -1.0, 0.0)), expected)
        with self.assertRaises(ValueError):
            destiny.GetBoxCenter((7, 1.0e100, 0.0, 0.0))
        with self.assertRaises(ValueError):
            park.GetBoxCenter(7, 1.0e100, 0.0, 0.0)

    def test_carbon_batch_is_one_versioned_atomic_outbox_item(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        interface = BevyNetworkInterface(backend, park)
        interface.send_batch(
            [(7, "DoDestinyUpdate", [(1, ("Start", ()))])],
            [([8, 9], "DoDestinyUpdate", [(1, ("Start", ()))])],
        )
        outbox = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        self.assertEqual(outbox["singlecasts"], [])
        self.assertEqual(outbox["narrowcasts"], [])
        self.assertEqual(len(outbox["batches"]), 1)
        envelope = outbox["batches"][0]
        self.assertEqual((envelope["schema_version"], envelope["mode"]), (2, "batch"))
        self.assertGreater(envelope["batch_id"], 0)

        interface._next_batch_id = I64_MAX
        interface.singlecast([])
        with self.assertRaisesRegex(OverflowError, "identifier exhausted"):
            interface.singlecast([])
        exhausted = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        self.assertEqual(exhausted["singlecasts"][0]["batch_id"], I64_MAX)
        backend.invoke(
            "destiny.net.server.NetworkInterface.singlecast",
            "call",
            park._handle,
            (exhausted["singlecasts"][0],),
            {},
        )
        conflicting = dict(exhausted["singlecasts"][0])
        conflicting["updates"] = [[7, "DoDestinyUpdate", []]]
        with self.assertRaisesRegex(BackendCallError, "conflicts"):
            backend.invoke(
                "destiny.net.server.NetworkInterface.singlecast",
                "call",
                park._handle,
                (conflicting,),
                {},
            )
        self.assertEqual(
            backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {}),
            {"singlecasts": [], "narrowcasts": [], "batches": []},
        )

    def test_carbon_codec_counts_memoryview_bytes_and_aggregate_text(self):
        multi_byte_view = memoryview(array("I", [1]))
        self.assertEqual((len(multi_byte_view), multi_byte_view.nbytes), (1, 4))
        with patch.object(carbon_codec, "MAX_CARBON_BINARY_BYTES", 3):
            with self.assertRaisesRegex(ValueError, "binary value exceeds"):
                carbon_codec.encode_carbon_value(multi_byte_view)

        with patch.object(carbon_codec, "MAX_CARBON_TOTAL_STRING_BYTES", 5):
            with self.assertRaisesRegex(ValueError, "total string byte limit"):
                carbon_codec.encode_carbon_value(["abc", "def"])

        with self.assertRaisesRegex(BackendCallError, "snapshot stream exceeds"):
            _read_stream(multi_byte_view, 3)
        with self.assertRaisesRegex(BackendCallError, "invalid snapshot base64"):
            _decode_snapshot_response("%%%")

        class OversizedAtomicSnapshotBackend(InMemoryBackend):
            def invoke(self, canonical_title, operation="call", target=None, args=(), kwargs=None):
                if canonical_title == "dbc.compat.Ballpark.CaptureSnapshot":
                    return {
                        "current_time": 0,
                        "snapshot": "A" * 33,
                    }
                return super().invoke(canonical_title, operation, target, args, kwargs)

        destiny.set_backend(OversizedAtomicSnapshotBackend())
        bounded_park = destiny.Ballpark()
        with patch("destiny._facade.MAX_SNAPSHOT_BYTES", 12):
            with self.assertRaisesRegex(BackendCallError, "configured limit"):
                bounded_park.CaptureFullState()

        class DestructiveNonWriter:
            def __init__(self):
                self.truncated = False

            def Seek(self, _offset, _origin=0):
                pass

            def Truncate(self, _size=0):
                self.truncated = True

        non_writer = DestructiveNonWriter()
        with self.assertRaisesRegex(TypeError, "Write/write"):
            _write_stream(non_writer, b"snapshot")
        self.assertFalse(non_writer.truncated)

        class CarbonStyleStream:
            def __init__(self):
                self.data = bytearray(b"stale-trailing-data")
                self.position = 0

            def Seek(self, offset, _origin=0):
                self.position = offset

            def Truncate(self, size=0):
                del self.data[size:]
                self.position = min(self.position, size)

            def Write(self, payload):
                end = self.position + len(payload)
                self.data[self.position:end] = payload
                self.position = end
                return len(payload)

        carbon_stream = CarbonStyleStream()
        _write_stream(carbon_stream, b"fresh")
        self.assertEqual(bytes(carbon_stream.data), b"fresh")

        class PartialWriter:
            def seek(self, _offset):
                pass

            def truncate(self, _size=0):
                pass

            def write(self, data):
                return len(data) - 1

        park = destiny.Ballpark()
        add_ball(park, 44)
        with self.assertRaisesRegex(OSError, "complete snapshot"):
            park.WriteFullStateToStream(PartialWriter())

    def test_bubble_membership_includes_noninteractive_and_filters_inert_balls(self):
        park = destiny.Ballpark(True)
        add_ball(park, 1)
        park.AddBall(
            2, 5.0, 1.0, 500.0, True, False, True, False, False,
            2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 1.0,
        )
        park.AddBall(
            3, 5.0, 1.0, 500.0, True, True, True, False, False,
            3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.25, 1.0,
        )
        park.Evolve()

        membership = park.GetBubbleMembership()
        self.assertEqual(membership["interactives"], {0: [1]})
        self.assertEqual(membership["members"], {0: [1, 2, 3]})
        self.assertEqual(membership["observers"], {1: [1, 2, 3]})

        park.RemoveBall(2, 2)
        park.CloakBall(3, destiny.DSTGMCLOAK)
        self.assertEqual(park.GetBubbleMembership()["members"], {0: [1]})

    def test_signal_slots_are_weak_and_one_failure_does_not_abort_others(self):
        signal = Signal("v3")
        calls = []

        class Listener:
            def receive(self, value):
                calls.append(value)

        listener = Listener()
        signal.connect(listener.receive)

        def bad(_value):
            raise RuntimeError("subscriber failed")

        def good(value):
            calls.append(value * 2)

        signal.connect(bad)
        signal.connect(good)
        with self.assertLogs(level="ERROR"):
            signal(3)
        self.assertEqual(sorted(calls), [3, 6])
        del listener
        collect()
        self.assertEqual(len(signal), 2)
        with self.assertRaises(TypeError):
            signal.connect(lambda: None)

    def test_background_driver_failure_is_sticky_and_visible(self):
        class FailingBackend(InMemoryBackend):
            def __init__(self):
                super().__init__()
                self.failed = Event()

            def update(self, target=None):
                self.failed.set()
                raise RuntimeError("injected driver failure")

        backend = FailingBackend()
        destiny.set_backend(backend)
        park = destiny.Ballpark()
        park.tickInterval = 1.0
        with self.assertLogs("destiny._facade", level="ERROR"):
            park.Start()
            self.assertTrue(backend.failed.wait(2.0))
            deadline = time.monotonic() + 2.0
            while park.driverError is None and time.monotonic() < deadline:
                time.sleep(0.01)
        self.assertIsNotNone(park.driverError)
        with self.assertRaises(BackendCallError):
            _ = park.currentTime

    def test_blocked_driver_stop_times_out_without_spawning_a_second_driver(self):
        class BlockingBackend(InMemoryBackend):
            def __init__(self):
                super().__init__()
                self.entered = Event()
                self.release_update = Event()

            def update(self, target=None):
                self.entered.set()
                self.release_update.wait(2.0)
                return super().update(target)

        backend = BlockingBackend()
        destiny.set_backend(backend)
        park = destiny.Ballpark()
        park.tickInterval = 1.0
        park.Start()
        self.assertTrue(backend.entered.wait(2.0))
        original_thread = park._driver_thread
        started = time.monotonic()
        with self.assertRaisesRegex(BackendCallError, "did not stop"):
            park._stop_driver(timeout=0.05)
        self.assertLess(time.monotonic() - started, 0.5)
        park._start_driver()
        self.assertIs(park._driver_thread, original_thread)

        backend.release_update.set()
        park._stop_driver(timeout=2.0)
        self.assertIsNone(park._driver_thread)
        park.close()


if __name__ == "__main__":
    unittest.main()
