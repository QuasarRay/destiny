# Copyright © 2023 CCP ehf.

import logging
from copy import copy
from collections import defaultdict
import blue

from ..const import ClientUpdateCountThisTick
from destiny._util.signal import Signal
from destiny._util.timing import Timer

logger = logging.getLogger(__name__)


class ParkUpdateBatcher(object):
    """
    Accumulates updates for users and bubbles that
    should get sent out in the ballparks DoPostTick method.
    """
    def __init__(self, ballpark, network_interface, character_interests, client_interests, bubble_updater):
        """
        :type ballpark: destiny.Ballpark
        :type network_interface: destiny.net.server.interface.NetworkInterface
        """
        self._park = ballpark
        self._character_history = defaultdict(list)  # Map of character_id to list of update actions (stamp, event)
        self._bubble_history = {}  # Map of bubble_id to list of updates
        self._network_interface = network_interface
        self._character_interests = character_interests
        self._client_interests = client_interests
        self._bubble_updater = bubble_updater
        self.on_add_to_character_history = Signal(signalName="on_add_to_character_history")
        self.on_add_to_bubble_history = Signal(signalName="on_add_to_bubble_history")

    def get_clients_with_character_history(self):
        """
        Returns the client IDs of all characters
        who have a pending Destiny update to be sent
        to them specifically.

        :rtype: set
        """
        client_ids = set()
        for char_id, state in self._character_history.items():
            client_id = self._character_interests.get_client_id_for_character(char_id)
            if client_id is None:
                continue
            if state:
                client_ids.add(client_id)
        return client_ids

    def send_batch(self):
        """
        Performs narrowcasts and singlecasts.
        Should be called in the ballparks DoPostTick method.
        """
        clients_waiting_for_bubble = set()
        clients_with_character_history = self.get_clients_with_character_history()
        # We send the total number of DoDestinyUpdate batches that
        # each client should expect per tick, because the client
        # needs a way to figure out when it has received all the Destiny
        # updates for that tick, so that it knows when it can start applying
        # gameplay updates from the ballpark, which may depend on the Destiny
        # update already having been applied. (Updates are not guaranteed to
        # arrive in the order in which we send them.)
        # The gameplay updates include a flag which specifies that Destiny updates
        # were sent by the server so that we know when to bother waiting for them.
        single_batch_narrowcasts = []  # For characters who only get DoDestinyUpdate for the bubble
        dual_batch_narrowcasts = []    # For characters who get DoDestinyUpdate for the bubble as well as for their character specifically.
        singlecasts = []

        for bubble_id, state in self._bubble_history.items():
            if bubble_id in self._park.bubbleInteractives:
                client_ids = set()
                for ball_id in self._park.bubbleInteractives[bubble_id]:
                    client_ids.update(self._client_interests.get_interested_client_ids_for_ball(ball_id))
                self._check_state_timestamp(state)
                dual_update_narrowcast_clients = client_ids & clients_with_character_history
                single_update_narrowcast_clients = client_ids - clients_with_character_history
                if single_update_narrowcast_clients:
                    single_batch_narrowcasts.append(
                        (
                            list(single_update_narrowcast_clients),
                            'DoDestinyUpdate',
                            state,
                            False,
                            ClientUpdateCountThisTick.ONE
                        )
                    )
                if dual_update_narrowcast_clients:
                    dual_batch_narrowcasts.append(
                        (
                            list(dual_update_narrowcast_clients),
                            'DoDestinyUpdate',
                            state,
                            False,
                            ClientUpdateCountThisTick.TWO
                        )
                    )
                clients_waiting_for_bubble.update(client_ids)
        self._bubble_history = {}

        for char_id, state in self._character_history.items():
            client_id = self._character_interests.get_client_id_for_character(char_id)
            if client_id is None:
                continue
            if state:
                wait_for_bubble = client_id in clients_waiting_for_bubble
                if wait_for_bubble:
                    update_count = ClientUpdateCountThisTick.TWO
                else:
                    update_count = ClientUpdateCountThisTick.ONE
                self._check_state_timestamp(state)
                singlecasts.append((client_id, 'DoDestinyUpdate', state, wait_for_bubble, update_count))

        self._character_history.clear()
        self._network_interface.singlecast(singlecasts)
        self._network_interface.narrowcast(single_batch_narrowcasts)
        self._network_interface.narrowcast(dual_batch_narrowcasts)

    def _check_state_timestamp(self, state):
        """
        All events which go out should be for the timestamp they are being processed in.
        """
        for entry in state:
            # Ignore the special case first tick.  No-one should be receiving it anyway, so
            # it does not matter that it mixes tick 0 actions into tick 1.
            if entry[0] != self._park.currentTime and entry[0] > 0:
                logger.error(
                    "Found ballpark action for mismatched timestamp {time} {state_id} {entry}".format(
                        time=self._park.currentTime,
                        state_id=id(state),
                        entry=entry
                    )
                )

    def add_to_character_history(self, character_id, action, in_front=False):
        """
        Add an action to the user history to get it to happen on the
        client ballpark for that character.
        :param character_id: The character whose action this is.
        :type character_id: int
        :param action: The action to send to the client.
                       (time_stamp, (ballpark_method_name, (*args,)))
        :type action: tuple
        :param in_front: True if this event should be inserted in front of
                         everything else. This is only done for full state
                         updates.
        :type in_front: bool
        """
        if in_front:
            self._character_history[character_id].insert(0, action)
        else:
            self._character_history[character_id].append(action)
        try:
            action_name = action[1][0]
        except IndexError:
            logger.error("Failed to retrieve action name for \"add to user history\" signal")
        else:
            self.on_add_to_character_history(action_name)

    def clear_character_history(self, char_id):
        """
        Clean up a character's history. This should happen
        when a user logs off.

        :param char_id: The character whose history to clean up.
        :type char_id: int
        """
        if char_id in self._character_history:
            del self._character_history[char_id]

    def add_to_bubble_history(self, bubble_id, action):
        """
        Add an action to the bubbles history, making sure it
        gets sent to all users who are present in the bubble.

        :param bubble_id: The bubble in which to insert the action.
        :type bubble_id: int
        :param action: The action to append to the bubble history.
                       (time_stamp, (ballpark_method_name, (*args,)))
        :type action: tuple
        """
        if bubble_id in self._bubble_history:
            self._bubble_history[bubble_id].append(action)
        else:
            self._bubble_history[bubble_id] = [action]
        try:
            action_name = action[1][0]
        except IndexError:
            logger.error("Failed to retrieve action name for \"add to bubble history\" signal")
        else:
            self.on_add_to_bubble_history(action_name)

    def send_full_state_update(self, char_id, src_id):
        """
        Sends a full state update for character char_id
        for all balls visible from src_id.

        :param char_id: The character to whom we are sending
                        the update
        :type char_id: int
        :param src_id: The ball to use as a reference point
                       when populating the update.
        :type src_id: int
        """
        last = blue.pyos.taskletTimer.EnterTasklet("destiny.net::parkupdatebatcher::issue_set_state")

        try:
            bubble_dump = blue.MemStream()
            self._park.WriteFullStateToStream(bubble_dump, src_id)
            state = bubble_dump.Read()
            set_state_event = self._park.currentTime, ('SetState', (state, src_id))
            logger.info("SetState issued for %s at time %s", src_id, self._park.currentTime)
            self.add_to_character_history(char_id, set_state_event, in_front=True)
        finally:
            blue.pyos.taskletTimer.ReturnFromTasklet(last)

    def _dump_ball_list(self, ball_id_list):
        dump = blue.MemStream()
        self._park.WriteBallsToStream(ball_id_list, dump)
        ball_stream = dump.Read()
        return 'AddBallsToPark', ((ball_stream),)

    def update_bubbles(self):
        """
        Send updates to clients about balls that have changed bubbles and
        have entered/left their bubble.
        """
        # Populate addition/deletion dictionaries
        self._bubble_updater.additions_and_deletions()
        additions_per_player = self._bubble_updater.additions_per_player
        deletions_per_player = self._bubble_updater.deletions_per_player
        additions_per_bubble = self._bubble_updater.additions_per_bubble
        deletions_per_bubble = self._bubble_updater.deletions_per_bubble

        add_actions_per_bubble = {}

        last = blue.pyos.taskletTimer.EnterTasklet("destiny.net::parkupdatebatcher::update_bubbles")
        try:
            # For each bubble, pre-build-and-serialise the addition/deletion updates
            with Timer("destiny.net::parkupdatebatcher::update_bubbles::PreBuildDeletions"):
                # Before: deletions_per_bubble value is a list of ballIDs
                # After : deletions_per_bubble value is an update we can add directly to AddToCharacterHistory
                for bubble_id, dels in deletions_per_bubble.items():
                    if len(dels) > 0:
                        user_actions = (
                            (self._park.currentTime, ('RemoveBalls', (dels,))),
                        )
                        packaged_action = blue.marshal.Save(user_actions)
                        deletions_per_bubble[bubble_id] = (
                            self._park.currentTime, ('PackagedAction', packaged_action)
                        )

            with Timer("destiny.net::parkupdatebatcher::update_bubbles::PreBuildAdditions"):
                # Before: additions_per_bubble value is a list of ballIDs
                # After : additions_per_bubble value is an update we can add directly to AddToCharacterHistory
                for bubble_id, adds in additions_per_bubble.items():
                    if len(adds) > 0:
                        # Collect up all the user actions for this bubble
                        user_actions = [
                            (self._park.currentTime, self._dump_ball_list(adds)),
                        ]

                        with Timer("destiny.net::parkupdatebatcher::update_bubbles::PreBuildAdditions::Marshal"):
                            # Now we have all the actions for this bubble, package them up into a single action
                            packaged_action = blue.marshal.Save(user_actions)
                            add_actions_per_bubble[bubble_id] = (
                                self._park.currentTime, ('PackagedAction', packaged_action)
                            )

            # Now 'deletions' and 'additions' contain the actions for each user
            # Let's build that into the history
            getter = self._character_interests.get_interested_character_ids_for_ball

            with Timer("destiny.net::parkupdatebatcher::update_bubbles::Deletions"):
                for ball_id, dels in deletions_per_player.items():
                    # If dels is an int, it is the ID of a bubble
                    # We can use this ID to look up a pre-serialised update for the user
                    if isinstance(dels, int):
                        deletions = deletions_per_bubble[dels]
                        if deletions:
                            for character_id in getter(ball_id):
                                self.add_to_character_history(character_id, deletions)
                    elif len(dels) > 0:
                        for character_id in getter(ball_id):
                            self.add_to_character_history(
                                character_id, (self._park.currentTime, ('RemoveBalls', (dels,)))
                            )

            with Timer("destiny.net::parkupdatebatcher::update_bubbles::Additions"):
                for ball_id, adds in additions_per_player.items():

                    # If adds is an int, it is the ID of a bubble
                    # We can use this ID to look up a pre-serialised update for the user
                    if isinstance(adds, int):
                        additions = add_actions_per_bubble.get(adds, [])
                        if additions:
                            for character_id in getter(ball_id):
                                self.add_to_character_history(character_id, additions)
                    elif len(adds) > 0:
                        characters = getter(ball_id)
                        if characters:
                            # Collect up all the user actions, and add them to the history in a single packaged action
                            user_actions = [
                                (self._park.currentTime, self._dump_ball_list(adds)),
                            ]
                            with Timer("destiny.net::parkupdatebatcher::update_bubbles::AddToCharacterHistory"):
                                # Now we have all the actions for this user, package them up into a single action
                                packaged_action = blue.marshal.Save(user_actions)
                                for character_id in characters:
                                    self.add_to_character_history(
                                        character_id, (self._park.currentTime, ('PackagedAction', packaged_action))
                                    )
        finally:
            blue.pyos.taskletTimer.ReturnFromTasklet(last)

    def get_bubble_history(self, bubble_id):
        """
        Get a copy of the history for the bubble.

        :param bubble_id: The bubble whose history we want to get.
        :type bubble_id: int
        """
        return copy(self._bubble_history.get(bubble_id, []))

    def bubble_has_history(self, bubble_id):
        """
        Does the bubble have a history?
        :param bubble_id: The bubble whose history we want to check for.
        :type bubble_id: int
        :rtype: bool
        """
        return bool(
            bubble_id in self._bubble_history and
            self._bubble_history[bubble_id]
        )

    def get_character_history(self, char_id):
        """
        Get a copy of the history for the character.

        :param char_id: The character whose history we want to get.
        :type char_id: int
        """
        return copy(self._character_history.get(char_id, []))

    def character_has_history(self, char_id):
        """
        Does the character have a history?

        :param char_id: The character whose history we want to check for.
        :type char_id: int
        :rtype: bool
        """
        return bool(
            char_id in self._character_history and
            self._character_history[char_id]
        )
