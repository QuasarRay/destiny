"""Portable server-side Carbon Destiny networking facades."""

from __future__ import annotations

from abc import ABC, abstractmethod
from collections import defaultdict
import copy
import logging
from threading import RLock
from typing import Any
from weakref import WeakKeyDictionary

from destiny.net.const import ClientUpdateCountThisTick
from destiny._errors import UnsupportedTitleError
from destiny._util.signal import Signal
from destiny.net._codec import I64_MAX, decode_carbon_value, encode_carbon_value


logger = logging.getLogger(__name__)
MAX_PENDING_HISTORY_ACTIONS = 100_000
MAX_EXPANDED_UPDATE_ROWS = 200_000


def _exact_i64(value, name):
    if type(value) is not int or not -(2**63) <= value <= I64_MAX:
        raise TypeError(f"{name} must be a signed 64-bit integer")
    return value


def _checked_next_tick(value):
    value = _exact_i64(value, "park timestamp")
    if value == I64_MAX:
        raise OverflowError("park timestamp cannot advance beyond signed 64-bit range")
    return value + 1


def _compat_bool(value, name):
    if type(value) is bool:
        return value
    if type(value) is int:
        return _exact_i64(value, name) != 0
    raise TypeError(f"{name} must be a boolean or integer")


class _BatchSequence:
    def __init__(self):
        self.lock = RLock()
        self.next_id = 1


_batch_sequences_lock = RLock()
_batch_sequences: WeakKeyDictionary[Any, dict[str, _BatchSequence]] = WeakKeyDictionary()


def _batch_sequence(backend, target):
    """Return one monotonic allocator for a backend/runtime pair."""

    try:
        with _batch_sequences_lock:
            targets = _batch_sequences.setdefault(backend, {})
            return targets.setdefault(target, _BatchSequence())
    except TypeError as exc:
        raise TypeError("network backends must be hashable and weak-referenceable") from exc


def _validate_update_rows(rows, *, narrowcast):
    if not isinstance(rows, (list, tuple)):
        raise TypeError("Carbon updates must be a list or tuple")
    if len(rows) > MAX_PENDING_HISTORY_ACTIONS:
        raise ValueError("Carbon update row count exceeds the compatibility limit")
    expanded_rows = 0
    for row in rows:
        if not isinstance(row, (list, tuple)) or len(row) < 3:
            raise ValueError("Carbon update rows must contain recipient, action, and state")
        recipients = row[0] if narrowcast else (row[0],)
        if narrowcast and not isinstance(recipients, (list, tuple)):
            raise TypeError("narrowcast recipients must be a list or tuple")
        if len(recipients) > MAX_PENDING_HISTORY_ACTIONS:
            raise ValueError("Carbon recipient count exceeds the compatibility limit")
        expanded_rows += len(recipients)
        if expanded_rows > MAX_EXPANDED_UPDATE_ROWS:
            raise ValueError("expanded Carbon update count exceeds the compatibility limit")
        for recipient in recipients:
            if type(recipient) is not int or not -(2**63) <= recipient <= I64_MAX:
                raise TypeError("Carbon recipient identifiers must be signed 64-bit integers")
        if not isinstance(row[1], str) or not row[1]:
            raise TypeError("Carbon update action must be a non-empty string")
        if not isinstance(row[2], (list, tuple)):
            raise TypeError("Carbon update state must be a list or tuple")
    return expanded_rows

class NetworkInterface(ABC):
    @abstractmethod
    def narrowcast(self, updates): ...

    @abstractmethod
    def singlecast(self, updates): ...

    def send_batch(self, singlecasts, narrowcasts):
        """Atomically submit one idempotent Carbon tick batch.

        Custom transports must override this method. Falling back to three
        side-effecting calls can split or duplicate a tick after a partial
        failure, so the compatibility layer fails closed instead.
        """
        raise NotImplementedError("the network transport must implement atomic send_batch")


