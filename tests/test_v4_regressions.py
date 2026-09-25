from __future__ import annotations

import base64
from io import BytesIO
import json
import math
from pathlib import Path
import sys
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny import _backend as backend_module  # noqa: E402
from destiny._backend import MAX_COORDINATE  # noqa: E402
from destiny._errors import BackendCallError  # noqa: E402
from destiny.net._codec import decode_carbon_value  # noqa: E402
from destiny.net.client import (  # noqa: E402
    MAX_CATCH_UP_TICKS,
    BaseTicker as ClientBaseTicker,
    ClientTickerInterface,
    TickErrorHandlerInterface,
    Ticker as ClientTicker,
)
from destiny.net.server import (  # noqa: E402
    Actions,
    BevyNetworkInterface,
    BubbleUpdater,
    ParkUpdateBatcher,
)


def add_ball(park, ball_id, *, x=0.0, y=0.0, radius=1.0, is_global=False):
    return park.AddBall(
        ball_id,
        5.0,
        radius,
        500.0,
        True,
        is_global,
        True,
        True,
        False,
        x,
        y,
        0.0,
        0.0,
        0.0,
        0.0,
        0.25,
        1.0,
    )


def snapshot_bytes(park):
    stream = BytesIO()
    park.WriteFullStateToStream(stream)
    return stream.getvalue()


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


class _NoopClientTicker(ClientBaseTicker):
    def do_pre_tick(self):
        pass

    def _flush_state(self, state, wait_for_bubble):
        self.flushed = (state, wait_for_bubble)


class _CharacterInterests:
    def get_client_id_for_character(self, character_id):
        return character_id


class _ClientInterests:
    def get_interested_client_ids_for_ball(self, _ball_id):
        return ()


class _Network:
    def send_batch(self, _singlecasts, _narrowcasts):
        pass


