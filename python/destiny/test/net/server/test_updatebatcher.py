# Copyright © 2023 CCP ehf.

import blue
import destiny
from destiny.net.server import ParkUpdateBatcher, BubbleUpdater
from destiny.net.const import ClientUpdateCountThisTick
from destiny.test.net.server.helpers import (
    add_ball_to_park,
    DestinyTestCase,
    TestNetwork,
    TestBallInfo,
    TestClientInterests,
    TestCharacterInterests
)

CHAR_ID_1 = 90000128
BALL_ID_1 = 1001
BALL_ID_2 = 1002
CLIENT_ID_1 = 13579

BUBBLE_1 = 21

GOTO_ACTION = (0, ("GotoPoint", (BALL_ID_1, 1.0, 2.0, 3.0)))
STOP_ACTION = (0, ("Stop", (BALL_ID_1)))


class TestUpdateBatcher(DestinyTestCase):
    def setUp(self):
        self._park = destiny.Ballpark()
        self._park.isMaster = True
        self._ball_info = TestBallInfo()
        self._network = TestNetwork()
        self._character_interests = TestCharacterInterests()
        self._client_interests = TestClientInterests()
        self._bubble_updater = BubbleUpdater(self._park)
        self._update_batcher = ParkUpdateBatcher(
            self._park,
            self._network,
            self._character_interests,
            self._client_interests,
            self._bubble_updater
        )

    def tearDown(self):
        del self._bubble_updater
        del self._update_batcher
        del self._park

    def test_add_to_character_history(self):
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self.assertListEqual(
            [GOTO_ACTION],
            self._update_batcher.get_character_history(CHAR_ID_1)
        )

    def test_add_to_character_history_adds_to_back(self):
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self._update_batcher.add_to_character_history(CHAR_ID_1, STOP_ACTION)
        self.assertListEqual(
            [GOTO_ACTION, STOP_ACTION],
            self._update_batcher.get_character_history(CHAR_ID_1)
        )

    def test_add_to_character_history_can_insert_in_front(self):
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self._update_batcher.add_to_character_history(CHAR_ID_1, STOP_ACTION, in_front=True)
        self.assertListEqual(
            [STOP_ACTION, GOTO_ACTION],
            self._update_batcher.get_character_history(CHAR_ID_1)
        )

    def test_clear_character_history(self):
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self._update_batcher.clear_character_history(CHAR_ID_1)
        self.assertListEqual(
            [],
            self._update_batcher.get_character_history(CHAR_ID_1)
        )

    def test_add_to_bubble_history(self):
        self._update_batcher.add_to_bubble_history(BUBBLE_1, GOTO_ACTION)
        self.assertListEqual(
            [GOTO_ACTION],
            self._update_batcher.get_bubble_history(BUBBLE_1)
        )

    def asssertCastsInBatchSinglecast(self, client_id, expected, cast_list):
        found = False
        for cast in cast_list:
            cast_client_id, name, _, _, casts_in_batch = cast
            if name == "DoDestinyUpdate" and cast_client_id == client_id:
                self.assertEqual(expected, casts_in_batch)
                found = True
        self.assertTrue(found)

    def asssertCastsInBatchNarrowcast(self, client_id, expected, cast_list):
        found = False
        for cast in cast_list:
            cast_client_id_list, name, _, _, casts_in_batch = cast
            if name == "DoDestinyUpdate" and client_id in cast_client_id_list:
                self.assertEqual(expected, casts_in_batch)
                found = True
        self.assertTrue(found)


    def test_character_history_only_has_cast_count_one(self):
        self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self._update_batcher.send_batch()
        self.assertEqual(1, len(self._network.singlecasts))
        self.assertEqual(0, len(self._network.narrowcasts))
        self.asssertCastsInBatchSinglecast(
            CLIENT_ID_1,
            ClientUpdateCountThisTick.ONE,
            self._network.singlecasts
        )

    def test_clients_with_character_history_are_returned_as_a_set(self):
        self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)

        client_ids = self._update_batcher.get_clients_with_character_history()

        self.assertSetEqual({CLIENT_ID_1}, client_ids)

    def test_bubble_history_only_has_cast_count_one(self):
        ball = self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.add_to_bubble_history(ball.newBubbleId, GOTO_ACTION)
        self._update_batcher.send_batch()
        self.assertEqual(0, len(self._network.singlecasts))
        self.assertEqual(1, len(self._network.narrowcasts))
        self.asssertCastsInBatchNarrowcast(
            CLIENT_ID_1,
            ClientUpdateCountThisTick.ONE,
            self._network.narrowcasts
        )

    def test_character_and_bubble_history_has_cast_count_two(self):
        ball = self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self._update_batcher.add_to_bubble_history(ball.newBubbleId, GOTO_ACTION)
        self._update_batcher.send_batch()
        self.asssertCastsInBatchNarrowcast(
            CLIENT_ID_1,
            ClientUpdateCountThisTick.TWO,
            self._network.narrowcasts
        )
        self.asssertCastsInBatchSinglecast(
            CLIENT_ID_1,
            ClientUpdateCountThisTick.TWO,
            self._network.singlecasts
        )

    def _set_up_ball(self, ball_id, char_id, client_id):
        ball = add_ball_to_park(self._park, ball_id=ball_id)
        self._client_interests.add_client_interest(ball_id, client_id)
        self._character_interests.add_interest(ball_id, char_id)
        self._character_interests.map_to_client(char_id, client_id)
        self._ball_info.set_characters_for_ball(ball_id, [char_id])
        return ball

    def _count_matching_set_state_actions(self, actions, ball_id):
        count = 0
        for timestamp, action in actions:
            action_name, action_parameters = action
            if action_name == "SetState":
                ball_data = action_parameters[0]
                park = self.read_balls_from_stream_into_park(ball_data)
                self.assertIn(ball_id, park.balls, "Ball %s not in SetState update stream" % ball_id)
                loaded_ball = park.balls[ball_id]
                self.assertBallsEqual(self._park.balls[ball_id], loaded_ball, ignore_bubble_id=True)
                count += 1
        return count

    def test_send_full_state_update(self):
        self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.send_full_state_update(CHAR_ID_1, BALL_ID_1)
        character_history = self._update_batcher.get_character_history(CHAR_ID_1)
        count = self._count_matching_set_state_actions(character_history, BALL_ID_1)
        self.assertEqual(1, count, "Could not find set state action including ball %s" % BALL_ID_1)

    def assertBallsInAddBallsAction(self, add_balls_action, ball_list):
        action_name, action_parameters = add_balls_action
        self.assertEqual("AddBallsToPark", action_name)
        self.assertEqual(1, len(action_parameters))
        ball_stream, = action_parameters
        park = self.read_balls_from_stream_into_park(ball_stream)
        for ball_id in ball_list:
            self.assertBallsEqual(self._park.balls[ball_id], park.balls[ball_id], ignore_bubble_id=True)

    def assertRemoveBallsActionRemovesBalls(self, remove_balls_action, ball_list):
        action_name, action_parameters = remove_balls_action
        self.assertEqual("RemoveBalls", action_name)
        self.assertEqual(1, len(action_parameters))
        ball_id_list, = action_parameters
        self.assertListEqual(ball_id_list, ball_list)

    def assertBallsInPackagedAddBallsAction(self, packaged_add_balls_action, ball_list):
        action_name, action_parameters = packaged_add_balls_action
        unpackaged_action_list = blue.marshal.Load(action_parameters)
        self.assertEqual(1, len(unpackaged_action_list))
        timestamp, add_balls_action = unpackaged_action_list[0]
        self.assertEqual(self._park.currentTime, timestamp)
        self.assertBallsInAddBallsAction(add_balls_action, ball_list)

    def assertPackagedRemoveBallsActionRemovesBalls(self, packaged_add_balls_action, ball_list):
        action_name, action_parameters = packaged_add_balls_action
        unpackaged_action_list = blue.marshal.Load(action_parameters)
        self.assertEqual(1, len(unpackaged_action_list))
        timestamp, add_balls_action = unpackaged_action_list[0]
        self.assertEqual(self._park.currentTime, timestamp)
        self.assertRemoveBallsActionRemovesBalls(add_balls_action, ball_list)

    def test_update_bubbles_inserts_add_balls_actions_to_character_history(self):
        self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        self._update_batcher.update_bubbles()
        self._park.Evolve()
        self._update_batcher.clear_character_history(CHAR_ID_1)
        add_ball_to_park(self._park, ball_id=BALL_ID_2)
        self._update_batcher.update_bubbles()

        char_history = self._update_batcher.get_character_history(CHAR_ID_1)
        self.assertEqual(1, len(char_history))
        timestamp, action = char_history[0]
        self.assertEqual(self._park.currentTime, timestamp)
        self.assertBallsInPackagedAddBallsAction(action, [BALL_ID_2])

    def test_update_bubbles_inserts_remove_balls_actions_to_character_history(self):
        self._set_up_ball(BALL_ID_1, CHAR_ID_1, CLIENT_ID_1)
        add_ball_to_park(self._park, ball_id=BALL_ID_2)
        self._update_batcher.update_bubbles()
        self._update_batcher.clear_character_history(CHAR_ID_1)
        self._park.RemoveBall(BALL_ID_2)
        self._park.Evolve()
        self._update_batcher.update_bubbles()

        char_history = self._update_batcher.get_character_history(CHAR_ID_1)
        self.assertEqual(1, len(char_history))
        timestamp, action = char_history[0]
        self.assertEqual(self._park.currentTime, timestamp)
        self.assertPackagedRemoveBallsActionRemovesBalls(action, [BALL_ID_2])

    def test_character_has_history(self):
        self.assertFalse(
            self._update_batcher.character_has_history(CHAR_ID_1)
        )
        self._update_batcher.add_to_character_history(CHAR_ID_1, GOTO_ACTION)
        self.assertTrue(
            self._update_batcher.character_has_history(CHAR_ID_1)
        )

    def test_bubble_has_history(self):
        self.assertFalse(
            self._update_batcher.bubble_has_history(BUBBLE_1)
        )
        self._update_batcher.add_to_bubble_history(BUBBLE_1, GOTO_ACTION)
        self.assertTrue(self._update_batcher.bubble_has_history(BUBBLE_1))