class BevyNetworkInterface(NetworkInterface):
    """Routes versioned Carbon updates to one explicit Bevy runtime."""

    PROTOCOL = "destiny-carbon-update"
    SCHEMA_VERSION = 2

    def __init__(self, backend, ballpark=None):
        self._backend = backend
        if ballpark is not None:
            if ballpark._backend is not backend:
                raise ValueError("ballpark belongs to a different backend")
            if getattr(ballpark, "_closed", False):
                raise ValueError("ballpark is closed")
            self._target = ballpark._handle
        else:
            runtimes = getattr(backend, "_runtimes", None)
            parks = getattr(backend, "_parks", None)
            known = runtimes if runtimes is not None else parks
            if known is None or len(known) != 1:
                raise ValueError("ballpark is required unless the backend owns exactly one runtime")
            self._target = next(iter(known))
        # The allocator belongs to the backend/runtime, not this short-lived
        # adapter. Recreating an interface must never restart IDs at one and be
        # rejected as a replay by the authoritative outbox.
        self._batch_sequence = _batch_sequence(backend, self._target)
        self._batch_lock = self._batch_sequence.lock

    @property
    def _next_batch_id(self):
        return self._batch_sequence.next_id

    @_next_batch_id.setter
    def _next_batch_id(self, value):
        if type(value) is not int or not 1 <= value <= I64_MAX + 1:
            raise ValueError("next Carbon batch identifier is invalid")
        self._batch_sequence.next_id = value

    def _envelope(self, mode, updates, *, batch_id=None):
        if not isinstance(updates, (list, tuple)):
            raise TypeError("updates must be a list or tuple")
        _validate_update_rows(updates, narrowcast=mode == "narrowcast")
        if batch_id is not None and (type(batch_id) is not int or not 1 <= batch_id <= I64_MAX):
            raise ValueError("invalid Carbon batch identifier")
        return {
            "protocol": self.PROTOCOL,
            "schema_version": self.SCHEMA_VERSION,
            "mode": mode,
            "batch_id": self._next_batch_id if batch_id is None else batch_id,
            "updates": encode_carbon_value(list(updates)),
        }

    def _submit(self, canonical_title, mode, updates):
        with self._batch_lock:
            if self._next_batch_id > I64_MAX:
                raise OverflowError("Carbon batch identifier exhausted")
            batch_id = self._next_batch_id
            envelope = self._envelope(mode, updates, batch_id=batch_id)
            result = self._backend.invoke(
                canonical_title,
                "call",
                self._target,
                (envelope,),
                {},
            )
            # Commit the allocator only after the backend acknowledges this
            # exact envelope. On an uncertain failure, a retry reuses the same
            # idempotency key instead of manufacturing a duplicate tick.
            self._next_batch_id = batch_id + 1
            return result

    @classmethod
    def decode_envelope(cls, envelope):
        if not isinstance(envelope, dict):
            raise TypeError("Carbon envelope must be an object")
        if set(envelope) != {"protocol", "schema_version", "mode", "batch_id", "updates"}:
            raise ValueError("Carbon envelope fields do not match schema v2")
        if (
            envelope.get("protocol") != cls.PROTOCOL
            or type(envelope.get("schema_version")) is not int
            or envelope.get("schema_version") != cls.SCHEMA_VERSION
        ):
            raise ValueError("unsupported Carbon envelope protocol")
        if envelope.get("mode") not in {"singlecast", "narrowcast", "batch"}:
            raise ValueError("invalid Carbon envelope mode")
        batch_id = envelope.get("batch_id")
        if type(batch_id) is not int or not 1 <= batch_id <= I64_MAX:
            raise ValueError("invalid Carbon batch identifier")
        updates = envelope.get("updates")
        if envelope.get("mode") == "batch":
            if not isinstance(updates, dict) or set(updates) != {"singlecasts", "narrowcasts"}:
                raise ValueError("Carbon batch updates must contain singlecasts and narrowcasts")
        elif not isinstance(updates, list):
            raise ValueError("Carbon envelope updates must be a list")
        decoded = decode_carbon_value(updates)
        if envelope.get("mode") == "batch":
            expanded = _validate_update_rows(decoded["singlecasts"], narrowcast=False)
            expanded += _validate_update_rows(decoded["narrowcasts"], narrowcast=True)
            if expanded > MAX_EXPANDED_UPDATE_ROWS:
                raise ValueError("expanded Carbon batch exceeds the compatibility limit")
        else:
            _validate_update_rows(decoded, narrowcast=envelope.get("mode") == "narrowcast")
        return decoded

    def narrowcast(self, updates):
        return self._submit(
            "destiny.net.server.NetworkInterface.narrowcast",
            "narrowcast",
            updates,
        )

    def singlecast(self, updates):
        return self._submit(
            "destiny.net.server.NetworkInterface.singlecast",
            "singlecast",
            updates,
        )

    def send_batch(self, singlecasts, narrowcasts):
        if not isinstance(singlecasts, (list, tuple)) or not isinstance(narrowcasts, (list, tuple)):
            raise TypeError("Carbon batch subsets must be lists or tuples")
        expanded = _validate_update_rows(singlecasts, narrowcast=False)
        expanded += _validate_update_rows(narrowcasts, narrowcast=True)
        if expanded > MAX_EXPANDED_UPDATE_ROWS:
            raise ValueError("expanded Carbon batch exceeds the compatibility limit")
        with self._batch_lock:
            if self._next_batch_id > I64_MAX:
                raise OverflowError("Carbon batch identifier exhausted")
            batch_id = self._next_batch_id
            envelope = {
                "protocol": self.PROTOCOL,
                "schema_version": self.SCHEMA_VERSION,
                "mode": "batch",
                "batch_id": batch_id,
                "updates": encode_carbon_value({
                    "singlecasts": list(singlecasts),
                    "narrowcasts": list(narrowcasts),
                }),
            }
            result = self._backend.invoke(
                "destiny.net.server.NetworkInterface.batch",
                "call",
                self._target,
                (envelope,),
                {},
            )
            self._next_batch_id = batch_id + 1
            return result


