from __future__ import annotations

from io import BytesIO
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
import _destiny  # noqa: E402
from destiny.net.server import BevyNetworkInterface  # noqa: E402


def add(park, ball_id, position, velocity=(0.0, 0.0, 0.0), radius=1.0):
    return park.AddBall(
        ball_id,
        5.0,
        radius,
        500.0,
        True,
        False,
        True,
        True,
        False,
        *position,
        *velocity,
        0.25,
        1.0,
    )


class ExtendedPythonContractTests(unittest.TestCase):
    def setUp(self) -> None:
        destiny.use_in_memory_backend()

    def tearDown(self) -> None:
        destiny.clear_backend()

    def test_direct_extension_import_shim_exports_same_types(self) -> None:
        self.assertIs(_destiny.Ballpark, destiny.Ballpark)
        self.assertIs(_destiny.Ball, destiny.Ball)

    def test_snapshot_round_trip_through_file_like_stream(self) -> None:
        source = destiny.Ballpark(True)
        ball = add(source, 42, (1.0, 2.0, 3.0), (4.0, 5.0, 6.0), 2.5)
        ball.isCloaked = 1
        ball.AddMiniBall(1.0, 0.0, 0.0, 0.5)
        ball.AddMiniCapsule(0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.25)
        ball.AddProximitySensor(20.0, 1.5, 3, True)
        stream = BytesIO()
        source.WriteFullStateToStream(stream)

        destination = destiny.Ballpark(False)
        destination.ReadFullStateFromStream(stream)
        restored = destination.GetBall(42)
        self.assertEqual((restored.x, restored.y, restored.z), (1.0, 2.0, 3.0))
        self.assertEqual((restored.vx, restored.vy, restored.vz), (4.0, 5.0, 6.0))
        self.assertEqual(restored.radius, 2.5)
        self.assertEqual(restored.isCloaked, 1)
        restored_stream = BytesIO()
        destination.WriteFullStateToStream(restored_stream)
        restored_payload = json.loads(restored_stream.getvalue())
        restored_state = restored_payload["balls"][0]
        self.assertEqual(restored_state["minis"][1]["a"], [0.0, 0.0, 0.0])
        self.assertEqual(restored_state["minis"][1]["b"], [0.0, 2.0, 0.0])
        self.assertEqual(restored_state["sensors"][0]["only_interactives"], True)

    def test_multiple_ballparks_are_isolated_on_one_backend(self) -> None:
        first = destiny.Ballpark(True)
        second = destiny.Ballpark(False)
        first_ball = add(first, 7, (1.0, 0.0, 0.0))
        second_ball = add(second, 7, (99.0, 0.0, 0.0))
        self.assertEqual(first_ball.x, 1.0)
        self.assertEqual(second_ball.x, 99.0)
        first.SetBallPosition(7, 5.0, 0.0, 0.0)
        self.assertEqual(first_ball.x, 5.0)
        self.assertEqual(second_ball.x, 99.0)

    def test_fixed_evolution_and_spatial_queries(self) -> None:
        park = destiny.Ballpark()
        moving = add(park, 1, (0.0, 0.0, 0.0), (10.0, 0.0, 0.0))
        add(park, 2, (6.0, 0.0, 0.0), radius=1.0)
        add(park, 3, (50.0, 0.0, 0.0), radius=1.0)
        park.tickInterval = 100
        park.friction = 0.0
        park.Evolve()
        self.assertAlmostEqual(moving.x, 1.0)
        self.assertEqual(park.currentTime, 1)
        self.assertEqual(park.GetBallIdsInRange(1, 6.0), [2])
        self.assertEqual(park.GetBallIdsAndDistInRange(1, 6.0), [(25.0, 2)])
        self.assertEqual(park.GetBallIdsInCapsule(1, 10.0, 0.0, 0.0, 1.0), [2])
        self.assertEqual(
            park.GetBallIdsInRangeOfTriangle(1, 10.0, 0.0, 0.0, 0.0, 10.0, 0.0, 0.1),
            [2],
        )

    def test_compound_collider_calls_and_impulse_are_callable(self) -> None:
        park = destiny.Ballpark()
        ball = add(park, 1, (0.0, 0.0, 0.0), radius=2.0)
        ball.AddMiniBall(1.0, 0.0, 0.0, 0.5)
        ball.AddMiniCapsule(0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.25)
        ball.AddMiniBox(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
        ball.AddProximitySensor(20.0)
        ball.ApplyImpulsiveForceAtPosition((5.0, 0.0, 0.0), (0.0, 1.0, 0.0))
        # This legacy client-only visual impulse must not change the
        # authoritative linear or angular trajectory.
        self.assertEqual((ball.vx, ball.vy, ball.vz), (0.0, 0.0, 0.0))
        self.assertEqual((ball.wx, ball.wy, ball.wz), (0.0, 0.0, 0.0))

    def test_network_bridge_preserves_carbon_update_tuples(self) -> None:
        class RecordingBackend:
            def __init__(self):
                self.calls = []
                self._parks = {"park:9": object()}

            def invoke(self, title, operation="call", target=None, args=(), kwargs=None):
                self.calls.append((title, operation, target, args, kwargs))

            def close(self):
                pass

            def update(self, target=None):
                pass

            def release(self, target):
                pass

        backend = RecordingBackend()
        network = BevyNetworkInterface(backend)
        single = [(7, "DoDestinyUpdate", [(1, ("Start", ()))])]
        narrow = [([7, 8], "DoDestinyUpdate", [(1, ("Start", ()))])]
        network.singlecast(single)
        network.narrowcast(narrow)
        self.assertEqual(backend.calls[0][0], "destiny.net.server.NetworkInterface.singlecast")
        self.assertEqual(backend.calls[0][2], "park:9")
        envelope = backend.calls[0][3][0]
        self.assertEqual(envelope["protocol"], "destiny-carbon-update")
        self.assertEqual(envelope["schema_version"], 2)
        self.assertGreater(envelope["batch_id"], 0)
        self.assertEqual(BevyNetworkInterface.decode_envelope(envelope), single)
        self.assertEqual(backend.calls[1][0], "destiny.net.server.NetworkInterface.narrowcast")


class SourceContractTests(unittest.TestCase):
    def test_c_header_is_c11_compatible(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "smoke.c"
            source.write_text(
                '#include "destiny_bevy_compat.h"\n'
                "int main(void) { DbcBuffer b = {0}; return b.len == 0 ? 0 : 1; }\n",
                encoding="utf-8",
            )
            subprocess.run(
                [
                    "cc",
                    "-std=c11",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pedantic",
                    "-fsyntax-only",
                    "-I",
                    str(ROOT / "include"),
                    str(source),
                ],
                check=True,
            )

    def test_rust_dependencies_are_pinned_to_audited_versions(self) -> None:
        cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('avian3d = { version = "=0.7.0"', cargo)
        self.assertIn('bevy = { version = "=0.19.1"', cargo)
        self.assertIn('bevy_replicon = { version = "=0.42.3"', cargo)
        self.assertIn('lightyear_messages = { version = "=0.29.0"', cargo)
        self.assertNotIn("lightyear_prediction =", cargo)
        self.assertIn('lightyear_replication = { version = "=0.29.0"', cargo)
        self.assertIn('features = ["std", "client", "server"]', cargo)

    def test_native_source_contains_live_integration_paths_and_panic_barriers(self) -> None:
        runtime = (ROOT / "src" / "runtime.rs").read_text(encoding="utf-8")
        network = (ROOT / "src" / "network.rs").read_text(encoding="utf-8")
        ffi = (ROOT / "src" / "ffi.rs").read_text(encoding="utf-8")
        for token in (
            "RigidBody::Dynamic",
            "Collider::sphere",
            "Collider::capsule_endpoints",
            "Collider::cuboid",
            "LinearVelocity",
            "AngularVelocity",
            "MaxLinearSpeed",
            "MaxAngularSpeed",
            "DestinySpaceFriction",
            "LinearDamping",
            "DestinyBubbleCollisionHooks",
            "with_collision_hooks",
            "Time<Physics>",
            "TimeUpdateStrategy::FixedTimesteps(1)",
            "SubstepCount(if options.use_iterative_collision",
            "DestinyCarbonReplicationBundle",
            "DestinyPendingRemoval",
            "restore_child_shapes",
            "new_with_app",
        ):
            self.assertIn(token, runtime)
        for token in (
            "DestinyCarbonInteropPlugin",
            "DestinyCarbonReplicationBundle",
            "Replicated",
            "Replicate",
            "Rooms",
            "MessageSender",
            "MessageReceiver",
            "CarbonClientIdentity",
            "sync_client_visibility_rooms",
            "sync_ball_visibility_rooms",
            "flush_carbon_outbox",
            "receive_carbon_frames",
            "app.replicate::<DestinyBallId>()",
            "app.replicate::<Position>()",
        ):
            self.assertIn(token, network)
        for dead_token in ("ServerMultiMessageSender", "DestinyChildCollider"):
            self.assertNotIn(dead_token, network)
        self.assertIn("catch_unwind", ffi)
        self.assertIn("LimitedWriter", ffi)
        self.assertIn("Box::from_raw", ffi)


if __name__ == "__main__":
    unittest.main()
