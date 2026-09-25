from __future__ import annotations

import copy
from io import BytesIO
import json
from pathlib import Path
import random
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._errors import BackendCallError  # noqa: E402
from destiny.net._codec import decode_carbon_value, encode_carbon_value  # noqa: E402


SEED = 0xD357_1A4


def add_ball(park, ball_id, rng):
    return park.AddBall(
        ball_id,
        rng.uniform(0.1, 1.0e4),
        rng.uniform(0.1, 25.0),
        rng.uniform(1.0, 500.0),
        True,
        ball_id % 7 == 0,
        True,
        ball_id % 3 != 0,
        False,
        rng.uniform(-1.0e6, 1.0e6),
        rng.uniform(-1.0e6, 1.0e6),
        rng.uniform(-1.0e6, 1.0e6),
        rng.uniform(-100.0, 100.0),
        rng.uniform(-100.0, 100.0),
        rng.uniform(-100.0, 100.0),
        rng.uniform(0.1, 10.0),
        rng.random(),
    )


def snapshot(park):
    stream = BytesIO()
    park.WriteFullStateToStream(stream)
    return stream.getvalue()


def random_carbon_value(rng, depth=0):
    leaves = [
        None,
        False,
        True,
        -(2**63),
        2**63 - 1,
        rng.randint(-1_000_000, 1_000_000),
        rng.uniform(-1.0e8, 1.0e8),
        "",
        "Carbon-Δ-🙂",
        bytes(rng.getrandbits(8) for _ in range(rng.randrange(0, 24))),
    ]
    if depth >= 4 or rng.random() < 0.45:
        return rng.choice(leaves)
    kind = rng.randrange(3)
    if kind == 0:
        return [random_carbon_value(rng, depth + 1) for _ in range(rng.randrange(5))]
    if kind == 1:
        return tuple(random_carbon_value(rng, depth + 1) for _ in range(rng.randrange(5)))
    return {
        f"key-{depth}-{index}": random_carbon_value(rng, depth + 1)
        for index in range(rng.randrange(5))
    }


class V4PropertyTests(unittest.TestCase):
    def setUp(self):
        destiny.settings.Reset()
        destiny.use_in_memory_backend()

    def tearDown(self):
        destiny.settings.Reset()
        destiny.clear_backend()

    def test_canonical_codec_round_trips_seeded_recursive_values(self):
        rng = random.Random(SEED)
        for case in range(500):
            with self.subTest(case=case):
                value = random_carbon_value(rng)
                encoded = encode_carbon_value(value)
                wire_copy = json.loads(
                    json.dumps(encoded, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
                )
                self.assertEqual(decode_carbon_value(wire_copy), value)
                self.assertEqual(encode_carbon_value(decode_carbon_value(encoded)), encoded)

    def test_seeded_snapshot_round_trip_is_canonical(self):
        rng = random.Random(SEED)
        source = destiny.Ballpark(True)
        source.tickInterval = 20.0
        source.friction = 0.75
        for ball_id in rng.sample(range(1, 10_000), 64):
            add_ball(source, ball_id, rng)
        source.ego = next(iter(source.balls))
        for _ in range(5):
            source.Evolve()

        payload = snapshot(source)
        restored = destiny.Ballpark()
        restored.ReadFullStateFromStream(BytesIO(payload))

        self.assertEqual(snapshot(restored), payload)
        self.assertEqual(restored.currentTime, source.currentTime)
        self.assertEqual(list(restored.balls), list(source.balls))

    def test_invalid_snapshot_mutations_are_atomic(self):
        rng = random.Random(SEED)
        park = destiny.Ballpark(True)
        add_ball(park, 1, rng)
        add_ball(park, 2, rng)
        park.ego = 1
        baseline = snapshot(park)
        parsed = json.loads(baseline)

        mutations = []
        invalid_radius = copy.deepcopy(parsed)
        invalid_radius["balls"][0]["radius"] = "large"
        mutations.append(invalid_radius)
        duplicate_id = copy.deepcopy(parsed)
        duplicate_id["balls"][1]["id"] = duplicate_id["balls"][0]["id"]
        mutations.append(duplicate_id)
        dangling_ego = copy.deepcopy(parsed)
        dangling_ego["park"]["ego"] = 99_999
        mutations.append(dangling_ego)
        unknown_field = copy.deepcopy(parsed)
        unknown_field["park"]["unexpected"] = True
        mutations.append(unknown_field)
        negative_tick = copy.deepcopy(parsed)
        negative_tick["park"]["current_time"] = -1
        mutations.append(negative_tick)
        dangling_sensor = copy.deepcopy(parsed)
        dangling_sensor["balls"][0]["sensors"] = [{
            "range": 10.0,
            "period": 2.0,
            "shuffle": 0,
            "only_interactives": False,
            "elapsed": 0.0,
            "members": [99_999],
            "cloak_sensor": False,
        }]
        mutations.append(dangling_sensor)

        for case, mutation in enumerate(mutations):
            with self.subTest(case=case), self.assertRaises(BackendCallError):
                park.ReadFullStateFromStream(
                    BytesIO(json.dumps(mutation, separators=(",", ":")).encode("utf-8"))
                )
            self.assertEqual(snapshot(park), baseline)

        for case in range(128):
            malformed = b"not-a-snapshot:" + bytes(
                rng.getrandbits(8) for _ in range(rng.randrange(0, 96))
            )
            with self.subTest(malformed=case), self.assertRaises(BackendCallError):
                park.ReadFullStateFromStream(BytesIO(malformed))
        self.assertEqual(snapshot(park), baseline)

    def test_seeded_geometry_queries_are_sorted_and_stable(self):
        rng = random.Random(SEED)
        park = destiny.Ballpark()
        add_ball(park, 1, rng)
        for ball_id in range(2, 82):
            add_ball(park, ball_id, rng)
        park.Evolve()

        for case in range(100):
            vector = [rng.uniform(-1.0e6, 1.0e6) for _ in range(3)]
            if vector == [0.0, 0.0, 0.0]:
                vector[0] = 1.0
            rows = park.GetBallIdsInCone(1, *vector, rng.uniform(0.0, 3.141592653589793))
            with self.subTest(case=case):
                self.assertEqual(rows, sorted(set(rows)))
                self.assertNotIn(1, rows)


if __name__ == "__main__":
    unittest.main()