class ClientInterestsInterface(ABC):
    @abstractmethod
    def add_client_interest(self, ball_id, client_id): ...

    @abstractmethod
    def get_all_interested_ball_ids(self): ...

    @abstractmethod
    def get_interested_client_ids_for_ball(self, ball_id): ...

    @abstractmethod
    def remove_client_interest(self, ball_id, client_id): ...

    @abstractmethod
    def has_client_interest(self, ball_id): ...


class CharacterInterestsInterface(ABC):
    @abstractmethod
    def add_interest(self, ball_id, char_id): ...

    @abstractmethod
    def get_interested_character_ids_for_ball(self, ball_id): ...

    @abstractmethod
    def remove_character_interest(self, ball_id, character_id): ...

    @abstractmethod
    def get_client_id_for_character(self, character_id): ...


class BallInfoInterface(ABC):
    @abstractmethod
    def get_character_for_ball(self, ball_id): ...

    @abstractmethod
    def get_characters_for_ball(self, ball_id): ...


class Actions:
    """Records Carbon action tuples for next-tick application."""

    _DEFERRED_LOCAL_ACTIONS = frozenset({
        "AddBallsToPark",
        "BallNotGlobal",
        "CloakBall",
        "RemoveBall",
        "RemoveGlobalBall",
        "UncloakBall",
    })

    def __init__(self, park):
        self._park = park
        self._system_history = []
        current_time = _exact_i64(park.currentTime, "park timestamp")
        self._stamp_for_system = current_time if current_time == 0 else _checked_next_tick(current_time)
        self.on_ball_made_global = Signal()
        self.on_add_to_system_history = Signal()

    def set_stamp_for_system(self, stamp):
        if type(stamp) is not int or not 0 <= stamp <= I64_MAX:
            raise TypeError("system timestamp must be a non-negative signed 64-bit integer")
        self._stamp_for_system = stamp

    def flush_history(self):
        result, self._system_history = self._system_history, []
        return result

    def _add(self, ball_id, event_name, *args, insert_ball_id_into_args=True, local_args=None):
        ball_id = _exact_i64(ball_id, "ball identifier")
        if event_name not in self._DEFERRED_LOCAL_ACTIONS:
            try:
                getattr(self._park, "_parent_" + event_name)
            except (AttributeError, UnsupportedTitleError) as exc:
                raise UnsupportedTitleError.for_title(f"destiny.Ballpark.{event_name}") from exc
        if insert_ball_id_into_args:
            args = (ball_id, *args)
        event = (event_name, args) if local_args is None else (event_name, args, local_args)
        row = (ball_id, self._stamp_for_system, event)
        appended = not self._system_history or self._system_history[-1] != row
        if appended:
            if len(self._system_history) >= MAX_PENDING_HISTORY_ACTIONS:
                raise OverflowError("system action history exceeds the compatibility limit")
            self._system_history.append(row)
            self.on_add_to_system_history(event_name)

    def _require_ball(self, ball_id):
        if not self._park.HasBall(ball_id):
            raise KeyError(f"ball {ball_id} is not in the park")

    @staticmethod
    def _require_dynamic_orientation():
        from destiny import settings

        if not settings.Get().useDynamicalOrientation:
            raise RuntimeError("Dynamical Orientation is disabled")

    def go_to_point(self, src_id, x, y, z):
        self._add(src_id, "GotoPoint", x, y, z)

    def go_to_direction(self, src_id, x, y, z):
        self._add(src_id, "GotoDirection", x, y, z)

    def orbit(self, src_id, dst_id, orbit_range_meters):
        self._add(src_id, "Orbit", dst_id, orbit_range_meters)

    def set_ball_troll(self, src_id, delay_ticks):
        self._add(src_id, "SetBallTroll", delay_ticks)

    def set_ball_velocity(self, src_id, vx, vy, vz):
        self._add(src_id, "SetBallVelocity", vx, vy, vz)

    def set_ball_angular_velocity(self, src_id, wx, wy, wz):
        self._require_dynamic_orientation()
        self._add(src_id, "SetBallAngularVelocity", wx, wy, wz)

    def set_max_angular_velocity(self, src_id, wx, wy, wz):
        self._require_dynamic_orientation()
        self._add(src_id, "SetMaxAngularVelocity", wx, wy, wz)

    def set_ball_rotation(self, src_id, rx, ry, rz, rw):
        self._require_dynamic_orientation()
        self._add(src_id, "SetBallRotation", rx, ry, rz, rw)

    def set_ball_massive(self, src_id, is_massive):
        self._add(src_id, "SetBallMassive", _compat_bool(is_massive, "is_massive"))

    def stop(self, src_id):
        self._add(src_id, "Stop")

    def set_ball_mass(self, src_id, mass_kg):
        self._add(src_id, "SetBallMass", mass_kg)

    def set_ball_agility(self, src_id, agility):
        self._add(src_id, "SetBallAgility", agility)

    def set_ball_angular_agility(self, src_id, angular_agility):
        self._require_dynamic_orientation()
        self._add(src_id, "SetBallAngularAgility", angular_agility)

    def add_ball_to_client_parks(self, src_id):
        from io import BytesIO

        self._require_ball(src_id)
        stream = BytesIO()
        self._park.WriteBallsToStream([src_id], stream)
        self._add(src_id, "AddBallsToPark", stream.getvalue(), insert_ball_id_into_args=False)

    def make_ball_local(self, src_id):
        self._require_ball(src_id)
        ball = self._park.GetBall(src_id)
        was_global = ball.isGlobal
        try:
            self._park.SetBallGlobal(src_id, False)
            self._add(src_id, "BallNotGlobal", ball.newBubbleId, insert_ball_id_into_args=False)
        except Exception:
            self._park.SetBallGlobal(src_id, was_global)
            raise

    def make_ball_global(self, src_id):
        self._require_ball(src_id)
        was_global = self._park.GetBall(src_id).isGlobal
        before = list(self._system_history)
        try:
            self._park.SetBallGlobal(src_id, True)
            self.add_ball_to_client_parks(src_id)
        except Exception:
            self._system_history = before
            self._park.SetBallGlobal(src_id, was_global)
            raise
        self.on_ball_made_global(src_id)

    def remove_ball(self, src_id):
        self._require_ball(src_id)
        ball = self._park.GetBall(src_id)
        before = list(self._system_history)
        if ball.isGlobal:
            self._add(src_id, "RemoveGlobalBall")
        try:
            self._park.RemoveBall(src_id)
        except Exception:
            self._system_history = before
            raise

    def set_speed_fraction(self, src_id, speed_fraction):
        self._add(src_id, "SetSpeedFraction", speed_fraction)

    def warp_to(self, src_id, x, y, z, minimum_range, warp_speed):
        self._add(src_id, "WarpTo", x, y, z, minimum_range, warp_speed)

    def entity_warp_in(self, src_id, x, y, z, warp_speed):
        self._add(src_id, "EntityWarpIn", x, y, z, warp_speed)

    def set_ball_position(self, src_id, x, y, z, is_local_ball=False):
        self._add(
            src_id,
            "SetBallPosition",
            x,
            y,
            z,
            local_args=_compat_bool(is_local_ball, "is_local_ball"),
        )

    def set_ball_harmonic(self, src_id, harmonic_value, corporation_id, alliance_id, is_forcefield):
        is_forcefield = _compat_bool(is_forcefield, "is_forcefield")
        if is_forcefield:
            self.set_ball_massive(src_id, (harmonic_value, corporation_id, alliance_id) != (-1, -1, -1))
        self._add(src_id, "SetBallHarmonic", harmonic_value, corporation_id, alliance_id, is_forcefield)

    def set_ball_radius(self, src_id, radius_meters):
        self._add(src_id, "SetBallRadius", radius_meters)

    def set_ball_free(self, src_id, is_free=True):
        self._add(src_id, "SetBallFree", _compat_bool(is_free, "is_free"))

    def set_ball_interactive(self, src_id, is_interactive):
        self._add(src_id, "SetBallInteractive", _compat_bool(is_interactive, "is_interactive"))

    def follow_ball(self, src_id, dst_id, range_meters):
        self._add(src_id, "FollowBall", dst_id, range_meters)

    def set_max_speed(self, src_id, max_meters_per_second):
        self._add(src_id, "SetMaxSpeed", max_meters_per_second)

    def set_max_angular_speed(self, src_id, max_radians_per_second):
        self._require_dynamic_orientation()
        self._add(src_id, "SetMaxAngularSpeed", max_radians_per_second)

    def cloak_ball(self, src_id, cloak_mode, uncloak_range_meters):
        if not self._park.HasBall(src_id):
            return False
        ball = self._park.GetBall(src_id)
        bubble_id = ball.oldBubbleId if ball.newBubbleId != ball.oldBubbleId else ball.newBubbleId
        self._add(src_id, "CloakBall", cloak_mode, uncloak_range_meters, local_args=bubble_id)
        return True

    def uncloak_ball(self, src_id):
        self._add(src_id, "UncloakBall")

    def launch_missile(self, src_id, dst_id, owner_id, is_aimed_launch, is_missile_massive):
        self._add(
            src_id,
            "LaunchMissile",
            dst_id,
            owner_id,
            _compat_bool(is_aimed_launch, "is_aimed_launch"),
            _compat_bool(is_missile_massive, "is_missile_massive"),
        )

    def clean_up_followers(self):
        try:
            follower_ids = self._park.GetRemoteFollowers()
        except (AttributeError, UnsupportedTitleError):
            return
        for follower_id in follower_ids:
            self.stop(follower_id)

    def undo_pending_cloak(self, src_id):
        return self._prune_pending_actions(src_id, "CloakBall")

    def _prune_pending_actions(self, src_id, event_name):
        before = len(self._system_history)
        self._system_history = [
            action for action in self._system_history
            if action[0] != src_id or action[2][0] != event_name
        ]
        return before - len(self._system_history)

    def _get_pending_actions(self, src_id, event_name):
        return [event for ball_id, _, event in self._system_history if ball_id == src_id and event[0] == event_name]

    def _has_pending_action(self, src_id, event_name):
        return bool(self._get_pending_actions(src_id, event_name))

    def has_pending_cloak(self, src_id):
        return self._has_pending_action(src_id, "CloakBall")

    def has_pending_uncloak(self, src_id):
        return self._has_pending_action(src_id, "UncloakBall")

    def get_pending_cloak_mode(self, src_id):
        events = self._get_pending_actions(src_id, "CloakBall")
        if events:
            return events[-1][1][1]
        return self._park.GetBall(src_id).isCloaked if self._park.HasBall(src_id) else None

    def has_pending_set_free(self, src_id):
        events = self._get_pending_actions(src_id, "SetBallFree")
        return bool(events and events[-1][1][1])

    def has_pending_set_not_free(self, src_id):
        events = self._get_pending_actions(src_id, "SetBallFree")
        return bool(events and not events[-1][1][1])

    def get_pending_interactive_state(self, src_id):
        events = self._get_pending_actions(src_id, "SetBallInteractive")
        if events:
            return bool(events[-1][1][1])
        return self._park.GetBall(src_id).isInteractive if self._park.HasBall(src_id) else None

    def get_pending_speed_fraction(self, src_id):
        events = self._get_pending_actions(src_id, "SetSpeedFraction")
        if events:
            return events[-1][1][1]
        return self._park.GetBall(src_id).speedFraction if self._park.HasBall(src_id) else None


