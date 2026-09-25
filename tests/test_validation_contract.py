from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

from destiny import _backend  # noqa: E402
from destiny.net import _codec  # noqa: E402
from destiny.net import server  # noqa: E402


CONTRACT = json.loads((ROOT / "registry" / "validation_contract.json").read_text("utf-8"))


def rust_usize_constant(source, name):
    match = re.search(
        rf"const {re.escape(name)}: usize = ([0-9_ *]+);",
        source,
    )
    if match is None:
        raise AssertionError(f"Rust constant {name} is missing")
    expression = match.group(1).replace("_", "")
    if not re.fullmatch(r"[0-9 *]+", expression):
        raise AssertionError(f"Rust constant {name} is not a literal arithmetic expression")
    result = 1
    for factor in expression.split("*"):
        result *= int(factor.strip())
    return result


class ValidationContractTests(unittest.TestCase):
    def test_python_runtime_limits_match_the_canonical_contract(self):
        snapshot = CONTRACT["snapshot"]
        solver = CONTRACT["solver"]
        abi = CONTRACT["abi"]
        proximity = CONTRACT["proximity"]
        expected = {
            "MAX_REQUEST_BYTES": abi["max_request_bytes"],
            "MAX_RESPONSE_BYTES": abi["max_response_bytes"],
            "MAX_SNAPSHOT_BYTES": snapshot["max_bytes"],
            "MAX_SNAPSHOT_ENCODED_BYTES": 4 * ((snapshot["max_bytes"] + 2) // 3),
            "MAX_SNAPSHOT_BALLS": snapshot["max_balls"],
            "MAX_CHILD_SHAPES_PER_BALL": snapshot["max_child_shapes_per_ball"],
            "MAX_CHILD_DESCRIPTOR_BYTES": snapshot["max_child_descriptor_bytes"],
            "MAX_COORDINATE": solver["max_coordinate"],
            "MAX_RADIUS": solver["max_radius"],
            "MAX_MASS": solver["max_mass"],
            "MAX_VELOCITY": solver["max_velocity"],
            "MAX_AGILITY": solver["max_agility"],
            "MAX_COLLISION_SUBSTEPS": solver["max_collision_substeps"],
            "MAX_PROXIMITY_EVENTS": proximity["max_events"],
            "MAX_PROXIMITY_WORK_PER_TICK": proximity["max_work_per_tick"],
            "MAX_EXPANDED_NETWORK_ROWS": CONTRACT["carbon"]["max_expanded_update_rows"],
        }
        for name, value in expected.items():
            with self.subTest(name=name):
                self.assertEqual(getattr(_backend, name), value)

    def test_python_carbon_limits_match_the_canonical_contract(self):
        carbon = CONTRACT["carbon"]
        expected = {
            "MAX_CARBON_BINARY_BYTES": carbon["max_binary_bytes"],
            "MAX_CARBON_TOTAL_BINARY_BYTES": carbon["max_total_binary_bytes"],
            "MAX_CARBON_STRING_BYTES": carbon["max_string_bytes"],
            "MAX_CARBON_TOTAL_STRING_BYTES": carbon["max_total_string_bytes"],
            "MAX_CARBON_VALUE_DEPTH": carbon["max_value_depth"],
            "MAX_CARBON_VALUE_NODES": carbon["max_value_nodes"],
        }
        for name, value in expected.items():
            with self.subTest(name=name):
                self.assertEqual(getattr(_codec, name), value)
        self.assertEqual(server.MAX_EXPANDED_UPDATE_ROWS, carbon["max_expanded_update_rows"])
        self.assertEqual(server.BevyNetworkInterface.PROTOCOL, carbon["protocol"])
        self.assertEqual(server.BevyNetworkInterface.SCHEMA_VERSION, carbon["schema_version"])

    def test_rust_hard_ceilings_match_the_canonical_contract(self):
        runtime = (ROOT / "src" / "runtime.rs").read_text("utf-8")
        network = (ROOT / "src" / "network.rs").read_text("utf-8")
        carbon_codec = (ROOT / "src" / "carbon_codec.rs").read_text("utf-8")
        snapshot = CONTRACT["snapshot"]
        carbon = CONTRACT["carbon"]
        runtime_expected = {
            "MAX_CONFIGURED_SNAPSHOT_BYTES": snapshot["max_bytes"],
            "MAX_CONFIGURED_SNAPSHOT_BALLS": snapshot["max_balls"],
            "MAX_CONFIGURED_CHILD_SHAPES_PER_BALL": snapshot["max_child_shapes_per_ball"],
            "MAX_CHILD_DESCRIPTOR_BYTES": snapshot["max_child_descriptor_bytes"],
            "MAX_CONFIGURED_OUTBOX_MESSAGES": carbon["max_outbox_messages"],
            "MAX_CONFIGURED_OUTBOX_BYTES": carbon["max_outbox_bytes"],
            "MAX_NETWORK_EXPANDED_ROWS": carbon["max_expanded_update_rows"],
        }
        codec_expected = {
            "MAX_CARBON_BINARY_BYTES": carbon["max_binary_bytes"],
            "MAX_CARBON_TOTAL_BINARY_BYTES": carbon["max_total_binary_bytes"],
            "MAX_CARBON_STRING_BYTES": carbon["max_string_bytes"],
            "MAX_CARBON_TOTAL_STRING_BYTES": carbon["max_total_string_bytes"],
            "MAX_CARBON_VALUE_DEPTH": carbon["max_value_depth"],
            "MAX_CARBON_VALUE_NODES": carbon["max_value_nodes"],
        }
        for name, value in runtime_expected.items():
            with self.subTest(runtime=name):
                self.assertEqual(rust_usize_constant(runtime, name), value)
        for name, value in codec_expected.items():
            with self.subTest(carbon_codec=name):
                self.assertEqual(rust_usize_constant(carbon_codec, name), value)
        self.assertEqual(
            rust_usize_constant(network, "MAX_EXPANDED_UPDATE_ROWS"),
            carbon["max_expanded_update_rows"],
        )

    def test_protocol_and_snapshot_versions_are_deliberately_stable(self):
        self.assertEqual(CONTRACT["schema_version"], 1)
        self.assertEqual(CONTRACT["snapshot"]["partial_modes"], [0, 1, 2])
        self.assertGreaterEqual(len(CONTRACT["rules"]), 10)
        self.assertIn(
            f'#define DBC_ABI_VERSION {CONTRACT["abi"]["version"]}u',
            (ROOT / "include" / "destiny_bevy_compat.h").read_text("utf-8"),
        )


if __name__ == "__main__":
    unittest.main()
