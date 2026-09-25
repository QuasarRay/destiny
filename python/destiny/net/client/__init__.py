"""Portable client ticker API preserving Carbon's Destiny module surface."""

from __future__ import annotations

from abc import ABC, abstractmethod
from collections import defaultdict
import json
import logging
from typing import Any

from destiny.net._codec import decode_carbon_value


logger = logging.getLogger(__name__)
MAX_PACKAGED_ACTION_BYTES = 8 * 1024 * 1024
MAX_PACKAGED_ACTIONS = 100_000
MAX_STATE_HISTORY = 10
MAX_PENDING_HISTORY_ACTIONS = 100_000
MAX_CATCH_UP_TICKS = 10_000
I64_MIN = -(2**63)
I64_MAX = 2**63 - 1


def _strict_object_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON field {key!r}")
        result[key] = value
    return result


def _validate_state_action(action):
    if not isinstance(action, (list, tuple)) or len(action) != 2:
        raise ValueError("every state action must be a (timestamp, event) pair")
    stamp, event = action
    if type(stamp) is not int or not 0 <= stamp <= I64_MAX:
        raise TypeError("state timestamps must be non-negative signed 64-bit integers")
    if not isinstance(event, (list, tuple)) or len(event) != 2:
        raise ValueError("every state event must be a (name, args) pair")
    if not isinstance(event[0], str) or not event[0]:
        raise TypeError("state event names must be non-empty strings")
    if not isinstance(event[1], (list, tuple)):
        raise TypeError("state event arguments must be a list or tuple")
    return action


def _expand_states(state, decoder=None):
    """Expand bounded PackagedAction payloads without requiring ``blue``."""
    if not isinstance(state, (list, tuple)):
        raise TypeError("state must be a list or tuple")
    if len(state) > MAX_PACKAGED_ACTIONS:
        raise ValueError("state exceeds the action-count limit")
    expanded = []
    for action in state:
        _validate_state_action(action)
        event = action[1]
        if event[0] != "PackagedAction":
            expanded.append(action)
            if len(expanded) > MAX_PACKAGED_ACTIONS:
                raise ValueError("state exceeds the action-count limit")
            continue
        payload = event[1]
        if isinstance(payload, (list, tuple)) and len(payload) == 1:
            payload = payload[0]
        if isinstance(payload, (list, tuple)):
            unpackaged = payload
        elif isinstance(payload, (bytes, bytearray, memoryview)):
            raw = bytes(payload)
            if len(raw) > MAX_PACKAGED_ACTION_BYTES:
                raise ValueError("PackagedAction exceeds the byte limit")
            # Never unpickle network input. Product hosts that must consume a
            # legacy Carbon/blue marshal can supply its audited decoder to the
            # ticker; the portable default is bounded UTF-8 JSON.
            unpackaged = (
                decoder(raw)
                if decoder is not None
                else decode_carbon_value(
                    json.loads(raw.decode("utf-8"), object_pairs_hook=_strict_object_pairs)
                )
            )
        else:
            raise TypeError("PackagedAction payload must be actions or encoded bytes")
        if not isinstance(unpackaged, (list, tuple)):
            raise TypeError("PackagedAction must decode to an action list")
        if len(unpackaged) > MAX_PACKAGED_ACTIONS or len(expanded) + len(unpackaged) > MAX_PACKAGED_ACTIONS:
            raise ValueError("PackagedAction exceeds the action-count limit")
        if any(
            isinstance(item, (list, tuple))
            and len(item) == 2
            and isinstance(item[1], (list, tuple))
            and len(item[1]) == 2
            and item[1][0] == "PackagedAction"
            for item in unpackaged
        ):
            raise ValueError("nested PackagedAction payloads are not supported")
        for unpackaged_action in unpackaged:
            _validate_state_action(unpackaged_action)
        expanded.extend(unpackaged)
        if len(expanded) > MAX_PACKAGED_ACTIONS:
            raise ValueError("state exceeds the action-count limit")
    set_state_indexes = [
        index
        for index, action in enumerate(expanded)
        if action[1][0] == "SetState"
    ]
    if set_state_indexes and set_state_indexes != [0]:
        raise ValueError("SetState must occur exactly once and as the first action")
    return expanded