class BaseTicker(ABC):
    def __init__(self, park, ball_info):
        self._park = park
        self._ball_info = ball_info
        self.stamp_for_system = _checked_next_tick(park.currentTime)
        self.current_system_history = []

    def tick(self):
        self._finalize_tick()
        self._increment_timestamp()
        self._flush_history()
        self._pre_update_bubbles()
        self._update_bubbles()
        self._post_update_bubbles()

    def _increment_timestamp(self):
        self.stamp_for_system = _checked_next_tick(self._park.currentTime)

    @abstractmethod
    def _finalize_tick(self): ...

    @abstractmethod
    def _flush_history(self): ...

    @abstractmethod
    def _pre_update_bubbles(self): ...

    @abstractmethod
    def _update_bubbles(self): ...

    @abstractmethod
    def _post_update_bubbles(self): ...


class BubbleUpdater:
    def __init__(self, park, *unused_interfaces):
        self.park = park
        self._tick = None
        self._membership_fingerprint = None
        self._previous_bubbles: dict[int, set[int]] = {}
        self._previous_observers: dict[int, set[int]] = {}
        self.additions_per_player = defaultdict(list)
        self.deletions_per_player = defaultdict(list)
        self.additions_per_bubble = defaultdict(list)
        self.deletions_per_bubble = defaultdict(list)

    def additions_and_deletions(self):
        membership = self.park.GetBubbleMembership()
        fingerprint = tuple(
            (name, tuple((key, tuple(value)) for key, value in sorted(rows.items())))
            for name, rows in sorted(membership.items())
        )
        if self._tick == self.park.currentTime and fingerprint == self._membership_fingerprint:
            return (
                self.additions_per_player,
                self.deletions_per_player,
                self.additions_per_bubble,
                self.deletions_per_bubble,
            )
        current = {bubble_id: set(ball_ids) for bubble_id, ball_ids in membership["members"].items()}
        observers = {ball_id: set(ball_ids) for ball_id, ball_ids in membership["observers"].items()}
        self.additions_per_player.clear()
        self.deletions_per_player.clear()
        self.additions_per_bubble.clear()
        self.deletions_per_bubble.clear()
        for bubble_id in sorted(set(current) | set(self._previous_bubbles)):
            additions = sorted(current.get(bubble_id, set()) - self._previous_bubbles.get(bubble_id, set()))
            deletions = sorted(self._previous_bubbles.get(bubble_id, set()) - current.get(bubble_id, set()))
            if additions:
                self.additions_per_bubble[bubble_id].extend(additions)
            if deletions:
                self.deletions_per_bubble[bubble_id].extend(deletions)
        for ball_id in sorted(set(observers) | set(self._previous_observers)):
            additions = observers.get(ball_id, set()) - self._previous_observers.get(ball_id, set())
            deletions = self._previous_observers.get(ball_id, set()) - observers.get(ball_id, set())
            if additions:
                self.additions_per_player[ball_id] = sorted(additions)
            if deletions:
                self.deletions_per_player[ball_id] = sorted(deletions)
        self._previous_bubbles = current
        self._previous_observers = observers
        self._tick = self.park.currentTime
        self._membership_fingerprint = fingerprint
        return (
            self.additions_per_player,
            self.deletions_per_player,
            self.additions_per_bubble,
            self.deletions_per_bubble,
        )


