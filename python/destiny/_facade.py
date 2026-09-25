"""Destiny-named facade over the backend dispatch protocol."""

from __future__ import annotations

import base64
from collections.abc import Iterator, Mapping
import io
import json
import logging
import math
from threading import Event, RLock, Thread, current_thread
from typing import Any

from ._backend import (
    MAX_PROXIMITY_EVENTS,
    MAX_SNAPSHOT_BALLS,
    MAX_SNAPSHOT_BYTES,
    Backend,
    InMemoryBackend,
    NativeBackend,
)
from ._errors import BackendCallError, UnsupportedTitleError
from ._registry import RECORDS


logger = logging.getLogger(__name__)


_default_backend: Backend | None = None
_backend_state_lock = RLock()
_backend_users: dict[int, tuple[Backend, int]] = {}
_retired_backends: set[int] = set()

_SUPPORTED_DYNAMIC_PARK_METHODS = frozenset({
    "AdjustTimes",
    "CheckVisibility",
    "CloakBall",
    "GetBallIdsAndDistInRange",
    "GetBallIdsInCapsule",
    "GetBallIdsInCone",
    "GetBallIdsInRange",
    "GetBallIdsInRangeOfTriangle",
    "GetBoxCenter",
    "GetCurrentEgoPos",
    "ScanCone",
    "SetBallAgility",
    "SetBallFree",
    "SetBallGlobal",
    "SetBallInteractive",
    "SetBallMassive",
    "SetBallRadius",
    "SetSpeedFraction",
    "Stop",
    "UncloakBall",
})


def _exact_i64(value: Any, name: str, *, allow_canonical_string: bool = False) -> int:
    if allow_canonical_string and isinstance(value, str):
        try:
            parsed = int(value)
        except ValueError as exc:
            raise BackendCallError(f"backend returned invalid {name}", code="invalid_backend_response") from exc
        if str(parsed) != value:
            raise BackendCallError(f"backend returned non-canonical {name}", code="invalid_backend_response")
        value = parsed
    if type(value) is not int or not -(2**63) <= value <= 2**63 - 1:
        raise BackendCallError(f"backend returned invalid {name}", code="invalid_backend_response")
    return value


def _input_i64(value: Any, name: str) -> int:
    if type(value) is not int:
        raise TypeError(f"{name} must be an integer")
    if not -(2**63) <= value <= 2**63 - 1:
        raise OverflowError(f"{name} must fit a signed 64-bit integer")
    return value


def _backend_timestamp(value: Any, name: str) -> int:
    result = _exact_i64(value, name)
    if result < 0:
        raise BackendCallError(f"backend returned negative {name}", code="invalid_backend_response")
    return result


def _input_bool(value: Any, name: str) -> bool:
    if type(value) is bool:
        return value
    if type(value) is int:
        return _input_i64(value, name) != 0
    raise TypeError(f"{name} must be a boolean or integer")