def merge_state_into_history(state, history_list, wait_for_bubble):
    """Merge timestamped state into an ascending, timestamp-grouped history."""
    if not state:
        return
    existing_actions = sum(len(group[0]) for group in history_list)
    if existing_actions + len(state) > MAX_PENDING_HISTORY_ACTIONS:
        raise ValueError("client history exceeds the action-count limit")
    entries_by_time = defaultdict(list)
    for entry in state:
        entries_by_time[entry[0]].append(entry)
    time_list = sorted(entries_by_time)
    time_list_idx = 0
    for history_idx, history_time_entries in enumerate(history_list):
        if time_list_idx >= len(time_list):
            break
        history_time = history_time_entries[0][0][0]
        entry_time = time_list[time_list_idx]
        if history_time < entry_time:
            wait_for_bubble = False
            continue
        if history_time == entry_time:
            history_time_entries[0].extend(entries_by_time[entry_time])
            history_time_entries[1] = False
        else:
            history_list.insert(history_idx, [entries_by_time[entry_time], wait_for_bubble])
            wait_for_bubble = False
        time_list_idx += 1
    for index in range(time_list_idx, len(time_list)):
        history_list.append([entries_by_time[time_list[index]], wait_for_bubble])


class ClientTickerInterface(ABC):
    def set_ballpark(self, ballpark):
        self._ballpark = ballpark

    @abstractmethod
    def on_set_state(self): ...

    @abstractmethod
    def on_ballpark_local_action(self, func_name, args): ...

    @abstractmethod
    def should_log_actions(self): ...

    @abstractmethod
    def get_ball_destruction_delay(self, ball, is_terminal=False): ...

    @abstractmethod
    def get_ball_destruction_delays(self, ball_ids, is_release=False): ...

    @abstractmethod
    def clean_up_after_ball_removal(self, ball_id, ball, is_terminal=False): ...

    @abstractmethod
    def clean_up_after_multiple_ball_removal(self, ball_ids, is_release=False): ...


class TickErrorHandlerInterface(ABC):
    def set_ballpark(self, ballpark):
        self._ballpark = ballpark

    @abstractmethod
    def on_fatal_desync(self): ...

    @abstractmethod
    def on_recoverable_desync(self): ...


class BaseTicker(ABC):
    def __init__(self, error_handler, packaged_action_decoder=None):
        self._ballpark = None
        self._history = []
        self._latest_set_state_time = 0
        self._error_handler = error_handler
        self._packaged_action_decoder = packaged_action_decoder

    def set_ballpark(self, ballpark):
        self._ballpark = ballpark
        self._error_handler.set_ballpark(ballpark)

    def update(self, state, wait_for_bubble):
        if self._ballpark is None:
            return
        try:
            if type(wait_for_bubble) is not bool:
                raise TypeError("wait_for_bubble must be a boolean")
            state = _expand_states(state, self._packaged_action_decoder)
        except Exception:  # noqa: BLE001 - custom decoders are an untrusted input boundary
            logger.exception("rejected malformed Carbon state")
            self._history.clear()
            self._error_handler.on_fatal_desync()
            return
        timestamps = {action[0] for action in state}
        if len(timestamps) > 1:
            self._history.clear()
            self._error_handler.on_fatal_desync()
            return
        try:
            self._flush_state(list(state), wait_for_bubble)
        except (TypeError, ValueError, OverflowError, RecursionError):
            logger.exception("rejected Carbon state history overflow")
            self._history.clear()
            self._error_handler.on_fatal_desync()

    @abstractmethod
    def do_pre_tick(self): ...

    @abstractmethod
    def _flush_state(self, state, wait_for_bubble): ...

    @property
    def _current_time(self):
        return self._ballpark.currentTime