class ParkUpdateBatcher:
    def __init__(self, park, network_interface, character_interests, client_interests, bubble_updater):
        self._park = park
        self._network_interface = network_interface
        self._character_interests = character_interests
        self._client_interests = client_interests
        self._bubble_updater = bubble_updater
        self._character_history = defaultdict(list)
        self._bubble_history = defaultdict(list)
        self._pending_history_actions = 0
        self.on_add_to_character_history = Signal("on_add_to_character_history")
        self.on_add_to_bubble_history = Signal("on_add_to_bubble_history")

    def add_to_character_history(self, character_id, action, in_front=False):
        if self._pending_history_actions >= MAX_PENDING_HISTORY_ACTIONS:
            raise OverflowError("pending Carbon history exceeds the compatibility limit")
        try:
            stored = copy.deepcopy(action)
        except RecursionError as exc:
            raise ValueError("Carbon history action exceeds the nesting limit") from exc
        self._check_state_timestamp([stored])
        target = self._character_history[character_id]
        target.insert(0, stored) if in_front else target.append(stored)
        self._pending_history_actions += 1
        self.on_add_to_character_history(stored[1][0])

    def add_to_bubble_history(self, bubble_id, action):
        if self._pending_history_actions >= MAX_PENDING_HISTORY_ACTIONS:
            raise OverflowError("pending Carbon history exceeds the compatibility limit")
        try:
            stored = copy.deepcopy(action)
        except RecursionError as exc:
            raise ValueError("Carbon history action exceeds the nesting limit") from exc
        self._check_state_timestamp([stored])
        self._bubble_history[bubble_id].append(stored)
        self._pending_history_actions += 1
        self.on_add_to_bubble_history(stored[1][0])

    def get_character_history(self, character_id):
        return copy.deepcopy(self._character_history.get(character_id, ()))

    def get_bubble_history(self, bubble_id):
        return copy.deepcopy(self._bubble_history.get(bubble_id, ()))

    def character_has_history(self, character_id):
        return bool(self.get_character_history(character_id))

    def bubble_has_history(self, bubble_id):
        return bool(self.get_bubble_history(bubble_id))

    def clear_character_history(self, character_id):
        removed = self._character_history.pop(character_id, None)
        if removed:
            self._pending_history_actions -= len(removed)

    def get_clients_with_character_history(self):
        result = set()
        for character_id, state in self._character_history.items():
            client_id = self._character_interests.get_client_id_for_character(character_id)
            if client_id is not None and state:
                result.add(client_id)
        return result

    def send_batch(self):
        clients_with_character_history = self.get_clients_with_character_history()
        clients_waiting_for_bubble = set()
        single_batch_narrowcasts = []
        dual_batch_narrowcasts = []
        for bubble_id, state in self._bubble_history.items():
            clients = set()
            for ball_id in self._park.bubbleInteractives.get(bubble_id, ()):
                clients.update(self._client_interests.get_interested_client_ids_for_ball(ball_id))
            if not clients or not state:
                continue
            self._check_state_timestamp(state)
            dual = clients & clients_with_character_history
            single = clients - clients_with_character_history
            if single:
                single_batch_narrowcasts.append(
                    (sorted(single), "DoDestinyUpdate", state, False, ClientUpdateCountThisTick.ONE)
                )
            if dual:
                dual_batch_narrowcasts.append(
                    (sorted(dual), "DoDestinyUpdate", state, False, ClientUpdateCountThisTick.TWO)
                )
            clients_waiting_for_bubble.update(clients)

        singlecasts = []
        for character_id, state in self._character_history.items():
            client_id = self._character_interests.get_client_id_for_character(character_id)
            if client_id is not None and state:
                self._check_state_timestamp(state)
                wait_for_bubble = client_id in clients_waiting_for_bubble
                update_count = ClientUpdateCountThisTick.TWO if wait_for_bubble else ClientUpdateCountThisTick.ONE
                singlecasts.append((client_id, "DoDestinyUpdate", state, wait_for_bubble, update_count))
        narrowcasts = [*single_batch_narrowcasts, *dual_batch_narrowcasts]
        if not singlecasts and not narrowcasts:
            self._character_history.clear()
            self._bubble_history.clear()
            self._pending_history_actions = 0
            return
        # Every timestamp and recipient subset has been validated before this
        # single side effect. The Bevy adapter assigns one idempotency key and
        # queues the complete tick atomically; a failed submit leaves histories
        # intact for a safe retry.
        self._network_interface.send_batch(singlecasts, narrowcasts)
        self._character_history.clear()
        self._bubble_history.clear()
        self._pending_history_actions = 0

    def _check_state_timestamp(self, state):
        for entry in state:
            if not isinstance(entry, (list, tuple)) or len(entry) != 2:
                raise ValueError("state entries must be (timestamp, event) pairs")
            stamp, event = entry
            if type(stamp) is not int or not -(2**63) <= stamp <= I64_MAX:
                raise TypeError("state timestamp must be a signed 64-bit integer")
            if not isinstance(event, (list, tuple)) or len(event) != 2:
                raise ValueError("state events must be (name, args) pairs")
            if not isinstance(event[0], str) or not event[0]:
                raise TypeError("state event name must be a non-empty string")
            if not isinstance(event[1], (list, tuple)):
                raise TypeError("state event arguments must be a list or tuple")
            if stamp < 0 or (stamp != self._park.currentTime and stamp > 0):
                raise ValueError(
                    f"state timestamp {stamp} does not match park time {self._park.currentTime}"
                )

    def send_full_state_update(self, character_id, source_id):
        from io import BytesIO

        stream = BytesIO()
        self._park.WriteFullStateToStream(stream, source_id)
        event = (self._park.currentTime, ("SetState", (stream.getvalue(), source_id)))
        self.add_to_character_history(character_id, event, in_front=True)

    def update_bubbles(self):
        additions_per_player, deletions_per_player, additions_per_bubble, deletions_per_bubble = (
            self._bubble_updater.additions_and_deletions()
        )

        def add_balls_event(ball_ids):
            from io import BytesIO

            stream = BytesIO()
            self._park.WriteBallsToStream(sorted(set(ball_ids)), stream)
            return self._park.currentTime, ("AddBallsToPark", (stream.getvalue(),))

        def remove_balls_event(ball_ids):
            return self._park.currentTime, ("RemoveBalls", (sorted(set(ball_ids)),))

        for bubble_id, ball_ids in additions_per_bubble.items():
            if ball_ids:
                self.add_to_bubble_history(bubble_id, add_balls_event(ball_ids))
        for bubble_id, ball_ids in deletions_per_bubble.items():
            if ball_ids:
                self.add_to_bubble_history(bubble_id, remove_balls_event(ball_ids))

        get_characters = self._character_interests.get_interested_character_ids_for_ball
        for ball_id, additions in additions_per_player.items():
            if additions:
                for character_id in get_characters(ball_id):
                    self.add_to_character_history(character_id, add_balls_event(additions))
        for ball_id, deletions in deletions_per_player.items():
            if deletions:
                for character_id in get_characters(ball_id):
                    self.add_to_character_history(character_id, remove_balls_event(deletions))
        return additions_per_player, deletions_per_player, additions_per_bubble, deletions_per_bubble