def _backend_number(value: Any, name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise BackendCallError(f"backend returned invalid {name}", code="invalid_backend_response")
    try:
        result = float(value)
    except OverflowError as exc:
        raise BackendCallError(
            f"backend returned out-of-range {name}",
            code="invalid_backend_response",
        ) from exc
    if not math.isfinite(result):
        raise BackendCallError(f"backend returned non-finite {name}", code="invalid_backend_response")
    return result


def _backend_vector(value: Any, name: str) -> list[float]:
    if not isinstance(value, list) or len(value) != 3:
        raise BackendCallError(f"backend returned invalid {name}", code="invalid_backend_response")
    return [_backend_number(component, f"{name} component") for component in value]


def _backend_id_list(value: Any, name: str) -> list[int]:
    if not isinstance(value, list) or len(value) > MAX_SNAPSHOT_BALLS:
        raise BackendCallError(f"backend returned invalid {name}", code="invalid_backend_response")
    result = [_exact_i64(ball_id, f"{name} ball ID") for ball_id in value]
    if result != sorted(set(result)):
        raise BackendCallError(
            f"backend returned unsorted or duplicate {name}",
            code="invalid_backend_response",
        )
    return result


def _backend_distance_rows(value: Any) -> list[tuple[float, int]]:
    if not isinstance(value, list) or len(value) > MAX_SNAPSHOT_BALLS:
        raise BackendCallError("backend returned invalid range rows", code="invalid_backend_response")
    result = []
    for row in value:
        if not isinstance(row, (list, tuple)) or len(row) != 2:
            raise BackendCallError("backend returned invalid range rows", code="invalid_backend_response")
        distance = _backend_number(row[0], "squared range distance")
        if distance < 0.0:
            raise BackendCallError("backend returned a negative squared distance", code="invalid_backend_response")
        result.append((distance, _exact_i64(row[1], "range ball ID")))
    ids = [row[1] for row in result]
    if ids != sorted(set(ids)):
        raise BackendCallError(
            "backend returned unsorted or duplicate range rows",
            code="invalid_backend_response",
        )
    return result


def _backend_proximity_events(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list) or len(value) > MAX_PROXIMITY_EVENTS:
        raise BackendCallError("backend returned invalid proximity events", code="invalid_backend_response")
    result = []
    for event in value:
        if not isinstance(event, dict) or event.get("kind") not in {"transition", "overflow"}:
            raise BackendCallError("backend returned an invalid proximity event", code="invalid_backend_response")
        kind = event["kind"]
        if kind == "transition":
            expected = {"kind", "owner_id", "other_id", "sensor_index", "entering", "tick"}
            if set(event) != expected or type(event.get("entering")) is not bool:
                raise BackendCallError("backend returned an invalid transition event", code="invalid_backend_response")
            normalized = {
                "kind": kind,
                "owner_id": _exact_i64(event["owner_id"], "proximity owner ID"),
                "other_id": _exact_i64(event["other_id"], "proximity other ID"),
                "sensor_index": _exact_i64(event["sensor_index"], "proximity sensor index"),
                "entering": event["entering"],
                "tick": _exact_i64(event["tick"], "proximity tick"),
            }
        else:
            reason = event.get("reason")
            expected = {"kind", "owner_id", "sensor_index", "reason", "tick"}
            if reason == "event_backpressure":
                expected.add("required_events")
            if reason not in {"work_budget", "event_backpressure"} or set(event) != expected:
                raise BackendCallError("backend returned an invalid overflow event", code="invalid_backend_response")
            normalized = {
                "kind": kind,
                "owner_id": _exact_i64(event["owner_id"], "proximity owner ID"),
                "sensor_index": _exact_i64(event["sensor_index"], "proximity sensor index"),
                "reason": reason,
                "tick": _exact_i64(event["tick"], "proximity tick"),
            }
            if reason == "event_backpressure":
                normalized["required_events"] = _exact_i64(
                    event["required_events"],
                    "required proximity event count",
                )
        if normalized["sensor_index"] < 0 or normalized["tick"] < 0:
            raise BackendCallError("backend returned a negative proximity index/tick", code="invalid_backend_response")
        if normalized.get("required_events", 0) < 0:
            raise BackendCallError("backend returned a negative proximity event count", code="invalid_backend_response")
        result.append(normalized)
    return result


def _bounded_i64_list(values: Any, name: str) -> list[int]:
    if isinstance(values, (str, bytes, bytearray, memoryview)):
        raise TypeError(f"{name} must be an iterable of integers")
    try:
        iterator = iter(values)
    except TypeError as exc:
        raise TypeError(f"{name} must be an iterable of integers") from exc
    result = []
    for value in iterator:
        if len(result) >= MAX_SNAPSHOT_BALLS:
            raise ValueError(f"{name} exceeds the snapshot ball-count limit")
        result.append(_input_i64(value, name))
    return result


def _validate_park_handle(value: Any) -> str:
    if not isinstance(value, str) or not value.startswith("park:"):
        raise BackendCallError("backend returned an invalid park handle", code="invalid_backend_response")
    suffix = value[5:]
    if not suffix.isascii() or not suffix.isdigit() or len(suffix) > 20:
        raise BackendCallError("backend returned a non-canonical park handle", code="invalid_backend_response")
    try:
        parsed = int(suffix)
    except ValueError as exc:
        raise BackendCallError(
            "backend returned an invalid park handle",
            code="invalid_backend_response",
        ) from exc
    if parsed > 2**64 - 1 or str(parsed) != suffix:
        raise BackendCallError("backend returned a non-canonical park handle", code="invalid_backend_response")
    return value


def _validate_ball_handle(value: Any, park_handle: str, expected_id: int | None = None) -> str:
    if not isinstance(value, str):
        raise BackendCallError("backend returned an invalid ball handle", code="invalid_backend_response")
    park_number = park_handle.rsplit(":", 1)[1]
    prefix = f"ball:{park_number}:"
    if not value.startswith(prefix):
        raise BackendCallError("backend returned a ball for a different park", code="invalid_backend_response")
    encoded_id = value[len(prefix):]
    ball_id = _exact_i64(encoded_id, "ball handle", allow_canonical_string=True)
    if expected_id is not None and ball_id != expected_id:
        raise BackendCallError(
            "backend returned a handle for the wrong ball",
            code="invalid_backend_response",
        )
    return value


def set_backend(backend: Backend) -> Backend:
    global _default_backend
    if not isinstance(backend, Backend):
        raise TypeError("backend must implement invoke(), update(), release(), and close()")
    to_close = None
    with _backend_state_lock:
        previous = _default_backend
        _default_backend = backend
        _retired_backends.discard(id(backend))
        if previous is not None and previous is not backend:
            to_close = _retire_backend_locked(previous)
    if to_close is not None:
        to_close.close()
    return backend


def get_backend() -> Backend:
    global _default_backend
    with _backend_state_lock:
        if _default_backend is None:
            _default_backend = NativeBackend.discover()
        return _default_backend


def use_in_memory_backend() -> InMemoryBackend:
    return set_backend(InMemoryBackend())  # type: ignore[return-value]


def use_native_backend(library_path: str | None = None, options: dict[str, Any] | None = None) -> NativeBackend:
    backend = NativeBackend(library_path, options) if library_path else NativeBackend.discover(options)
    return set_backend(backend)  # type: ignore[return-value]


def clear_backend() -> None:
    global _default_backend
    to_close = None
    with _backend_state_lock:
        backend, _default_backend = _default_backend, None
        if backend is not None:
            to_close = _retire_backend_locked(backend)
    if to_close is not None:
        to_close.close()


def _retire_backend_locked(backend: Backend) -> Backend | None:
    key = id(backend)
    state = _backend_users.get(key)
    if state is None or state[1] == 0:
        _backend_users.pop(key, None)
        _retired_backends.discard(key)
        return backend
    _retired_backends.add(key)
    return None


def _acquire_ballpark_backend(explicit: Backend | None) -> Backend:
    global _default_backend
    with _backend_state_lock:
        if explicit is None:
            if _default_backend is None:
                _default_backend = NativeBackend.discover()
            backend = _default_backend
        else:
            if not isinstance(explicit, Backend):
                raise TypeError("_backend must implement the Backend protocol")
            backend = explicit
        key = id(backend)
        existing = _backend_users.get(key)
        count = 0 if existing is None else existing[1]
        _backend_users[key] = (backend, count + 1)
        return backend


def _release_ballpark_backend(backend: Backend) -> None:
    to_close = None
    with _backend_state_lock:
        key = id(backend)
        existing = _backend_users.get(key)
        if existing is None or existing[0] is not backend or existing[1] <= 0:
            return
        count = existing[1] - 1
        if count:
            _backend_users[key] = (backend, count)
        else:
            _backend_users.pop(key, None)
            if key in _retired_backends:
                _retired_backends.remove(key)
                to_close = backend
    if to_close is not None:
        to_close.close()


class _BoundObject:
    __slots__ = ("_backend", "_handle")

    def __init__(self, backend: Backend, handle: str) -> None:
        object.__setattr__(self, "_backend", backend)
        object.__setattr__(self, "_handle", handle)

    def _invoke(self, title: str, operation: str = "call", *args: Any, **kwargs: Any) -> Any:
        return self._backend.invoke(title, operation, self._handle, args, kwargs)


class Ball(_BoundObject):
    """Destiny ``Ball`` whose state is stored in Bevy/Avian components."""

    __slots__ = ("_park",)

    def __init__(self, ball_id: int = 0, *, _backend: Backend | None = None, _handle: str | None = None, _park=None) -> None:
        if _backend is None or _handle is None:
            raise TypeError("Ball instances are created by Ballpark.AddBall")
        super().__init__(_backend, _handle)
        object.__setattr__(self, "_park", _park)

    @property
    def ballpark(self):
        return self._park

    def _invoke(self, title: str, operation: str = "call", *args: Any, **kwargs: Any) -> Any:
        if self._park is not None and self._park._closed:
            raise BackendCallError("ballpark is closed", code="closed")
        if self._park is not None and self._park._driver_error is not None:
            error = self._park._driver_error
            raise BackendCallError(
                f"automatic Destiny driver failed: {error}",
                code="driver_failed",
                details=repr(error),
            ) from error
        return super()._invoke(title, operation, *args, **kwargs)

    def GetRotatedVector(self, vector: list[float]) -> list[float]:
        if not isinstance(vector, list) or len(vector) != 3:
            raise TypeError("GetRotatedVector expects a three-element list")
        rotated = _backend_vector(
            self._invoke("destiny.Ball.GetRotatedVector", "call", vector),
            "rotated vector",
        )
        vector[:] = rotated
        return vector

    def AddMiniBall(self, x: float, y: float, z: float, radius: float) -> None:
        self._invoke("destiny.Ball.AddMiniBall", "call", x, y, z, radius)

    def AddMiniCapsule(self, ax, ay, az, bx, by, bz, radius) -> None:
        self._invoke("destiny.Ball.AddMiniCapsule", "call", ax, ay, az, bx, by, bz, radius)

    def AddMiniBox(self, c0, c1, c2, x0, x1, x2, y0, y1, y2, z0, z1, z2) -> None:
        self._invoke("destiny.Ball.AddMiniBox", "call", c0, c1, c2, x0, x1, x2, y0, y1, y2, z0, z1, z2)

    def AddProximitySensor(self, range_value, period=2.0, shuffle=0, onlyInteractives=False) -> None:
        self._invoke("destiny.Ball.AddProximitySensor", "call", range_value, period, shuffle, onlyInteractives)

    def __getattr__(self, name: str) -> Any:
        canonical_title = f"destiny.Ball.{name}"
        record = RECORDS.get(canonical_title)
        if record is None:
            raise AttributeError(name)
        raise UnsupportedTitleError.for_title(canonical_title)

    def __setattr__(self, name: str, value: Any) -> None:
        if name.startswith("_"):
            object.__setattr__(self, name, value)
            return
        descriptor = getattr(type(self), name, None)
        if isinstance(descriptor, property):
            if descriptor.fset is None:
                raise AttributeError(f"{type(self).__name__}.{name} is read-only")
            descriptor.__set__(self, value)
            return
        canonical_title = f"destiny.Ball.{name}"
        if canonical_title in RECORDS:
            raise UnsupportedTitleError.for_title(canonical_title)
        object.__setattr__(self, name, value)


def _ball_property(name: str, *, writable: bool = True):
    title = f"destiny.Ball.{name}"

    def getter(self: Ball):
        value = self._invoke(title, "get")
        if name in {"isFree", "isGlobal", "isMassive", "isInteractive"}:
            if type(value) is not bool:
                raise BackendCallError(
                    f"backend returned invalid Ball.{name}",
                    code="invalid_backend_response",
                )
            return value
        if name in {"id", "isCloaked", "newBubbleId", "oldBubbleId", "effectStamp"}:
            result = _exact_i64(value, f"Ball.{name}")
            if name == "id":
                expected_id = _exact_i64(
                    self._handle.rsplit(":", 1)[1],
                    "bound ball handle",
                    allow_canonical_string=True,
                )
                if result != expected_id:
                    raise BackendCallError(
                        "backend returned an ID that does not match the bound ball",
                        code="invalid_backend_response",
                    )
            if name == "isCloaked" and result not in {0, 1, 2, 3}:
                raise BackendCallError(
                    "backend returned an invalid cloak mode",
                    code="invalid_backend_response",
                )
            return result
        return _backend_number(value, f"Ball.{name}")

    if not writable:
        return property(getter)

    def setter(self: Ball, value):
        self._invoke(title, "set", value)

    return property(getter, setter)


for _field_name in (
    "mass", "radius", "maxVelocity", "maxAngularVelocity", "x", "y", "z",
    "vx", "vy", "vz", "wx", "wy", "wz", "rx", "ry", "rz", "rw",
    "isFree", "isGlobal", "isMassive", "isInteractive", "isCloaked",
    "Agility", "speedFraction",
):
    setattr(Ball, _field_name, _ball_property(_field_name))

for _field_name in ("id", "roll", "pitch", "yaw", "newBubbleId", "oldBubbleId", "effectStamp"):
    setattr(Ball, _field_name, _ball_property(_field_name, writable=False))


class ClientBall(Ball):
    @property
    def centerDist(self) -> float:
        return _backend_number(self._invoke("destiny.ClientBall.centerDist", "get"), "center distance")

    @property
    def surfaceDist(self) -> float:
        return _backend_number(self._invoke("destiny.ClientBall.surfaceDist", "get"), "surface distance")

    def ApplyImpulsiveForceAtPosition(self, force, position) -> None:
        self._invoke("destiny.ClientBall.ApplyImpulsiveForceAtPosition", "call", list(force), list(position))

    def __getattr__(self, name: str) -> Any:
        canonical_title = f"destiny.ClientBall.{name}"
        if canonical_title in RECORDS:
            raise UnsupportedTitleError.for_title(canonical_title)
        return super().__getattr__(name)


class _BallsView(Mapping[int, Ball]):
    def __init__(self, park: "Ballpark") -> None:
        self._park = park

    def __getitem__(self, ball_id: int) -> Ball:
        return self._park.GetBall(ball_id)

    def __iter__(self) -> Iterator[int]:
        targets = self._park._invoke("dbc.compat.Ballpark.ListBalls")
        if not isinstance(targets, list) or len(targets) > MAX_SNAPSHOT_BALLS:
            raise BackendCallError("backend returned an invalid ball list", code="invalid_backend_response")
        ball_ids = []
        for target in targets:
            validated = _validate_ball_handle(target, self._park._handle)
            ball_id = _exact_i64(validated.rsplit(":", 1)[1], "ball handle", allow_canonical_string=True)
            ball_ids.append(ball_id)
        if ball_ids != sorted(set(ball_ids)):
            raise BackendCallError(
                "backend returned unsorted or duplicate ball handles",
                code="invalid_backend_response",
            )
        for ball_id in ball_ids:
            yield ball_id

    def __len__(self) -> int:
        return sum(1 for _ in self)


class Ballpark(_BoundObject):
    """Destiny Ballpark facade backed by one Bevy ``App``/``World`` runtime."""

    __slots__ = (
        "_balls",
        "_closed",
        "_driver_stop",
        "_driver_thread",
        "_driver_lock",
        "_driver_error",
        "_driver_generation",
        "_is_master",
        "_backend_registered",
    )

    def __init__(self, isMaster: bool = False, *, _backend: Backend | None = None) -> None:
        is_master = _input_bool(isMaster, "isMaster")
        backend = _acquire_ballpark_backend(_backend)
        handle = None
        try:
            handle = backend.invoke("destiny.Ballpark.__init__", "construct", None, (is_master,), {})
            validated_handle = _validate_park_handle(handle)
        except Exception:
            # A backend can allocate a runtime and still return a malformed
            # handle. Release both ownership layers before surfacing that
            # protocol failure; the old path leaked the facade registration
            # (and potentially the backend runtime) on this boundary.
            if isinstance(handle, str):
                try:
                    backend.release(handle)
                except Exception:
                    logger.exception("failed to release a backend after invalid construction output")
            _release_ballpark_backend(backend)
            raise
        super().__init__(backend, validated_handle)
        object.__setattr__(self, "_balls", _BallsView(self))
        object.__setattr__(self, "_closed", False)
        object.__setattr__(self, "_driver_stop", Event())
        object.__setattr__(self, "_driver_thread", None)
        object.__setattr__(self, "_driver_lock", RLock())
        object.__setattr__(self, "_driver_error", None)
        object.__setattr__(self, "_driver_generation", 0)
        object.__setattr__(self, "_is_master", is_master)
        object.__setattr__(self, "_backend_registered", True)

    @property
    def balls(self) -> Mapping[int, Ball]:
        # This is a wrapper-owned ECS view. The audit correctly found no single
        # stock Bevy field equivalent for Destiny's Blue dictionary.
        return self._balls

    def _invoke(self, title: str, operation: str = "call", *args: Any, **kwargs: Any) -> Any:
        with self._driver_lock:
            if self._closed:
                raise BackendCallError("ballpark is closed", code="closed")
            if self._driver_error is not None and title not in {
                "destiny.Ballpark.Pause",
                "destiny.Ballpark.isRunning",
            }:
                raise BackendCallError(
                    f"automatic Destiny driver failed: {self._driver_error}",
                    code="driver_failed",
                    details=repr(self._driver_error),
                ) from self._driver_error
            return super()._invoke(title, operation, *args, **kwargs)

    @property
    def bubbleInteractives(self) -> dict[int, list[int]]:
        return self.GetBubbleMembership()["interactives"]

    def GetBubbleMembership(self) -> dict[str, dict[int, list[int]]]:
        """Return one lock-consistent visibility/membership snapshot."""
        raw = self._invoke("dbc.compat.Ballpark.BubbleMembership")
        if not isinstance(raw, dict) or set(raw) != {"interactives", "members", "observers"}:
            raise BackendCallError("backend returned invalid bubble membership", code="invalid_backend_response")
        result: dict[str, dict[int, list[int]]] = {}
        for name, rows in raw.items():
            if not isinstance(rows, dict):
                raise BackendCallError("backend returned invalid bubble membership rows", code="invalid_backend_response")
            normalized_rows = {}
            for key, value in rows.items():
                normalized_key = _exact_i64(key, "membership key", allow_canonical_string=True)
                if normalized_key in normalized_rows or not isinstance(value, list):
                    raise BackendCallError("backend returned invalid bubble membership rows", code="invalid_backend_response")
                normalized_ids = [_exact_i64(ball_id, "membership ball id") for ball_id in value]
                if normalized_ids != sorted(set(normalized_ids)):
                    raise BackendCallError("backend returned unsorted or duplicate membership IDs", code="invalid_backend_response")
                normalized_rows[normalized_key] = normalized_ids
            result[name] = normalized_rows
        return result

    def AddBall(self, *args) -> Ball:
        if not args:
            raise TypeError("AddBall requires a source ID and ball parameters")
        source_id = _input_i64(args[0], "source_id")
        handle = self._invoke("destiny.Ballpark.AddBall", "call", *args)
        ball_type = Ball if self._is_master else ClientBall
        return ball_type(
            _backend=self._backend,
            _handle=_validate_ball_handle(handle, self._handle, source_id),
            _park=self,
        )

    def GetBall(self, ball_id: int) -> Ball:
        ball_id = _input_i64(ball_id, "ball_id")
        handle = self._invoke("dbc.compat.Ballpark.GetBall", "call", ball_id)
        ball_type = Ball if self._is_master else ClientBall
        return ball_type(
            _backend=self._backend,
            _handle=_validate_ball_handle(handle, self._handle, ball_id),
            _park=self,
        )

    def HasBall(self, ball_id: int) -> bool:
        result = self._invoke("dbc.compat.Ballpark.HasBall", "call", _input_i64(ball_id, "ball_id"))
        if type(result) is not bool:
            raise BackendCallError("backend returned a non-boolean HasBall result", code="invalid_backend_response")
        return result

    def RemoveBall(self, ball_id: int, delay: int = 0) -> None:
        self._invoke(
            "destiny.Ballpark.RemoveBall",
            "call",
            _input_i64(ball_id, "ball_id"),
            _input_i64(delay, "delay"),
        )

    def ClearAll(self) -> None:
        self._invoke("destiny.Ballpark.ClearAll")

    def Pause(self) -> None:
        self._stop_driver()
        self._invoke("destiny.Ballpark.Pause")

    def Start(self) -> None:
        self._invoke("destiny.Ballpark.Start")
        try:
            self._start_driver()
        except Exception:
            super()._invoke("destiny.Ballpark.Pause")
            raise

    def Evolve(self) -> None:
        with self._driver_lock:
            thread = self._driver_thread
            if thread is not None and thread.is_alive() and thread is not current_thread():
                raise BackendCallError(
                    "manual Evolve is unavailable while the automatic driver owns the scheduler",
                    code="scheduler_busy",
                )
            self._invoke("destiny.Ballpark.Evolve")

    def SetBallPosition(self, ball_id, x, y, z) -> None:
        self._invoke("destiny.Ballpark.SetBallPosition", "call", ball_id, x, y, z)

    def SetBallVelocity(self, ball_id, vx, vy, vz) -> None:
        self._invoke("destiny.Ballpark.SetBallVelocity", "call", ball_id, vx, vy, vz)

    def SetBallAngularVelocity(self, ball_id, wx, wy, wz) -> None:
        self._invoke("destiny.Ballpark.SetBallAngularVelocity", "call", ball_id, wx, wy, wz)

    def SetBallRotation(self, ball_id, rx, ry, rz, rw) -> None:
        self._invoke("destiny.Ballpark.SetBallRotation", "call", ball_id, rx, ry, rz, rw)

    def SetBallMass(self, ball_id, mass) -> None:
        self._invoke("destiny.Ballpark.SetBallMass", "call", ball_id, mass)

    def SetMaxSpeed(self, ball_id, speed) -> None:
        self._invoke("destiny.Ballpark.SetMaxSpeed", "call", ball_id, speed)

    def SetMaxAngularSpeed(self, ball_id, speed) -> None:
        self._invoke("destiny.Ballpark.SetMaxAngularSpeed", "call", ball_id, speed)

    def GetSurfaceDist(self, first_id, second_id):
        result = self._invoke("destiny.Ballpark.GetSurfaceDist", "call", first_id, second_id)
        return None if result is None else _backend_number(result, "surface distance")

    def GetCenterDist(self, first_id, second_id):
        result = self._invoke("destiny.Ballpark.GetCenterDist", "call", first_id, second_id)
        return None if result is None else _backend_number(result, "center distance")

    def AddProximitySensor(self, ball_id, range_value, period=2.0, shuffle=0, onlyInteractives=False) -> None:
        self._invoke("destiny.Ballpark.AddProximitySensor", "call", ball_id, range_value, period, shuffle, onlyInteractives)

    def RemoveProximitySensor(self, ball_id) -> None:
        self._invoke("destiny.Ballpark.RemoveProximitySensor", "call", ball_id)

    def WriteFullStateToStream(self, stream, source_id: int = -1) -> None:
        encoded = self._invoke(
            "dbc.compat.Ballpark.Serialize",
            "call",
            None,
            _input_i64(source_id, "source_id"),
        )
        _write_stream(stream, _decode_snapshot_response(encoded))

    def WriteBallsToStream(self, ball_ids, stream) -> None:
        encoded = self._invoke(
            "dbc.compat.Ballpark.Serialize",
            "call",
            _bounded_i64_list(ball_ids, "ball_ids"),
        )
        _write_stream(stream, _decode_snapshot_response(encoded))

    def ReadFullStateFromStream(self, stream, partial: int = 0, *, _restart_driver: bool = True) -> None:
        if type(partial) is not int or partial not in {0, 1, 2}:
            raise ValueError("partial must be one of 0, 1, or 2")
        encoded = base64.b64encode(_read_stream(stream, MAX_SNAPSHOT_BYTES)).decode("ascii")
        self._stop_driver()
        try:
            self._invoke("dbc.compat.Ballpark.Deserialize", "call", encoded, partial)
        finally:
            if not _restart_driver and not self._closed:
                super()._invoke("destiny.Ballpark.Pause")
            elif not self._closed and self.isRunning:
                self._start_driver()

    def CaptureFullState(self, source_id: int = -1) -> tuple[int, bytes]:
        """Capture ``(currentTime, bytes)`` in one backend transaction."""
        result = self._invoke(
            "dbc.compat.Ballpark.CaptureSnapshot",
            "call",
            _input_i64(source_id, "source_id"),
        )
        if not isinstance(result, dict) or set(result) != {"current_time", "snapshot"}:
            raise BackendCallError("backend returned an invalid atomic snapshot", code="invalid_backend_response")
        encoded = result["snapshot"]
        if not isinstance(encoded, str):
            raise BackendCallError("backend returned an invalid atomic snapshot", code="invalid_backend_response")
        current_time = _backend_timestamp(result["current_time"], "snapshot timestamp")
        return current_time, _decode_snapshot_response(
            encoded,
            expected_current_time=current_time,
        )

    @property
    def driverError(self):
        return self._driver_error

    def ClearDriverError(self) -> None:
        with self._driver_lock:
            thread = self._driver_thread
            if thread is not None and thread.is_alive():
                raise BackendCallError("cannot clear a live driver failure", code="scheduler_busy")
            object.__setattr__(self, "_driver_error", None)

    def DrainProximityEvents(self) -> list[dict[str, Any]]:
        return _backend_proximity_events(self._invoke("dbc.compat.Proximity.DrainEvents"))

    def close(self) -> None:
        if self._closed:
            return
        self._stop_driver()
        with self._driver_lock:
            try:
                self._backend.release(self._handle)
            finally:
                object.__setattr__(self, "_closed", True)
                if self._backend_registered:
                    object.__setattr__(self, "_backend_registered", False)
                    _release_ballpark_backend(self._backend)

    def __enter__(self) -> "Ballpark":
        if self._closed:
            raise BackendCallError("ballpark is closed", code="closed")
        return self

    def __exit__(self, exc_type, exc, traceback) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - destructors must never propagate
            pass

    def _start_driver(self) -> None:
        with self._driver_lock:
            if self._closed:
                raise BackendCallError("ballpark is closed", code="closed")
            if self._driver_error is not None:
                raise BackendCallError(
                    f"automatic Destiny driver failed: {self._driver_error}",
                    code="driver_failed",
                ) from self._driver_error
            thread = self._driver_thread
            if thread is not None and thread.is_alive():
                return
            generation = self._driver_generation + 1
            stop = Event()
            object.__setattr__(self, "_driver_generation", generation)
            object.__setattr__(self, "_driver_stop", stop)
            thread = Thread(
                target=self._drive,
                args=(stop, generation),
                name=f"destiny-{self._handle}-{generation}",
                daemon=True,
            )
            object.__setattr__(self, "_driver_thread", thread)
            thread.start()

    def _stop_driver(self, timeout: float = 5.0) -> None:
        with self._driver_lock:
            stop = self._driver_stop
            thread = self._driver_thread
            stop.set()
        if thread is not None and thread is not current_thread():
            thread.join(timeout=timeout)
            if thread.is_alive():
                raise BackendCallError(
                    "automatic Destiny driver did not stop before the timeout",
                    code="driver_stop_timeout",
                )
        with self._driver_lock:
            if self._driver_thread is thread and (thread is None or not thread.is_alive()):
                object.__setattr__(self, "_driver_thread", None)

    def _drive(self, stop: Event, generation: int) -> None:
        while not stop.is_set():
            try:
                with self._driver_lock:
                    if generation != self._driver_generation or self._driver_thread is not current_thread():
                        return
                # Backend calls can block in product transports. Never hold the
                # lifecycle lock across one: Pause/close must remain able to set
                # the generation-specific stop event and enforce their timeout.
                tick_interval = _backend_number(
                    super()._invoke("destiny.Ballpark.tickInterval", "get"),
                    "tick interval",
                )
                if tick_interval <= 0.0:
                    raise BackendCallError(
                        "backend returned a non-positive tick interval",
                        code="invalid_backend_response",
                    )
                interval = max(0.001, tick_interval / 1000.0)
                if stop.wait(interval):
                    return
                with self._driver_lock:
                    if generation != self._driver_generation or stop.is_set():
                        return
                self._backend.update(self._handle)
            except Exception as exc:  # noqa: BLE001 - retain and surface terminal driver failure
                logger.exception("automatic Destiny driver failed for %s", self._handle)
                with self._driver_lock:
                    object.__setattr__(self, "_driver_error", exc)
                stop.set()
                try:
                    self._backend.invoke("destiny.Ballpark.Pause", "call", self._handle, (), {})
                except Exception:
                    logger.exception("failed to pause backend after automatic driver failure")
                return

    def __getattr__(self, name: str) -> Any:
        original_name = name
        if name.startswith("_parent_"):
            name = name[len("_parent_"):]
        canonical_title = f"destiny.Ballpark.{name}"
        record = RECORDS.get(canonical_title)
        if record is None:
            raise AttributeError(original_name)
        if name in _SUPPORTED_DYNAMIC_PARK_METHODS:
            return lambda *args, **kwargs: self._invoke(canonical_title, "call", *args, **kwargs)
        raise UnsupportedTitleError.for_title(canonical_title)

    def __setattr__(self, name: str, value: Any) -> None:
        if name.startswith("_"):
            object.__setattr__(self, name, value)
            return
        descriptor = getattr(type(self), name, None)
        if isinstance(descriptor, property):
            if descriptor.fset is None:
                raise AttributeError(f"{type(self).__name__}.{name} is read-only")
            descriptor.__set__(self, value)
            return
        canonical_title = f"destiny.Ballpark.{name}"
        if canonical_title in RECORDS:
            raise UnsupportedTitleError.for_title(canonical_title)
        object.__setattr__(self, name, value)


def _park_property(name: str, *, writable: bool = True):
    title = f"destiny.Ballpark.{name}"

    def getter(self: Ballpark):
        value = self._invoke(title, "get")
        if name in {"isRunning", "isMaster"}:
            if type(value) is not bool:
                raise BackendCallError(
                    f"backend returned invalid Ballpark.{name}",
                    code="invalid_backend_response",
                )
            return value
        if name == "currentTime":
            return _backend_timestamp(value, "Ballpark.currentTime")
        if name in {"time", "ego"}:
            return _exact_i64(value, f"Ballpark.{name}")
        return _backend_number(value, f"Ballpark.{name}")

    if not writable:
        return property(getter)

    def setter(self: Ballpark, value):
        self._invoke(title, "set", value)

    return property(getter, setter)


for _field_name in ("tickInterval", "friction", "time", "ego"):
    setattr(Ballpark, _field_name, _park_property(_field_name))

for _field_name in ("isRunning", "currentTime", "isMaster"):
    setattr(Ballpark, _field_name, _park_property(_field_name, writable=False))


def _park_method(name: str):
    """Create an inspectable wrapper for a supported native park call."""

    title = f"destiny.Ballpark.{name}"

    def method(self: Ballpark, *args: Any, **kwargs: Any):
        result = self._invoke(title, "call", *args, **kwargs)
        if name in {
            "GetBallIdsInCapsule",
            "GetBallIdsInCone",
            "GetBallIdsInRange",
            "GetBallIdsInRangeOfTriangle",
        }:
            return _backend_id_list(result, name)
        if name == "GetBallIdsAndDistInRange":
            return _backend_distance_rows(result)
        if name in {"GetBoxCenter", "GetCurrentEgoPos"}:
            return _backend_vector(result, name)
        if name == "ScanCone":
            return None if result is None else _backend_id_list(result, name)
        if name == "CheckVisibility":
            return _exact_i64(result, name)
        if result is not None:
            raise BackendCallError(
                f"backend returned an unexpected result for {name}",
                code="invalid_backend_response",
            )
        return None

    method.__name__ = name
    method.__qualname__ = f"Ballpark.{name}"
    return method


# These used to exist only through ``__getattr__``. Installing real class
# members makes ``dir()``, static analysis, and the generated reachability
# ledger agree with what callers can actually invoke.
for _method_name in sorted(_SUPPORTED_DYNAMIC_PARK_METHODS):
    if not hasattr(Ballpark, _method_name):
        setattr(Ballpark, _method_name, _park_method(_method_name))


def _write_stream(stream: Any, data: bytes) -> None:
    writer = getattr(stream, "Write", None) or getattr(stream, "write", None)
    if writer is None:
        raise TypeError("stream must provide Write/write")
    if hasattr(stream, "Seek"):
        try:
            stream.Seek(0, 0)
        except TypeError:
            stream.Seek(0)
        truncator = getattr(stream, "Truncate", None)
        if truncator is not None:
            try:
                truncator(0)
            except TypeError:
                truncator()
        elif hasattr(stream, "SetSize"):
            stream.SetSize(0)
    elif hasattr(stream, "seek"):
        stream.seek(0)
        if hasattr(stream, "truncate"):
            stream.truncate(0)
    written = writer(data)
    if written is not None and (isinstance(written, bool) or written != len(data)):
        raise OSError("stream Write/write did not consume the complete snapshot")
    if hasattr(stream, "Seek"):
        try:
            stream.Seek(0, 0)
        except TypeError:
            stream.Seek(0)
    elif hasattr(stream, "seek"):
        stream.seek(0)


def _read_stream(stream: Any, limit: int = MAX_SNAPSHOT_BYTES) -> bytes:
    if not isinstance(limit, int) or limit <= 0:
        raise ValueError("stream limit must be a positive integer")
    if isinstance(stream, (bytes, bytearray, memoryview)):
        size = stream.nbytes if isinstance(stream, memoryview) else len(stream)
        if size > limit:
            raise BackendCallError("snapshot stream exceeds the configured byte limit", code="snapshot_too_large")
        data = stream.tobytes() if isinstance(stream, memoryview) else bytes(stream)
        if len(data) != size:
            raise BackendCallError("snapshot stream changed size while reading", code="invalid_stream")
        return data
    if hasattr(stream, "Seek"):
        try:
            stream.Seek(0, 0)
        except TypeError:
            stream.Seek(0)
    elif hasattr(stream, "seek"):
        stream.seek(0)
    reader = getattr(stream, "Read", None) or getattr(stream, "read", None)
    if reader is None:
        raise TypeError("stream must provide Read/read")
    chunks = []
    total = 0
    while True:
        try:
            chunk = reader(min(65536, limit + 1 - total))
        except TypeError as exc:
            raise TypeError(
                "stream Read/read must accept a bounded byte-count argument"
            ) from exc
        if not chunk:
            break
        if isinstance(chunk, str):
            if len(chunk) > limit - total:
                raise BackendCallError(
                    "snapshot stream exceeds the configured byte limit",
                    code="snapshot_too_large",
                )
            chunk = chunk.encode("latin-1")
        elif isinstance(chunk, (bytes, bytearray, memoryview)):
            size = chunk.nbytes if isinstance(chunk, memoryview) else len(chunk)
            if size > limit - total:
                raise BackendCallError(
                    "snapshot stream exceeds the configured byte limit",
                    code="snapshot_too_large",
                )
            chunk = chunk.tobytes() if isinstance(chunk, memoryview) else bytes(chunk)
            if len(chunk) != size:
                raise BackendCallError("snapshot stream changed size while reading", code="invalid_stream")
        else:
            raise TypeError("stream Read/read must return a bytes-like object")
        total += len(chunk)
        if total > limit:
            raise BackendCallError("snapshot stream exceeds the configured byte limit", code="snapshot_too_large")
        chunks.append(chunk)
        if len(chunk) == 0:
            break
    return b"".join(chunks)


def _decode_snapshot_response(encoded: Any, *, expected_current_time: int | None = None) -> bytes:
    """Validate a backend snapshot before copying it into a caller's stream."""

    if not isinstance(encoded, str):
        raise BackendCallError("backend returned a non-string snapshot", code="invalid_backend_response")
    try:
        raw_encoded = encoded.encode("ascii")
    except UnicodeEncodeError as exc:
        raise BackendCallError("backend returned non-ASCII snapshot base64", code="invalid_backend_response") from exc
    encoded_limit = 4 * ((MAX_SNAPSHOT_BYTES + 2) // 3)
    if len(raw_encoded) > encoded_limit:
        raise BackendCallError("backend snapshot exceeds the configured limit", code="response_too_large")
    try:
        payload = base64.b64decode(raw_encoded, validate=True)
    except Exception as exc:  # noqa: BLE001 - normalize malformed backend output
        raise BackendCallError("backend returned invalid snapshot base64", code="invalid_backend_response") from exc
    if base64.b64encode(payload) != raw_encoded:
        raise BackendCallError("backend returned non-canonical snapshot base64", code="invalid_backend_response")
    if len(payload) > MAX_SNAPSHOT_BYTES:
        raise BackendCallError("backend snapshot exceeds the configured limit", code="response_too_large")
    def reject_duplicates(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"duplicate snapshot field {key!r}")
            result[key] = value
        return result

    try:
        snapshot = json.loads(payload.decode("utf-8"), object_pairs_hook=reject_duplicates)
    except (UnicodeError, ValueError, json.JSONDecodeError, RecursionError) as exc:
        raise BackendCallError(
            "backend returned invalid snapshot JSON",
            code="invalid_backend_response",
        ) from exc
    expected_root = {"format", "schema_version", "park", "balls"}
    expected_park = {
        "is_master",
        "running",
        "tick_interval_ms",
        "friction",
        "current_time",
        "time",
        "ego",
        "collision_substeps",
        "use_iterative_collision",
        "use_dynamical_orientation",
        "disable_dynamical_orientation_for_missiles",
        "use_new_orbit",
        "pending_removals",
        "snapshot_semantics",
    }
    if (
        not isinstance(snapshot, dict)
        or set(snapshot) != expected_root
        or snapshot.get("format") != "destiny-bevy-compat-state-v3"
        or type(snapshot.get("schema_version")) is not int
        or snapshot.get("schema_version") != 3
        or not isinstance(snapshot.get("park"), dict)
        or set(snapshot["park"]) != expected_park
        or not isinstance(snapshot.get("balls"), list)
        or len(snapshot["balls"]) > MAX_SNAPSHOT_BALLS
    ):
        raise BackendCallError(
            "backend returned a snapshot with an invalid envelope",
            code="invalid_backend_response",
        )
    embedded_time = _backend_timestamp(snapshot["park"].get("current_time"), "snapshot current_time")
    if expected_current_time is not None and embedded_time != expected_current_time:
        raise BackendCallError(
            "backend snapshot timestamp does not match its atomic capture label",
            code="invalid_backend_response",
        )
    return payload