class Ticker(BaseTicker):
    """Carbon-compatible history ticker using the facade's parent-method aliases."""

    LOCAL_ACTIONS = {"AddBallsToPark", "RemoveBall", "RemoveBalls"}

    def __init__(self, error_handler, client_ticker_interface, packaged_action_decoder=None):
        super().__init__(error_handler, packaged_action_decoder)
        self._client_ticker_interface = client_ticker_interface
        self._state_is_valid = False
        self._states = []
        self._last_stamp = -1
        self._should_rebase = False

    def set_ballpark(self, ballpark):
        if ballpark is not None:
            # Carbon tickers own simulation scheduling. An internal wall-clock
            # driver would race snapshot labels, rewind, and replay.
            ballpark.Pause()
        super().set_ballpark(ballpark)
        self._client_ticker_interface.set_ballpark(ballpark)

    def _flush_state(self, state, wait_for_bubble):
        if not state:
            return
        if state[0][1][0] == "SetState":
            self._latest_set_state_time = state[0][0]
            self._history = [
                entry for entry in self._history
                if entry[0] and entry[0][0][0] > self._latest_set_state_time
            ]
            # A replacement at the same timestamp supersedes any previously
            # queued actions for that timestamp. Retaining that group caused
            # merge_state_into_history() to append SetState after an ordinary
            # action, so the ticker never applied the replacement snapshot.
        elif state[0][0] < self._latest_set_state_time:
            return
        merge_state_into_history(state, self._history, wait_for_bubble)

    def do_pre_tick(self):
        while self._history:
            if self._history[0][1]:
                return
            state, _ = self._history.pop(0)
            event_stamp = state[0][0]
            if event_stamp > self._current_time and event_stamp - self._current_time < 3:
                self._history.insert(0, [state, False])
                return
            self._real_flush_state(state)
            if self._state_is_valid and self._should_rebase:
                self.store_state(mid_tick=True)
            if self._history and self._history[0][1]:
                return
            if self._history:
                self._ballpark.Evolve()

    def do_post_tick(self, stamp):
        if type(stamp) is not int or not 0 <= stamp <= I64_MAX:
            raise TypeError("post-tick timestamp must be a non-negative signed 64-bit integer")
        if self._should_rebase:
            self.flush_simulation_history()
            self._should_rebase = False
        elif stamp > self._last_stamp + 10:
            self.store_state()
        self._last_stamp = stamp

    def _set_state(self, state, ego_ball_id):
        from io import BytesIO

        if type(ego_ball_id) is not int or not I64_MIN <= ego_ball_id <= I64_MAX:
            raise TypeError("ego ball identifier must be a signed 64-bit integer")

        rollback = None
        rollback_ego = None
        if self._state_is_valid:
            rollback_stamp, rollback = self._ballpark.CaptureFullState()
            rollback_ego = self._ballpark.ego
        self._state_is_valid = False
        try:
            self._ballpark.ReadFullStateFromStream(BytesIO(state), _restart_driver=False)
            self._ballpark.Pause()
            if ego_ball_id != 0 and not self._ballpark.HasBall(ego_ball_id):
                raise ValueError("SetState ego does not reference a restored ball")
            self._ballpark.ego = ego_ball_id
        except Exception:
            if rollback is not None:
                self._ballpark.ReadFullStateFromStream(BytesIO(rollback), _restart_driver=False)
                self._ballpark.Pause()
                self._ballpark.ego = rollback_ego
                if self._ballpark.currentTime != rollback_stamp:
                    raise RuntimeError("failed to restore the pre-SetState rollback snapshot")
            raise
        self._client_ticker_interface.on_set_state()
        self._state_is_valid = True
        self.flush_simulation_history()

    def _real_flush_state(self, state):
        if not state:
            return
        _, (first_name, first_args) = state[0]
        if first_name == "SetState":
            try:
                self._set_state(*first_args)
            except Exception:  # noqa: BLE001 - replacement failure is a fatal control-flow boundary
                logger.exception("SetState failed")
                self._state_is_valid = False
                self._should_rebase = False
                self._error_handler.on_fatal_desync()
                return
        if not self._state_is_valid:
            return
        self._should_rebase = False
        synchronized = False
        for event_stamp, (func_name, args) in state:
            if func_name == "SetState":
                continue
            try:
                if not synchronized:
                    synchronized = self.synchronize_to_simulation_time(event_stamp)
                if not synchronized:
                    self._error_handler.on_recoverable_desync()
                    return
                self._should_rebase = True
                if func_name in self.LOCAL_ACTIONS:
                    getattr(self, func_name)(*args)
                else:
                    getattr(self._ballpark, "_parent_" + func_name)(*args)
                    self._client_ticker_interface.on_ballpark_local_action(func_name, args)
                    if func_name == "CloakBall" and self._ballpark.ego and self._ballpark.ego != args[0]:
                        self.RemoveBall(args[0])
            except Exception:
                logger.exception("%s failed", func_name)
                # Continuing after an action partially mutates the simulation
                # makes every later state transition untrustworthy. Carbon's
                # caller must replace the state from a fresh SetState instead.
                self._state_is_valid = False
                self._should_rebase = False
                self._error_handler.on_fatal_desync()
                return

    def AddBallsToPark(self, state):
        from io import BytesIO

        self._ballpark.ReadFullStateFromStream(BytesIO(state), 2)

    def RemoveBall(self, ball_id, terminal=False):
        if not self._ballpark.HasBall(ball_id):
            return
        ball = self._ballpark.GetBall(ball_id)
        delay = self._client_ticker_interface.get_ball_destruction_delay(ball, is_terminal=terminal)
        self._ballpark.RemoveBall(ball_id, delay)
        self._client_ticker_interface.clean_up_after_ball_removal(ball_id, ball, terminal)

    def RemoveBalls(self, ball_ids, is_release=False):
        existing = [ball_id for ball_id in ball_ids if self._ballpark.HasBall(ball_id)]
        self._client_ticker_interface.clean_up_after_multiple_ball_removal(existing, is_release)
        delays = self._client_ticker_interface.get_ball_destruction_delays(existing, is_release=is_release)
        for ball_id in existing:
            self._ballpark.RemoveBall(ball_id, delays.get(ball_id, 0))

    def clear_states(self):
        self._history.clear()
        self._states.clear()

    def store_state(self, mid_tick=False):
        stamp, payload = self._ballpark.CaptureFullState()
        self._states.append((stamp, payload, bool(mid_tick)))
        if len(self._states) > MAX_STATE_HISTORY:
            self._states = self._states[:1] + self._states[-(MAX_STATE_HISTORY - 1):]
        self._last_stamp = stamp

    def synchronize_to_simulation_time(self, stamp):
        if type(stamp) is not int or not 0 <= stamp <= I64_MAX:
            raise TypeError("simulation timestamp must be a non-negative signed 64-bit integer")
        if self._ballpark.currentTime > stamp:
            eligible = [state for state in self._states if state[0] <= stamp]
            if not eligible:
                return False
            from io import BytesIO

            _, payload, _ = eligible[-1]
            self._ballpark.ReadFullStateFromStream(BytesIO(payload), 1, _restart_driver=False)
        remaining = stamp - self._ballpark.currentTime
        if remaining > MAX_CATCH_UP_TICKS:
            return False
        while self._ballpark.currentTime < stamp:
            self._ballpark.Evolve()
        return self._ballpark.currentTime == stamp

    def flush_simulation_history(self, new_base_snapshot=True):
        last_mid_state = None
        if new_base_snapshot and self._states:
            candidate = self._states[-1]
            if candidate[2] and candidate[0] == self._current_time - 1:
                last_mid_state = candidate
        self._states.clear()
        if new_base_snapshot:
            if last_mid_state is not None:
                self._states.append(last_mid_state)
            self.store_state()

    def invalidate_state(self):
        self._state_is_valid = False

    def release(self):
        if self._ballpark is not None:
            self.RemoveBalls(list(self._ballpark.balls), is_release=True)
            self._ballpark.ClearAll()
            self.flush_simulation_history(new_base_snapshot=False)
        self._history.clear()
        self._latest_set_state_time = 0
        self._state_is_valid = False
        self.set_ballpark(None)


__all__ = [
    "BaseTicker",
    "ClientTickerInterface",
    "TickErrorHandlerInterface",
    "Ticker",
    "MAX_CATCH_UP_TICKS",
    "_expand_states",
    "merge_state_into_history",
]