class Ticker(BaseTicker):
    def __init__(self, park, ball_info, actions, update_batcher):
        super().__init__(park, ball_info)
        self._actions = actions
        self._update_batcher = update_batcher
        self.on_ball_cloaking = Signal("on_ball_cloaking")
        self.on_ball_uncloaking = Signal("on_ball_uncloaking")
        self.on_set_ball_position = Signal("on_set_ball_position")

    def _finalize_tick(self):
        self._actions.clean_up_followers()

    def _flush_history(self):
        self.current_system_history = self._actions.flush_history()

    def _increment_timestamp(self):
        super()._increment_timestamp()
        self._actions.set_stamp_for_system(self.stamp_for_system)

    def _pre_update_bubbles(self):
        applied = []
        for row in self.current_system_history:
            _, _, event = row
            action, args = event[:2]
            if action in {"CloakBall", "UncloakBall", "RemoveBall", "RemoveGlobalBall", "BallNotGlobal", "AddBallsToPark"}:
                applied.append(row)
                continue
            try:
                parent = getattr(self._park, "_parent_" + action)
            except (AttributeError, UnsupportedTitleError):
                logger.error("Rejected unsupported authoritative action %s", action)
                continue
            try:
                parent(*args)
            except Exception:  # noqa: BLE001 - rejected actions must never enter outgoing history
                logger.exception("Ballpark action %s failed for args %r", action, args)
                continue
            applied.append(row)
        self.current_system_history = applied

    def _update_bubbles(self):
        self._update_batcher.update_bubbles()

    def _post_update_bubbles(self):
        for event_ball_id, stamp, raw_event in self.current_system_history:
            action, args = raw_event[:2]
            local_args = raw_event[2] if len(raw_event) > 2 else None
            event = (action, args)

            if action == "CloakBall":
                if not self._park.HasBall(event_ball_id):
                    continue
                try:
                    self._park.CloakBall(*args)
                except Exception:  # noqa: BLE001 - do not publish a transition that failed locally
                    logger.exception("CloakBall failed for args %r", args)
                    continue
                self.on_ball_cloaking(event_ball_id)
                bubble_id = local_args
                self._update_batcher.add_to_bubble_history(bubble_id, (stamp, event))
                continue
            if action == "UncloakBall":
                if not self._park.HasBall(event_ball_id):
                    continue
                ball = self._park.GetBall(event_ball_id)
                try:
                    self._park.UncloakBall(*args)
                except Exception:  # noqa: BLE001 - do not publish a transition that failed locally
                    logger.exception("UncloakBall failed for args %r", args)
                    continue
                self.on_ball_uncloaking(event_ball_id)
                own_character = self._ball_info.get_character_for_ball(event_ball_id)
                if own_character is not None:
                    self._update_batcher.add_to_character_history(own_character, (stamp, event))
                add_event = self.generate_add_balls_update(event_ball_id)
                for interactive_id in self._park.bubbleInteractives.get(ball.newBubbleId, ()):
                    for character_id in self._ball_info.get_characters_for_ball(interactive_id):
                        if character_id is not None and character_id != own_character:
                            self._update_batcher.add_to_character_history(character_id, (stamp, add_event), in_front=True)
                continue
            if action == "SetBallPosition":
                is_local = local_args is True
                self.on_set_ball_position(event_ball_id, is_local)
                if is_local:
                    continue
            if action == "AddBallsToPark":
                for bubble_id in self._park.bubbleInteractives:
                    self._update_batcher.add_to_bubble_history(bubble_id, (stamp, event))
                continue
            if action in {"BallNotGlobal", "RemoveGlobalBall"}:
                local_bubble = args[0] if action == "BallNotGlobal" and args else None
                for bubble_id in self._park.bubbleInteractives:
                    if bubble_id != local_bubble:
                        self._update_batcher.add_to_bubble_history(
                            bubble_id,
                            (stamp, ("RemoveBall", (event_ball_id,))),
                        )
                continue

            if not self._park.HasBall(event_ball_id):
                continue
            ball = self._park.GetBall(event_ball_id)
            if ball.isCloaked:
                character_id = self._ball_info.get_character_for_ball(event_ball_id)
                if character_id is not None:
                    self._update_batcher.add_to_character_history(character_id, (stamp, event))
            else:
                self._update_batcher.add_to_bubble_history(ball.newBubbleId, (stamp, event))

    def generate_add_balls_update(self, ball_id):
        from io import BytesIO

        stream = BytesIO()
        self._park.WriteBallsToStream([ball_id], stream)
        return "AddBallsToPark", (stream.getvalue(),)


__all__ = [
    "Actions",
    "BallInfoInterface",
    "BaseTicker",
    "BevyNetworkInterface",
    "BubbleUpdater",
    "CharacterInterestsInterface",
    "ClientInterestsInterface",
    "decode_carbon_value",
    "encode_carbon_value",
    "NetworkInterface",
    "ParkUpdateBatcher",
    "Signal",
    "Ticker",
]