class V4RegressionTests(unittest.TestCase):
    def setUp(self):
        destiny.settings.Reset()
        destiny.use_in_memory_backend()

    def tearDown(self):
        destiny.settings.Reset()
        destiny.clear_backend()

    def test_partial_mode_and_quaternion_component_updates_are_exact_and_atomic(self):
        park = destiny.Ballpark()
        ball = add_ball(park, 1)
        state = snapshot_bytes(park)

        for invalid in (True, 1.0, "1"):
            with self.subTest(invalid=invalid), self.assertRaises((TypeError, ValueError)):
                park.ReadFullStateFromStream(BytesIO(state), invalid)

        before = (ball.rx, ball.ry, ball.rz, ball.rw)
        with self.assertRaises(ValueError):
            ball.rw = 0.0
        self.assertEqual((ball.rx, ball.ry, ball.rz, ball.rw), before)

    def test_sensor_replacement_obeys_child_and_numeric_limits(self):
        park = destiny.Ballpark(True)
        ball = add_ball(park, 1)
        with self.assertRaises(ValueError):
            ball.AddProximitySensor(MAX_COORDINATE * 2.0)

        with patch.object(backend_module, "MAX_CHILD_SHAPES_PER_BALL", 2):
            ball.AddMiniBall(1.0, 0.0, 0.0, 0.25)
            park.CloakBall(1, destiny.DSTNORMALCLOAK, 10.0)
            with self.assertRaises(BackendCallError):
                ball.AddProximitySensor(1.0)

    def test_snapshot_rejects_missing_cloak_state_invalid_ego_and_excessive_depth_atomically(self):
        park = destiny.Ballpark(True)
        ball = add_ball(park, 1, x=7.0)
        park.CloakBall(1, destiny.DSTNORMALCLOAK, 10.0)
        payload = json.loads(snapshot_bytes(park))

        missing_prior = json.loads(json.dumps(payload))
        missing_prior["balls"][0]["massive_before_cloak"] = None
        with self.assertRaises(BackendCallError):
            park.ReadFullStateFromStream(BytesIO(json.dumps(missing_prior).encode()))
        self.assertEqual(ball.x, 7.0)

        invalid_ego = json.loads(json.dumps(payload))
        invalid_ego["park"]["ego"] = 999
        with self.assertRaises(BackendCallError):
            park.ReadFullStateFromStream(BytesIO(json.dumps(invalid_ego).encode()))
        self.assertEqual(ball.x, 7.0)

        deeply_nested = (b"[" * 2_000) + b"0" + (b"]" * 2_000)
        with self.assertRaises(BackendCallError):
            park.ReadFullStateFromStream(BytesIO(deeply_nested))
        self.assertEqual(ball.x, 7.0)

    def test_filtered_snapshots_clear_omitted_ego_and_restore_cleanly(self):
        source = destiny.Ballpark()
        add_ball(source, 1)
        add_ball(source, 2, x=5.0)
        source.ego = 1

        stream = BytesIO()
        source.WriteBallsToStream([2], stream)
        payload = json.loads(stream.getvalue())
        self.assertEqual(payload["park"]["ego"], 0)

        restored = destiny.Ballpark()
        restored.ReadFullStateFromStream(stream)
        self.assertEqual(list(restored.balls), [2])
        self.assertEqual(restored.ego, 0)

    def test_partial_one_prunes_preserved_sensor_members_for_removed_future_balls(self):
        source = destiny.Ballpark()
        add_ball(source, 1)
        source.tickInterval = 100.0
        source.Evolve()
        checkpoint = snapshot_bytes(source)

        target = destiny.Ballpark()
        owner = add_ball(target, 1)
        add_ball(target, 2, x=2.0)
        target.tickInterval = 100.0
        target.Evolve()
        owner.AddProximitySensor(10.0, 0.1)
        target.Evolve()
        target.DrainProximityEvents()

        target.ReadFullStateFromStream(BytesIO(checkpoint), 1)
        target.Evolve()
        self.assertEqual(target.DrainProximityEvents(), [])

    def test_network_batch_ids_are_shared_across_interface_recreation(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        BevyNetworkInterface(backend, park).singlecast([])
        BevyNetworkInterface(backend, park).singlecast([])
        outbox = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        self.assertEqual([row["batch_id"] for row in outbox["singlecasts"]], [1, 2])

    def test_network_retry_reuses_one_idempotency_key_after_lost_reply(self):
        class CommitThenLoseReply(backend_module.InMemoryBackend):
            def __init__(self):
                super().__init__()
                self.lose_reply = True

            def invoke(self, canonical_title, operation="call", target=None, args=(), kwargs=None):
                result = super().invoke(canonical_title, operation, target, args, kwargs)
                if (
                    canonical_title == "destiny.net.server.NetworkInterface.singlecast"
                    and self.lose_reply
                ):
                    self.lose_reply = False
                    raise BackendCallError("simulated lost acknowledgement", code="transport_error")
                return result

        backend = CommitThenLoseReply()
        park = destiny.Ballpark(_backend=backend)
        interface = BevyNetworkInterface(backend, park)
        updates = [(7, "DoDestinyUpdate", [(0, ("Start", ()))])]

        with self.assertRaises(BackendCallError):
            interface.singlecast(updates)
        self.assertEqual(interface._next_batch_id, 1)
        interface.singlecast(updates)
        self.assertEqual(interface._next_batch_id, 2)

        outbox = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        self.assertEqual(len(outbox["singlecasts"]), 1)
        self.assertEqual(outbox["singlecasts"][0]["batch_id"], 1)
        park.close()

    def test_network_retry_fingerprint_ignores_json_object_insertion_order(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        base = {
            "protocol": "destiny-carbon-update",
            "schema_version": 2,
            "mode": "singlecast",
            "batch_id": 1,
            "updates": [[7, "DoDestinyUpdate", [{"alpha": 1, "beta": 2}]]],
        }
        reordered = {
            "protocol": "destiny-carbon-update",
            "schema_version": 2,
            "mode": "singlecast",
            "batch_id": 1,
            "updates": [[7, "DoDestinyUpdate", [{"beta": 2, "alpha": 1}]]],
        }
        for envelope in (base, reordered):
            backend.invoke(
                "destiny.net.server.NetworkInterface.singlecast",
                "call",
                park._handle,
                (envelope,),
                {},
            )
        outbox = backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {})
        self.assertEqual(len(outbox["singlecasts"]), 1)

    def test_direct_network_queue_rejects_aggregate_fanout_before_mutation(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        envelope = {
            "protocol": "destiny-carbon-update",
            "schema_version": 2,
            "mode": "narrowcast",
            "batch_id": 1,
            "updates": [[[7, 8, 9], "DoDestinyUpdate", []]],
        }
        with patch.object(backend_module, "MAX_EXPANDED_NETWORK_ROWS", 2):
            with self.assertRaises(BackendCallError):
                backend.invoke(
                    "destiny.net.server.NetworkInterface.narrowcast",
                    "call",
                    park._handle,
                    (envelope,),
                    {},
                )
        self.assertEqual(
            backend.invoke("dbc.compat.Network.DrainOutbox", "call", park._handle, (), {}),
            {"singlecasts": [], "narrowcasts": [], "batches": []},
        )

    def test_history_access_is_deeply_isolated_and_timestamps_are_exact(self):
        park = destiny.Ballpark()
        batcher = ParkUpdateBatcher(
            park,
            _Network(),
            _CharacterInterests(),
            _ClientInterests(),
            BubbleUpdater(park),
        )
        action = (0, ("Event", ([{"nested": [1]}],)))
        batcher.add_to_character_history(7, action)
        exposed = batcher.get_character_history(7)
        exposed[0][1][1][0][0]["nested"].append(2)
        self.assertEqual(batcher.get_character_history(7)[0][1][1][0][0]["nested"], [1])

        with self.assertRaises((TypeError, ValueError)):
            batcher._check_state_timestamp([("0", ("Event", ()))])

    def test_malformed_client_state_fails_closed_and_future_catchup_is_bounded(self):
        park = destiny.Ballpark()
        add_ball(park, 1)
        errors = _Errors()
        base = _NoopClientTicker(errors)
        base.set_ballpark(park)
        with self.assertLogs("destiny.net.client", level="ERROR"):
            base.update([("0", ("Stop", (1,)))], False)
        self.assertEqual(errors.fatal, 1)

        ticker = ClientTicker(errors, _Client())
        ticker.set_ballpark(park)
        self.assertFalse(ticker.synchronize_to_simulation_time(MAX_CATCH_UP_TICKS + 1))
        self.assertEqual(park.currentTime, 0)

        for malformed, wait_for_bubble in (
            (
                [
                    (0, ("Start", ())),
                    (0, ("SetState", (b"{}", 0))),
                ],
                False,
            ),
            ([(0, ("Start", ()))], 1),
        ):
            before = errors.fatal
            with self.assertLogs("destiny.net.client", level="ERROR"):
                base.update(malformed, wait_for_bubble)
            self.assertEqual(errors.fatal, before + 1)

        before = errors.fatal
        with self.assertLogs("destiny.net.client", level="ERROR"):
            base.update([(-1, ("Start", ()))], False)
        self.assertEqual(errors.fatal, before + 1)

        before = errors.fatal
        with self.assertLogs("destiny.net.client", level="ERROR"):
            base.update((row for row in [(0, ("Start", ()))]), False)
        self.assertEqual(errors.fatal, before + 1)

    def test_snapshot_output_rejects_corrupt_live_sensor_references(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        add_ball(park, 1)
        add_ball(park, 2)
        state = backend._parks[park._handle]
        state.balls[1].sensors = [{
            "range": 10.0,
            "period": 2.0,
            "shuffle": 0,
            "only_interactives": False,
            "elapsed": 0.0,
            "members": [2],
            "cloak_sensor": False,
        }]

        selected = BytesIO()
        park.WriteBallsToStream([1], selected)
        self.assertEqual(json.loads(selected.getvalue())["balls"][0]["sensors"][0]["members"], [])

        state.balls[1].sensors[0]["members"] = [999]
        with self.assertRaises(BackendCallError):
            snapshot_bytes(park)
        with self.assertRaises(TypeError):
            backend.invoke(
                "dbc.compat.Ballpark.Serialize",
                "call",
                park._handle,
                ((ball_id for ball_id in [1]), -1),
                {},
            )

    def test_packaged_action_json_rejects_duplicate_fields(self):
        park = destiny.Ballpark()
        errors = _Errors()
        ticker = _NoopClientTicker(errors)
        ticker.set_ballpark(park)
        packaged = b'[{"$destiny_bevy_tuple_v1":[],"$destiny_bevy_tuple_v1":[]}]'

        with self.assertLogs("destiny.net.client", level="ERROR"):
            ticker.update([(0, ("PackagedAction", (packaged,)))], False)

        self.assertEqual(errors.fatal, 1)

    def test_same_timestamp_set_state_supersedes_queued_actions(self):
        park = destiny.Ballpark()
        add_ball(park, 1)
        replacement = snapshot_bytes(park)
        errors = _Errors()
        ticker = ClientTicker(errors, _Client())
        ticker.set_ballpark(park)

        ticker.update([(0, ("Stop", (1,)))], False)
        ticker.update([(0, ("SetState", (replacement, 1)))], False)
        ticker.do_pre_tick()

        self.assertTrue(ticker._state_is_valid)
        self.assertEqual(ticker._history, [])
        self.assertEqual(errors.fatal, 0)

    def test_binary_tags_require_canonical_base64(self):
        with self.assertRaises(ValueError):
            decode_carbon_value({"$destiny_bevy_bytes_v1": "/x=="})
        self.assertEqual(decode_carbon_value({"$destiny_bevy_bytes_v1": "/w=="}), b"\xff")

    def test_extreme_triangle_and_zero_angle_cone_remain_numerically_defined(self):
        park = destiny.Ballpark()
        add_ball(park, 1)
        add_ball(park, 2, x=5.0e99, y=5.0e99)
        add_ball(park, 3, x=5.0)
        park.Evolve()

        self.assertIn(
            2,
            park.GetBallIdsInRangeOfTriangle(
                1,
                1.0e100,
                0.0,
                0.0,
                0.0,
                1.0e100,
                0.0,
                0.0,
            ),
        )
        self.assertEqual(park.GetBallIdsInCone(1, 10.0, 0.0, 0.0, 0.0), [3])
        self.assertEqual(park.GetBallIdsInCone(1, 10.0, 0.0, 0.0, 5e-324), [3])

    def test_global_balls_block_visibility_across_bubble_ids_and_negative_scan_is_rejected(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        add_ball(park, 1)
        add_ball(park, 2, x=10.0)
        add_ball(park, 3, x=5.0, is_global=True)
        park.Evolve()
        backend._parks[park._handle].balls[3].new_bubble_id = 99

        self.assertEqual(park.CheckVisibility(1, 2), 3)
        with self.assertRaises(ValueError):
            park.ScanCone(1, -0.1, 10.0, 1.0, 0.0, 0.0)

    def test_bubble_cache_detects_same_tick_membership_changes(self):
        backend = destiny.get_backend()
        park = destiny.Ballpark()
        add_ball(park, 1)
        park.Evolve()
        updater = BubbleUpdater(park)
        updater.additions_and_deletions()

        add_ball(park, 2)
        backend._parks[park._handle].balls[2].new_bubble_id = 0
        _, _, additions, _ = updater.additions_and_deletions()
        self.assertEqual(additions[0], [2])

    def test_public_identifiers_and_server_timestamps_are_exact(self):
        with self.assertRaises(TypeError):
            destiny.Ballpark("false")
        park = destiny.Ballpark()
        add_ball(park, 1)
        for invalid in (True, 1.0, "1"):
            with self.subTest(invalid=invalid), self.assertRaises((TypeError, OverflowError)):
                park.GetBall(invalid)
            with self.subTest(invalid=invalid), self.assertRaises((TypeError, OverflowError)):
                park.RemoveBall(invalid)

        batcher = ParkUpdateBatcher(
            park,
            _Network(),
            _CharacterInterests(),
            _ClientInterests(),
            BubbleUpdater(park),
        )
        with self.assertRaises(ValueError):
            batcher._check_state_timestamp([(-1, ("Event", ()))])

        actions = Actions(park)
        with self.assertRaises(TypeError):
            actions.set_ball_massive(1, "false")

    def test_invalid_constructor_handle_releases_backend_ownership(self):
        class MalformedHandleBackend:
            def __init__(self):
                self.released = []

            def invoke(self, canonical_title, operation="call", target=None, args=(), kwargs=None):
                self.assert_construct = (canonical_title, operation, target, args, kwargs)
                return "park:01"

            def update(self, target=None):
                pass

            def release(self, target):
                self.released.append(target)

            def close(self):
                pass

        backend = MalformedHandleBackend()
        with self.assertRaises(BackendCallError):
            destiny.Ballpark(_backend=backend)
        self.assertEqual(backend.released, ["park:01"])

    def test_facade_rejects_malformed_query_and_event_outputs_before_exposure(self):
        class MalformedResultBackend:
            def __init__(self):
                self.results = {
                    "dbc.compat.Ballpark.GetBall": "ball:0:1",
                    "destiny.Ball.id": 2,
                    "destiny.Ball.GetRotatedVector": [1.0, float("nan"), 3.0],
                    "destiny.Ballpark.GetBallIdsInRange": [2, 2],
                    "destiny.Ballpark.GetCurrentEgoPos": [0.0, 1.0],
                    "dbc.compat.Ballpark.CaptureSnapshot": {
                        "current_time": -1,
                        "snapshot": "e30=",
                    },
                    "dbc.compat.Proximity.DrainEvents": [{"kind": "overflow"}],
                }

            def invoke(self, canonical_title, operation="call", target=None, args=(), kwargs=None):
                if canonical_title == "destiny.Ballpark.__init__":
                    return "park:0"
                return self.results.get(canonical_title)

            def update(self, target=None):
                pass

            def release(self, target):
                pass

            def close(self):
                pass

        backend = MalformedResultBackend()
        park = destiny.Ballpark(_backend=backend)
        ball = park.GetBall(1)
        with self.assertRaises(BackendCallError):
            _ = ball.id
        vector = [9.0, 9.0, 9.0]
        with self.assertRaises(BackendCallError):
            ball.GetRotatedVector(vector)
        self.assertEqual(vector, [9.0, 9.0, 9.0])
        with self.assertRaises(BackendCallError):
            park.GetBallIdsInRange(1, 10.0)
        with self.assertRaises(BackendCallError):
            park.GetCurrentEgoPos()
        with self.assertRaises(BackendCallError):
            park.CaptureFullState()
        mismatched = {
            "format": "destiny-bevy-compat-state-v3",
            "schema_version": 3,
            "park": {
                "is_master": False,
                "running": False,
                "tick_interval_ms": 1000.0,
                "friction": 0.0,
                "current_time": 1,
                "time": 0,
                "ego": 0,
                "collision_substeps": 1,
                "use_iterative_collision": False,
                "use_dynamical_orientation": False,
                "disable_dynamical_orientation_for_missiles": False,
                "use_new_orbit": False,
                "pending_removals": [],
                "snapshot_semantics": "logical-authoritative",
            },
            "balls": [],
        }
        backend.results["dbc.compat.Ballpark.CaptureSnapshot"] = {
            "current_time": 0,
            "snapshot": base64.b64encode(
                json.dumps(mismatched, separators=(",", ":")).encode("utf-8")
            ).decode("ascii"),
        }
        with self.assertRaises(BackendCallError):
            park.CaptureFullState()
        with self.assertRaises(BackendCallError):
            park.DrainProximityEvents()
        backend.results["dbc.compat.Ballpark.GetBall"] = "ball:0:2"
        with self.assertRaises(BackendCallError):
            park.GetBall(1)
        park.close()

    def test_duplicate_actions_do_not_emit_false_addition_signals(self):
        park = destiny.Ballpark(True)
        add_ball(park, 1)
        actions = Actions(park)
        observed = []

        def observe(name):
            observed.append(name)

        actions.on_add_to_system_history.connect(observe)
        actions.stop(1)
        actions.stop(1)
        self.assertEqual(len(actions.flush_history()), 1)
        self.assertEqual(observed, ["Stop"])


if __name__ == "__main__":
    unittest.main()
