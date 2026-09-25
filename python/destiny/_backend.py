"""Backend protocol, Bevy C-ABI loader, and deterministic test backend."""

from __future__ import annotations

import base64
import binascii
from collections import defaultdict
import copy
import ctypes
from dataclasses import asdict, dataclass, field
import json
import math
import os
from pathlib import Path
import sys
from threading import RLock
from typing import Any, Protocol, runtime_checkable

from ._errors import (
    AbiMismatchError,
    BackendCallError,
    BackendUnavailableError,
    BallNotFoundError,
    UnsupportedTitleError,
)


ABI_VERSION = 1
F64_MAX = sys.float_info.max
MAX_REQUEST_BYTES = 48 * 1024 * 1024
MAX_RESPONSE_BYTES = 48 * 1024 * 1024
MAX_SNAPSHOT_BYTES = 32 * 1024 * 1024
MAX_SNAPSHOT_ENCODED_BYTES = 4 * ((MAX_SNAPSHOT_BYTES + 2) // 3)
MAX_SNAPSHOT_BALLS = 100_000
MAX_LIVE_BALLS = 100_000
MAX_CHILD_SHAPES_PER_BALL = 4_096
MAX_CHILD_DESCRIPTOR_BYTES = 48 * 1024 * 1024
MAX_OUTBOX_MESSAGES = 10_000
MAX_OUTBOX_BYTES = 48 * 1024 * 1024
MAX_COLLISION_SUBSTEPS = 1_024
MAX_PROXIMITY_EVENTS = 10_000
MAX_PROXIMITY_WORK_PER_TICK = 2_000_000
MAX_EXPANDED_NETWORK_ROWS = 200_000
MAX_COORDINATE = 1.0e100
MAX_RADIUS = 1.0e50
MAX_MASS = 1.0e100
MAX_VELOCITY = 1.0e100
MAX_AGILITY = 1.0e100
I64_MIN = -(2**63)
I64_MAX = 2**63 - 1


@runtime_checkable
class Backend(Protocol):
    def invoke(
        self,
        canonical_title: str,
        operation: str = "call",
        target: str | None = None,
        args: tuple[Any, ...] | list[Any] = (),
        kwargs: dict[str, Any] | None = None,
    ) -> Any: ...

    def update(self, target: str | None = None) -> None: ...

    def release(self, target: str) -> None: ...

    def close(self) -> None: ...


class _DbcBuffer(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.c_void_p),
        ("len", ctypes.c_size_t),
        ("status", ctypes.c_int32),
    ]


class NativeBackend:
    """ctypes adapter for the stable JSON-over-C ABI exported by the Rust crate."""

    def __init__(self, library_path: str | os.PathLike[str], options: dict[str, Any] | None = None) -> None:
        self.library_path = str(Path(library_path).resolve())
        self._library = ctypes.CDLL(self.library_path)
        self._configure_symbols()
        actual_version = int(self._library.dbc_abi_version())
        if actual_version != ABI_VERSION:
            raise AbiMismatchError(f"native ABI {actual_version} does not match Python ABI {ABI_VERSION}")
        self._options = dict(options or {})
        self._runtimes: dict[str, int] = {}
        self._next_park = 0
        self._active_park: str | None = None
        self._lock = RLock()

    def _create_runtime(self, options: dict[str, Any]) -> int:
        try:
            encoded = _bounded_json_bytes(
                options,
                MAX_REQUEST_BYTES,
                error_code="request_too_large",
            )
        except BackendCallError:
            raise
        except (TypeError, ValueError) as exc:
            raise BackendCallError("runtime options must be finite JSON values", code="invalid_request") from exc
        error = _DbcBuffer()
        runtime = self._library.dbc_runtime_new(encoded, len(encoded), ctypes.byref(error))
        if runtime:
            error_status = int(error.status)
            try:
                unexpected = self._consume(error)
            except Exception:
                self._library.dbc_runtime_free(runtime)
                raise
            if unexpected or error_status != 0:
                self._library.dbc_runtime_free(runtime)
                diagnostic = unexpected.decode("utf-8", "replace") or f"status {error_status}"
                raise BackendUnavailableError(
                    f"native backend returned a runtime with an unexpected diagnostic: {diagnostic}"
                )
        else:
            message = self._consume(error) or b"failed to create Bevy runtime"
            raise BackendUnavailableError(message.decode("utf-8", "replace"))
        return int(runtime)

    def _select_runtime(self, target: str | None) -> tuple[str, int]:
        handle = None
        if target and target.startswith("park:"):
            handle = target
        elif target and target.startswith("ball:"):
            parts = target.split(":")
            if len(parts) == 3 and parts[1].isdigit():
                handle = f"park:{parts[1]}"
        elif target is None:
            handle = self._active_park
        if handle not in self._runtimes:
            raise BackendCallError(f"invalid ballpark target {target!r}", code="invalid_target")
        self._active_park = handle
        return handle, self._runtimes[handle]

    def _configure_symbols(self) -> None:
        library = self._library
        library.dbc_abi_version.argtypes = []
        library.dbc_abi_version.restype = ctypes.c_uint32
        library.dbc_runtime_new.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.POINTER(_DbcBuffer)]
        library.dbc_runtime_new.restype = ctypes.c_void_p
        library.dbc_runtime_free.argtypes = [ctypes.c_void_p]
        library.dbc_runtime_free.restype = None
        library.dbc_runtime_call.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t]
        library.dbc_runtime_call.restype = _DbcBuffer
        library.dbc_runtime_update.argtypes = [ctypes.c_void_p]
        library.dbc_runtime_update.restype = _DbcBuffer
        library.dbc_buffer_free.argtypes = [_DbcBuffer]
        library.dbc_buffer_free.restype = None

    @classmethod
    def discover(cls, options: dict[str, Any] | None = None) -> "NativeBackend":
        attempted: list[str] = []
        explicit = os.environ.get("DESTINY_BEVY_COMPAT_LIBRARY")
        if explicit:
            try:
                candidate = Path(explicit).expanduser().resolve(strict=True)
            except OSError as exc:
                raise BackendUnavailableError(f"Explicit native library cannot be resolved: {explicit}") from exc
            if not candidate.is_file():
                raise BackendUnavailableError(f"Explicit native library is not a file: {candidate}")
            return cls(candidate, options)
        package_dir = Path(__file__).resolve().parent
        names = {
            "win32": ("destiny_bevy_compat.dll", "_destiny_bevy_compat.dll"),
            "darwin": ("libdestiny_bevy_compat.dylib",),
        }.get(sys.platform, ("libdestiny_bevy_compat.so",))
        for name in names:
            attempted.append(str(package_dir / name))
        for candidate in attempted:
            if Path(candidate).is_file():
                return cls(candidate, options)
        raise BackendUnavailableError(
            "No wheel-adjacent Bevy compatibility library was found. Install a platform wheel, "
            "or explicitly opt in to a verified external library with DESTINY_BEVY_COMPAT_LIBRARY. "
            f"Checked: {', '.join(attempted) or '(no paths)'}"
        )

    def _consume(self, buffer: _DbcBuffer) -> bytes:
        try:
            if bool(buffer.data) != bool(buffer.len):
                raise BackendCallError(
                    "native backend returned an invalid pointer/length pair",
                    code="invalid_native_buffer",
                )
            if not buffer.data:
                return b""
            if buffer.len > MAX_RESPONSE_BYTES:
                raise BackendCallError("native response exceeds the response-size limit", code="response_too_large")
            return ctypes.string_at(buffer.data, buffer.len)
        finally:
            self._library.dbc_buffer_free(buffer)

    def invoke(
        self,
        canonical_title: str,
        operation: str = "call",
        target: str | None = None,
        args: tuple[Any, ...] | list[Any] = (),
        kwargs: dict[str, Any] | None = None,
    ) -> Any:
        with self._lock:
            if canonical_title == "destiny.Ballpark.__init__":
                if operation != "construct" or target is not None or kwargs or len(args) > 1:
                    raise BackendCallError(
                        "Ballpark construction requires operation=construct, a null target, no kwargs, and at most isMaster",
                        code="invalid_request",
                    )
                handle = f"park:{self._next_park}"
                self._next_park += 1
                options = dict(self._options)
                options["is_master"] = _boolean(args[0], "isMaster") if args else False
                from ._settings import settings

                configuration = settings.Get()
                options.setdefault("collision_substeps", configuration.collisionMaxIterations)
                options.setdefault("use_iterative_collision", configuration.useIterativeCollision)
                options.setdefault("use_dynamical_orientation", configuration.useDynamicalOrientation)
                options.setdefault(
                    "disable_dynamical_orientation_for_missiles",
                    configuration.disableDynamicalOrientationForMissiles,
                )
                options.setdefault("use_new_orbit", configuration.useNewOrbit)
                self._runtimes[handle] = self._create_runtime(options)
                self._active_park = handle
                return handle
            park_handle, runtime = self._select_runtime(target)
            request = {
                "title": canonical_title,
                "operation": operation,
                "target": target,
                "args": list(args),
                "kwargs": kwargs or {},
            }
            try:
                encoded = _bounded_json_bytes(
                    request,
                    MAX_REQUEST_BYTES,
                    error_code="request_too_large",
                )
            except BackendCallError:
                raise
            except (TypeError, ValueError) as exc:
                raise BackendCallError("request must contain finite JSON values", code="invalid_request") from exc
            # Keep selection and the synchronous native call under one lock so
            # another thread cannot close this runtime between those steps.
            buffer = self._library.dbc_runtime_call(runtime, encoded, len(encoded))
            status = int(buffer.status)
            raw = self._consume(buffer)
            if len(raw) > MAX_RESPONSE_BYTES:
                raise BackendCallError("native response exceeds the response-size limit", code="response_too_large")
        try:
            response = json.loads(raw.decode("utf-8"), object_pairs_hook=_strict_object_pairs)
        except (UnicodeDecodeError, json.JSONDecodeError, ValueError, RecursionError) as exc:
            raise BackendCallError("native backend returned invalid JSON", details=raw[:200]) from exc
        if status not in {0, 1, 2}:
            raise BackendCallError("native backend returned an invalid ABI status", details=status)
        if not isinstance(response, dict) or type(response.get("ok")) is not bool:
            raise BackendCallError("native backend returned an invalid response envelope", details=raw[:200])
        if response.get("ok"):
            if status != 0:
                raise BackendCallError(
                    "native backend returned success with a failing ABI status",
                    code="invalid_backend_response",
                    details=status,
                )
            if set(response) != {"ok", "result"}:
                raise BackendCallError("native backend returned an invalid success envelope", details=raw[:200])
            result = response.get("result")
            park_number = park_handle.rsplit(":", 1)[1]
            if canonical_title in {"destiny.Ballpark.AddBall", "dbc.compat.Ballpark.GetBall"}:
                ball_id = _native_ball_handle_id(result)
                return f"ball:{park_number}:{ball_id}"
            if canonical_title == "dbc.compat.Ballpark.ListBalls":
                if not isinstance(result, list):
                    raise BackendCallError("native backend returned an invalid ball list", code="invalid_backend_response")
                return [f"ball:{park_number}:{_native_ball_handle_id(item)}" for item in result]
            if canonical_title == "destiny.Ballpark.GetBallIdsAndDistInRange":
                if not isinstance(result, list):
                    raise BackendCallError("native backend returned invalid range rows", code="invalid_backend_response")
                normalized = []
                for item in result:
                    if not isinstance(item, list) or len(item) != 2:
                        raise BackendCallError("native backend returned invalid range rows", code="invalid_backend_response")
                    distance = _finite_number(item[0], "native range distance")
                    ball_id = _integer(item[1], "native range ball id")
                    normalized.append((distance, ball_id))
                return normalized
            return result
        if set(response) != {"ok", "error"}:
            raise BackendCallError("native backend returned an invalid error envelope", details=raw[:200])
        error = response.get("error") or {}
        if not isinstance(error, dict):
            raise BackendCallError("native backend returned an invalid error envelope", details=raw[:200])
        code = error.get("code")
        message = error.get("message")
        if not isinstance(code, str) or not code or not isinstance(message, str):
            raise BackendCallError("native backend returned malformed error fields", details=raw[:200])
        if code == "unsupported_title":
            canonical = error.get("canonical_title", canonical_title)
            verdict = error.get("verdict", "UNKNOWN")
            difference = error.get("material_difference", "")
            mapping_items = error.get("mapping_items", ())
            if (
                not isinstance(canonical, str)
                or not isinstance(verdict, str)
                or not isinstance(difference, str)
                or not isinstance(mapping_items, list)
                or any(not isinstance(item, str) for item in mapping_items)
            ):
                raise BackendCallError("native backend returned malformed unsupported-title fields", details=raw[:200])
            raise UnsupportedTitleError(
                canonical,
                verdict,
                difference,
                tuple(mapping_items),
            )
        if code == "ball_not_found":
            raise BallNotFoundError(_integer(error.get("ball_id"), "native missing ball id"))
        raise BackendCallError(
            message,
            code=code,
            details=error,
        )

    def update(self, target: str | None = None) -> None:
        with self._lock:
            _, runtime = self._select_runtime(target)
            buffer = self._library.dbc_runtime_update(runtime)
            status = int(buffer.status)
            raw = self._consume(buffer)
        if len(raw) > MAX_RESPONSE_BYTES:
            raise BackendCallError("native response exceeds the response-size limit", code="response_too_large")
        if not raw:
            raise BackendCallError("native backend returned an empty update response", code="invalid_backend_response")
        try:
            response = json.loads(raw.decode("utf-8"), object_pairs_hook=_strict_object_pairs)
        except (UnicodeDecodeError, json.JSONDecodeError, ValueError, RecursionError) as exc:
            raise BackendCallError("native backend returned invalid update JSON", details=raw[:200]) from exc
        if status not in {0, 1, 2}:
            raise BackendCallError("native backend returned an invalid ABI status", details=status)
        if not isinstance(response, dict) or type(response.get("ok")) is not bool:
            raise BackendCallError("native backend returned an invalid update envelope", details=raw[:200])
        if response["ok"] and status != 0:
            raise BackendCallError(
                "native backend returned update success with a failing ABI status",
                code="invalid_backend_response",
                details=status,
            )
        expected_fields = {"ok", "result"} if response["ok"] else {"ok", "error"}
        if set(response) != expected_fields:
            raise BackendCallError("native backend returned an invalid update envelope", details=raw[:200])
        if not response["ok"]:
            error = response["error"]
            if (
                not isinstance(error, dict)
                or not isinstance(error.get("message"), str)
                or not isinstance(error.get("code"), str)
            ):
                raise BackendCallError("native backend returned malformed update error fields", details=raw[:200])
            raise BackendCallError(error["message"], code=error["code"], details=error)

    def release(self, target: str) -> None:
        with self._lock:
            if not isinstance(target, str) or not target.startswith("park:"):
                raise BackendCallError(f"invalid ballpark target {target!r}", code="invalid_target")
            runtime = self._runtimes.pop(target, None)
            if runtime is None:
                return
            if self._active_park == target:
                self._active_park = next(iter(self._runtimes), None)
            self._library.dbc_runtime_free(runtime)

    def close(self) -> None:
        lock = getattr(self, "_lock", None)
        if lock is None:
            return
        with lock:
            runtimes = getattr(self, "_runtimes", {})
            for runtime in tuple(runtimes.values()):
                self._library.dbc_runtime_free(runtime)
            runtimes.clear()
            self._active_park = None

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:  # noqa: BLE001 - destructors must never propagate
            pass


@dataclass
class _BallState:
    id: int
    mass: float
    radius: float
    max_velocity: float
    is_free: bool
    is_global: bool
    is_massive: bool
    is_interactive: bool
    is_space_junk: bool
    position: list[float]
    velocity: list[float]
    agility: float
    speed_fraction: float
    max_angular_velocity: float = F64_MAX
    angular_agility: float = 0.0
    angular_velocity: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0])
    rotation: list[float] = field(default_factory=lambda: [0.0, 0.0, 0.0, 1.0])
    is_cloaked: int = 0
    new_bubble_id: int = -1
    old_bubble_id: int = -1
    effect_stamp: int = 0
    massive_before_cloak: bool | None = None
    minis: list[dict[str, Any]] = field(default_factory=list)
    sensors: list[dict[str, Any]] = field(default_factory=list)


@dataclass
class _ParkState:
    is_master: bool = False
    running: bool = False
    tick_interval_ms: float = 1000.0
    friction: float = 1_000_000.0
    current_time: int = 0
    time: int = 0
    ego: int = 0
    balls: dict[int, _BallState] = field(default_factory=dict)
    proximity_events: list[dict[str, Any]] = field(default_factory=list)
    collision_substeps: int = 20
    use_iterative_collision: bool = False
    use_dynamical_orientation: bool = False
    disable_dynamical_orientation_for_missiles: bool = False
    use_new_orbit: bool = False
    pending_removals: dict[int, int] = field(default_factory=dict)
    network_outbox: dict[str, list[dict[str, Any]]] = field(
        default_factory=lambda: {"singlecasts": [], "narrowcasts": [], "batches": []}
    )
    network_outbox_bytes: int = 0
    network_last_batch_id: int = 0
    network_last_envelope: bytes | None = None
    visual_angular_velocity: dict[int, list[float]] = field(default_factory=dict)


def _normalize_quaternion(values: list[float]) -> list[float]:
    scale = max(abs(value) for value in values)
    if scale == 0.0 or not math.isfinite(scale):
        raise ValueError("rotation quaternion must be finite and non-zero")
    scaled = [value / scale for value in values]
    norm = math.dist(scaled, (0.0, 0.0, 0.0, 0.0))
    if norm == 0.0 or not math.isfinite(norm):
        raise ValueError("rotation quaternion must be finite and non-zero")
    stable_norm = scale * norm
    if math.isfinite(stable_norm) and abs(stable_norm - 1.0) <= sys.float_info.epsilon * 8.0:
        # Snapshots contain already-normalized authoritative quaternions.
        # Preserve their exact f64 components instead of adding one ulp of
        # drift on every capture/restore cycle.
        return list(values)
    return [value / norm for value in scaled]


def _normalized_vector(values: list[float]) -> list[float] | None:
    scale = max(abs(value) for value in values)
    if scale == 0.0:
        return None
    if not math.isfinite(scale):
        raise ValueError("vector must contain finite values")
    scaled = [value / scale for value in values]
    length = math.dist(scaled, (0.0, 0.0, 0.0))
    if length == 0.0 or not math.isfinite(length):
        return None
    return [value / length for value in scaled]


def _clamp_vector_magnitude(values: list[float], limit: float) -> list[float]:
    length = math.dist(values, (0.0, 0.0, 0.0))
    if length <= limit or length == 0.0:
        return list(values)
    direction = _normalized_vector(values)
    return [component * limit for component in direction] if direction is not None else [0.0, 0.0, 0.0]


def _quaternion_from_x_direction(vector: list[float]) -> list[float]:
    direction = _normalized_vector(vector)
    if direction is None:
        return [0.0, 0.0, 0.0, 1.0]
    x, y, z = direction
    if x <= -1.0 + 1e-12:
        return [0.0, 1.0, 0.0, 0.0]
    return _normalize_quaternion([0.0, -z, y, 1.0 + x])


def _euler_from_quaternion(q: list[float]) -> tuple[float, float, float]:
    x, y, z, w = _normalize_quaternion(q)
    sinr_cosp = 2.0 * (w * x + y * z)
    cosr_cosp = 1.0 - 2.0 * (x * x + y * y)
    roll = math.atan2(sinr_cosp, cosr_cosp)
    sinp = 2.0 * (w * y - z * x)
    pitch = math.copysign(math.pi / 2.0, sinp) if abs(sinp) >= 1.0 else math.asin(sinp)
    siny_cosp = 2.0 * (w * z + x * y)
    cosy_cosp = 1.0 - 2.0 * (y * y + z * z)
    return roll, pitch, math.atan2(siny_cosp, cosy_cosp)


def _rotate_vector(q: list[float], vector: list[float]) -> list[float]:
    x, y, z, w = q
    vx, vy, vz = vector
    tx, ty, tz = 2.0 * (y * vz - z * vy), 2.0 * (z * vx - x * vz), 2.0 * (x * vy - y * vx)
    return [
        vx + w * tx + (y * tz - z * ty),
        vy + w * ty + (z * tx - x * tz),
        vz + w * tz + (x * ty - y * tx),
    ]


def _closest_point_on_triangle(point, a, b, c):
    subtract = lambda left, right: [left[i] - right[i] for i in range(3)]
    dot = lambda left, right: sum(left[i] * right[i] for i in range(3))
    add_scaled = lambda origin, vector, scale: [origin[i] + vector[i] * scale for i in range(3)]
    raw_ab, raw_ac, raw_ap = subtract(b, a), subtract(c, a), subtract(point, a)
    scale = max(abs(value) for value in (*raw_ab, *raw_ac, *raw_ap))
    if scale == 0.0:
        return list(a)
    # Barycentric regions are invariant under uniform scaling. Keeping every
    # dot product near unity avoids the fourth-power overflow in the textbook
    # formula for otherwise valid 1e100-scale coordinates.
    ab = [value / scale for value in raw_ab]
    ac = [value / scale for value in raw_ac]
    ap = [value / scale for value in raw_ap]
    d1, d2 = dot(ab, ap), dot(ac, ap)
    if d1 <= 0.0 and d2 <= 0.0:
        return list(a)
    bp = [value / scale for value in subtract(point, b)]
    d3, d4 = dot(ab, bp), dot(ac, bp)
    if d3 >= 0.0 and d4 <= d3:
        return list(b)
    vc = d1 * d4 - d3 * d2
    if vc <= 0.0 and d1 >= 0.0 and d3 <= 0.0:
        return add_scaled(a, raw_ab, d1 / (d1 - d3))
    cp = [value / scale for value in subtract(point, c)]
    d5, d6 = dot(ab, cp), dot(ac, cp)
    if d6 >= 0.0 and d5 <= d6:
        return list(c)
    vb = d5 * d2 - d1 * d6
    if vb <= 0.0 and d2 >= 0.0 and d6 <= 0.0:
        return add_scaled(a, raw_ac, d2 / (d2 - d6))
    va = d3 * d6 - d5 * d4
    if va <= 0.0 and d4 - d3 >= 0.0 and d5 - d6 >= 0.0:
        bc = subtract(c, b)
        return add_scaled(b, bc, (d4 - d3) / ((d4 - d3) + (d5 - d6)))
    denominator = 1.0 / (va + vb + vc)
    v, w = vb * denominator, vc * denominator
    return [a[i] + raw_ab[i] * v + raw_ac[i] * w for i in range(3)]


def _finite_number(value: Any, name: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise TypeError(f"{name} must be a finite number")
    try:
        result = float(value)
    except OverflowError as exc:
        raise ValueError(f"{name} must be finite") from exc
    if not math.isfinite(result):
        raise ValueError(f"{name} must be finite")
    return result


def _finite_sum(left: float, right: float, name: str) -> float:
    result = left + right
    if not math.isfinite(result):
        raise OverflowError(f"{name} exceeds finite f64 range")
    return result


def _integer(value: Any, name: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise TypeError(f"{name} must be an integer")
    result = int(value)
    if result < I64_MIN or result > I64_MAX:
        raise OverflowError(f"{name} must fit a signed 64-bit integer")
    return result


def _native_ball_handle_id(value: Any) -> int:
    if not isinstance(value, str) or not value.startswith("ball:"):
        raise BackendCallError("native backend returned an invalid ball handle", code="invalid_backend_response")
    encoded = value[5:]
    try:
        parsed = int(encoded)
    except ValueError as exc:
        raise BackendCallError("native backend returned an invalid ball handle", code="invalid_backend_response") from exc
    if str(parsed) != encoded:
        raise BackendCallError("native backend returned a non-canonical ball handle", code="invalid_backend_response")
    try:
        return _integer(parsed, "native ball handle")
    except (TypeError, OverflowError) as exc:
        raise BackendCallError("native backend returned an out-of-range ball handle", code="invalid_backend_response") from exc


def _tick_interval(value: Any, name: str = "tickInterval") -> float:
    milliseconds = _finite_number(value, name)
    if milliseconds < 0.000001:
        raise ValueError(f"{name} must be at least one nanosecond")
    if milliseconds / 1000.0 > I64_MAX * 2.0:
        raise ValueError(f"{name} exceeds the native Duration range")
    return milliseconds


def _checked_i64_add(left: int, right: int, name: str) -> int:
    result = left + right
    if result < I64_MIN or result > I64_MAX:
        raise OverflowError(f"{name} overflows a signed 64-bit integer")
    return result


def _strict_object_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON field {key!r}")
        result[key] = value
    return result


def _bounded_json_bytes(
    value: Any,
    limit: int,
    *,
    error_code: str,
    sort_keys: bool = False,
) -> bytes:
    """Encode JSON incrementally and stop before exceeding ``limit`` bytes."""
    output = bytearray()
    encoder = json.JSONEncoder(
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
        sort_keys=sort_keys,
    )
    try:
        for text in encoder.iterencode(value):
            chunk = text.encode("utf-8")
            if len(output) + len(chunk) > limit:
                raise BackendCallError("encoded JSON exceeds the configured byte limit", code=error_code)
            output.extend(chunk)
    except RecursionError as exc:
        raise BackendCallError("JSON value exceeds the nesting limit", code=error_code) from exc
    return bytes(output)


def _deterministic_sensor_phase(ball_id: int, period: float) -> float:
    value = (ball_id & ((1 << 64) - 1)) + 0x9E3779B97F4A7C15
    value &= (1 << 64) - 1
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & ((1 << 64) - 1)
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & ((1 << 64) - 1)
    value ^= value >> 31
    return (value >> 11) * (1.0 / (1 << 53)) * period


def _boolean(value: Any, name: str) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        if value < I64_MIN or value > I64_MAX:
            raise OverflowError(f"{name} integer must fit a signed 64-bit value")
        return value != 0
    raise TypeError(f"{name} must be a boolean or integer")


def _strict_boolean(value: Any, name: str) -> bool:
    if not isinstance(value, bool):
        raise TypeError(f"{name} must be a boolean")
    return value


def _vector(values: Any, length: int, name: str) -> list[float]:
    if not isinstance(values, (list, tuple)) or len(values) != length:
        raise TypeError(f"{name} must contain exactly {length} values")
    return [_finite_number(value, f"{name}[{index}]") for index, value in enumerate(values)]


def _bounded_number(value: Any, name: str, maximum: float, *, minimum: float = 0.0) -> float:
    result = _finite_number(value, name)
    if result < minimum or abs(result) > maximum:
        raise ValueError(f"{name} is outside the solver-safe range [{minimum}, {maximum}]")
    return result


def _descriptor_bytes(rows: list[dict[str, Any]]) -> int:
    total = 0
    encoder = json.JSONEncoder(separators=(",", ":"), ensure_ascii=False, allow_nan=False)
    for text in encoder.iterencode(rows):
        total += len(text.encode("utf-8"))
        if total > MAX_CHILD_DESCRIPTOR_BYTES:
            break
    return total


def _validate_mini_descriptor(row: Any) -> None:
    if not isinstance(row, dict):
        raise TypeError("mini collider must be an object")
    kind = row.get("kind")
    if kind == "sphere":
        if set(row) != {"kind", "position", "radius"}:
            raise ValueError("mini sphere fields do not match the snapshot schema")
        position = _vector(row.get("position"), 3, "mini sphere position")
        radius = _finite_number(row.get("radius"), "mini sphere radius")
        if radius <= 0.0 or radius > MAX_RADIUS or any(abs(value) > MAX_COORDINATE for value in position):
            raise ValueError("mini sphere radius must be positive")
        return
    if kind == "capsule":
        if set(row) != {"kind", "a", "b", "radius"}:
            raise ValueError("mini capsule fields do not match the snapshot schema")
        a = _vector(row.get("a"), 3, "mini capsule endpoint a")
        b = _vector(row.get("b"), 3, "mini capsule endpoint b")
        distance = math.dist(a, b)
        radius = _finite_number(row.get("radius"), "mini capsule radius")
        if (
            not math.isfinite(distance)
            or distance == 0.0
            or distance > MAX_COORDINATE
            or radius <= 0.0
            or radius > MAX_RADIUS
            or any(abs(value) > MAX_COORDINATE for value in (*a, *b))
        ):
            raise ValueError("mini capsule needs distinct endpoints and a positive radius")
        return
    if kind == "box":
        if set(row) != {"kind", "basis"}:
            raise ValueError("mini box fields do not match the snapshot schema")
        basis = _vector(row.get("basis"), 12, "mini box basis")
        axes = (basis[3:6], basis[6:9], basis[9:12])
        lengths = [math.dist(axis, (0.0, 0.0, 0.0)) for axis in axes]
        if any(not math.isfinite(length) or length <= 0.0 for length in lengths):
            raise ValueError("mini box axes must be finite and non-zero")
        normalized = [[component / lengths[index] for component in axes[index]] for index in range(3)]
        for left, right in ((0, 1), (0, 2), (1, 2)):
            if abs(sum(normalized[left][index] * normalized[right][index] for index in range(3))) > 1e-6:
                raise ValueError("mini box axes must be mutually orthogonal")
        determinant = (
            normalized[0][0] * (normalized[1][1] * normalized[2][2] - normalized[1][2] * normalized[2][1])
            - normalized[0][1] * (normalized[1][0] * normalized[2][2] - normalized[1][2] * normalized[2][0])
            + normalized[0][2] * (normalized[1][0] * normalized[2][1] - normalized[1][1] * normalized[2][0])
        )
        if determinant <= 0.0:
            raise ValueError("mini box basis must be right-handed")
        center = [basis[index] + 0.5 * sum(axis[index] for axis in axes) for index in range(3)]
        if not all(math.isfinite(value) for value in center):
            raise ValueError("mini box center overflowed")
        if any(abs(value) > MAX_COORDINATE for value in basis) or any(length > MAX_RADIUS for length in lengths):
            raise ValueError("mini box exceeds solver-safe shape bounds")
        return
    raise ValueError(f"unsupported mini collider kind {kind!r}")


def _validate_sensor_descriptor(row: Any) -> None:
    if not isinstance(row, dict):
        raise TypeError("sensor must be an object")
    required_fields = {
        "range",
        "period",
        "shuffle",
        "only_interactives",
        "elapsed",
        "members",
        "cloak_sensor",
    }
    if set(row) != required_fields:
        raise ValueError("sensor fields do not match the snapshot schema")
    range_value = _finite_number(row["range"], "sensor range")
    if abs(range_value) > MAX_COORDINATE:
        raise ValueError("sensor range exceeds the solver-safe limit")
    period = _finite_number(row["period"], "sensor period")
    if period <= 0.0:
        raise ValueError("sensor period must be positive")
    _integer(row["shuffle"], "sensor shuffle")
    if not isinstance(row["only_interactives"], bool):
        raise TypeError("sensor only_interactives must be boolean")
    if not isinstance(row["cloak_sensor"], bool):
        raise TypeError("sensor cloak_sensor must be boolean")
    elapsed = _finite_number(row["elapsed"], "sensor elapsed")
    if elapsed < 0.0 or elapsed >= period:
        raise ValueError("sensor elapsed must be in [0, period)")
    members = row["members"]
    if not isinstance(members, list):
        raise TypeError("sensor members must be an integer list")
    normalized = [_integer(member, "sensor member") for member in members]
    if len(normalized) != len(set(normalized)):
        raise ValueError("sensor members must not contain duplicates")


def _validate_ball_state(ball: _BallState, *, normalize_api_values: bool = False) -> _BallState:
    ball.id = _integer(ball.id, "id")
    ball.mass = _finite_number(ball.mass, "mass")
    ball.radius = _finite_number(ball.radius, "radius")
    ball.max_velocity = _finite_number(ball.max_velocity, "max_velocity")
    ball.max_angular_velocity = _finite_number(ball.max_angular_velocity, "max_angular_velocity")
    ball.agility = _finite_number(ball.agility, "agility")
    ball.angular_agility = _finite_number(ball.angular_agility, "angular_agility")
    ball.speed_fraction = _finite_number(ball.speed_fraction, "speed_fraction")
    if normalize_api_values:
        ball.mass = max(0.0, ball.mass)
        ball.radius = max(0.0, ball.radius)
        ball.max_velocity = max(0.0, ball.max_velocity)
        ball.max_angular_velocity = max(0.0, ball.max_angular_velocity)
        ball.agility = 1.0 if ball.agility <= 0.0 else ball.agility
        ball.angular_agility = max(0.0, ball.angular_agility)
        ball.speed_fraction = min(1.0, max(0.0, ball.speed_fraction))
    elif (
        ball.mass < 0.0
        or ball.radius < 0.0
        or ball.max_velocity < 0.0
        or ball.max_angular_velocity < 0.0
        or ball.agility <= 0.0
        or ball.angular_agility < 0.0
        or not 0.0 <= ball.speed_fraction <= 1.0
    ):
        raise ValueError("ball contains an out-of-range scalar")
    ball.position = _vector(ball.position, 3, "position")
    ball.velocity = _vector(ball.velocity, 3, "velocity")
    ball.angular_velocity = _vector(ball.angular_velocity, 3, "angular_velocity")
    ball.rotation = _normalize_quaternion(_vector(ball.rotation, 4, "rotation"))
    if any(abs(value) > MAX_COORDINATE for value in ball.position):
        raise ValueError("position exceeds the solver-safe coordinate limit")
    if any(abs(value) > MAX_VELOCITY for value in (*ball.velocity, *ball.angular_velocity)):
        raise ValueError("velocity exceeds the solver-safe magnitude limit")
    if ball.mass > MAX_MASS:
        raise ValueError("mass exceeds the solver-safe limit")
    if ball.radius > MAX_RADIUS:
        raise ValueError("radius exceeds the solver-safe limit")
    if ball.max_velocity > MAX_VELOCITY or ball.max_angular_velocity > MAX_VELOCITY:
        raise ValueError("speed limit exceeds the solver-safe limit")
    if ball.agility > MAX_AGILITY or ball.angular_agility > MAX_AGILITY:
        raise ValueError("agility exceeds the solver-safe limit")
    ball.is_cloaked = _integer(ball.is_cloaked, "is_cloaked")
    if ball.is_cloaked < 0 or ball.is_cloaked > 3:
        raise ValueError("is_cloaked must be in the range 0..3")
    ball.new_bubble_id = _integer(ball.new_bubble_id, "new_bubble_id")
    ball.old_bubble_id = _integer(ball.old_bubble_id, "old_bubble_id")
    ball.effect_stamp = _integer(ball.effect_stamp, "effect_stamp")
    for name in ("is_free", "is_global", "is_massive", "is_interactive", "is_space_junk"):
        if not isinstance(getattr(ball, name), bool):
            raise TypeError(f"{name} must be boolean")
    if ball.massive_before_cloak is not None and not isinstance(ball.massive_before_cloak, bool):
        raise TypeError("massive_before_cloak must be boolean or null")
    if not normalize_api_values and ball.is_cloaked and ball.is_massive:
        raise ValueError("a cloaked snapshot ball cannot be massive")
    if not normalize_api_values and ball.is_cloaked and ball.massive_before_cloak is None:
        raise ValueError("a cloaked snapshot ball must retain massive_before_cloak")
    if not normalize_api_values and not ball.is_cloaked and ball.massive_before_cloak is not None:
        raise ValueError("an uncloaked snapshot ball cannot retain massive_before_cloak")
    if not isinstance(ball.minis, list):
        raise TypeError("minis must be a list of objects")
    if not isinstance(ball.sensors, list):
        raise TypeError("sensors must be a list of objects")
    if len(ball.minis) + len(ball.sensors) > MAX_CHILD_SHAPES_PER_BALL:
        raise ValueError("per-ball child-shape limit exceeded")
    if len(ball.sensors) > 2:
        raise ValueError("a ball supports one user sensor and one cloak sensor")
    if _descriptor_bytes([*ball.minis, *ball.sensors]) > MAX_CHILD_DESCRIPTOR_BYTES:
        raise ValueError("per-ball child descriptor byte limit exceeded")
    for row in ball.minis:
        _validate_mini_descriptor(row)
    for row in ball.sensors:
        _validate_sensor_descriptor(row)
        sensor_reach = ball.radius + _finite_number(row["range"], "sensor range")
        if not math.isfinite(sensor_reach):
            raise OverflowError("sensor range plus ball radius overflowed")
        if sensor_reach < 0.0:
            raise ValueError("sensor range ends inside the ball center")
    cloak_sensors = sum(bool(row.get("cloak_sensor")) for row in ball.sensors)
    user_sensors = len(ball.sensors) - cloak_sensors
    if cloak_sensors > 1 or user_sensors > 1:
        raise ValueError("a ball supports at most one sensor of each kind")
    return ball


def _same_visibility_partition(source: _BallState, candidate: _BallState) -> bool:
    return candidate.is_global or source.new_bubble_id == candidate.new_bubble_id


def _point_segment_distance(point: list[float], start: list[float], end: list[float]) -> tuple[float, float]:
    segment = [end[index] - start[index] for index in range(3)]
    if not all(math.isfinite(value) for value in segment):
        return math.inf, 0.0
    segment_length = math.dist(segment, (0.0, 0.0, 0.0))
    if not math.isfinite(segment_length):
        return math.inf, 0.0
    if segment_length == 0.0:
        return math.dist(point, start), 0.0
    direction = [value / segment_length for value in segment]
    offset = [point[index] - start[index] for index in range(3)]
    if not all(math.isfinite(value) for value in offset):
        return math.inf, 0.0
    projection = sum(offset[index] * direction[index] for index in range(3))
    if not math.isfinite(projection):
        return math.inf, 0.0
    distance_along = max(0.0, min(segment_length, projection))
    parameter = distance_along / segment_length
    closest = [start[index] + distance_along * direction[index] for index in range(3)]
    return math.dist(point, closest), parameter


def _sphere_intersects_destiny_cone(
    offset: list[float],
    radius: float,
    direction: list[float],
    height: float,
    angle: float,
) -> bool:
    if angle == 0.0:
        endpoint = [direction[index] * height for index in range(3)]
        distance, _ = _point_segment_distance(offset, [0.0, 0.0, 0.0], endpoint)
        return distance <= radius
    distance = math.dist(offset, (0.0, 0.0, 0.0))
    maximum = height + radius
    if not math.isfinite(distance) or distance > maximum:
        return False
    sine = math.sin(angle)
    cosine = math.cos(angle)
    axial = sum(direction[index] * offset[index] for index in range(3))
    if not math.isfinite(axial):
        return False
    radial = math.dist(
        offset,
        [direction[index] * axial for index in range(3)],
    )
    # This is algebraically equivalent to the shifted-apex Destiny test, but
    # it never constructs radius / sin(angle). That quotient overflows for
    # valid subnormal angles and used to reject spheres directly on the axis.
    expanded_axial = axial * sine + radius
    if (
        not math.isfinite(radial)
        or not math.isfinite(expanded_axial)
        or expanded_axial <= 0.0
        or expanded_axial <= abs(cosine) * radial
    ):
        return False
    reverse_projection = -axial
    if (
        reverse_projection > 0.0
        and reverse_projection >= distance * abs(sine)
        and distance > radius
    ):
        return False
    return True


class InMemoryBackend:
    """Behavioral contract backend; explicit opt-in and never presented as Bevy."""

    def __init__(self) -> None:
        self._parks: dict[str, _ParkState] = {}
        self._active_park: str | None = None
        self._next_park = 0
        self._lock = RLock()

    def close(self) -> None:
        with self._lock:
            self._parks.clear()
            self._active_park = None

    def release(self, target: str) -> None:
        with self._lock:
            if not isinstance(target, str) or not target.startswith("park:"):
                raise BackendCallError(f"invalid ballpark target {target!r}", code="invalid_target")
            self._parks.pop(target, None)
            if self._active_park == target:
                self._active_park = next(iter(self._parks), None)

    def update(self, target: str | None = None) -> None:
        with self._lock:
            self._activate(target)
            if self._park.running:
                self._evolve_one_tick()

    @property
    def _park(self) -> _ParkState:
        if self._active_park is None:
            raise BackendCallError("no ballpark has been constructed", code="invalid_target")
        return self._parks[self._active_park]

    def _activate(self, target: str | None) -> None:
        handle = None
        if target and target.startswith("park:"):
            handle = target
        elif target and target.startswith("ball:"):
            parts = target.split(":")
            if len(parts) == 3 and parts[1].isdigit():
                handle = f"park:{parts[1]}"
        elif len(self._parks) == 1:
            handle = next(iter(self._parks))
        elif target is None and self._active_park is not None:
            handle = self._active_park
        if handle not in self._parks:
            raise BackendCallError(f"invalid ballpark target {target!r}", code="invalid_target")
        self._active_park = handle

    def _ball_id(self, target: str | None) -> int:
        if not target or not target.startswith("ball:"):
            raise BackendCallError("ball operation requires a ball target", code="invalid_target")
        parts = target.split(":")
        if (
            len(parts) != 3
            or not parts[1].isdigit()
            or f"park:{parts[1]}" != self._active_park
        ):
            raise BackendCallError(f"invalid ball target {target!r}", code="invalid_target")
        try:
            return _integer(int(parts[2]), "ball_id")
        except (ValueError, TypeError, OverflowError) as exc:
            raise BackendCallError(f"invalid ball target {target!r}", code="invalid_target") from exc

    def _ball(self, ball_id: int) -> _BallState:
        try:
            return self._park.balls[_integer(ball_id, "ball_id")]
        except KeyError as exc:
            raise BallNotFoundError(ball_id) from exc

    def _ensure_live_descriptor_budget(
        self,
        ball_id: int,
        minis: list[dict[str, Any]],
        sensors: list[dict[str, Any]],
    ) -> None:
        total = 0
        for candidate_id, candidate in self._park.balls.items():
            rows = [*minis, *sensors] if candidate_id == ball_id else [*candidate.minis, *candidate.sensors]
            total += _descriptor_bytes(rows)
            if total > MAX_CHILD_DESCRIPTOR_BYTES:
                raise BackendCallError("live child descriptor byte limit exceeded", code="limit_exceeded")

    def _cloak_ball_state(
        self,
        ball: _BallState,
        cloak_mode: int,
        uncloak_range: float | None = None,
    ) -> None:
        if cloak_mode not in {1, 2, 3}:
            raise ValueError("cloak_mode must be 1, 2, or 3")
        # Stage every part of the transition before touching live state.
        retained_sensors = [sensor for sensor in ball.sensors if not sensor.get("cloak_sensor", False)]
        if self._park.is_master and cloak_mode == 1:
            range_value = 2000.0 if uncloak_range is None else _finite_number(uncloak_range, "uncloak_range")
            if range_value <= 0.0:
                raise ValueError("uncloak_range must be positive")
            if not math.isfinite(ball.radius + range_value):
                raise OverflowError("uncloak range plus ball radius overflowed")
            if len(ball.minis) + len(retained_sensors) >= MAX_CHILD_SHAPES_PER_BALL:
                raise BackendCallError("per-ball sensor limit exceeded", code="limit_exceeded")
            replacement_sensor = {
                "range": range_value,
                "period": 2.0,
                "shuffle": 0,
                "only_interactives": False,
                "elapsed": 0.0,
                "members": [],
                "cloak_sensor": True,
            }
        else:
            replacement_sensor = None
        staged_sensors = [*retained_sensors]
        if replacement_sensor is not None:
            _validate_sensor_descriptor(replacement_sensor)
            staged_sensors.append(replacement_sensor)
        self._ensure_live_descriptor_budget(ball.id, ball.minis, staged_sensors)
        if ball.is_cloaked == 0:
            ball.massive_before_cloak = ball.is_massive
        ball.is_cloaked = cloak_mode
        ball.is_massive = False
        ball.sensors = staged_sensors

    def _uncloak_ball_state(self, ball: _BallState) -> None:
        if ball.is_cloaked == 0:
            ball.sensors = [sensor for sensor in ball.sensors if not sensor.get("cloak_sensor", False)]
            return
        ball.is_cloaked = 0
        ball.sensors = [sensor for sensor in ball.sensors if not sensor.get("cloak_sensor", False)]
        # Warp controller state is explicitly unsupported, so every supported
        # uncloaking transition is the original non-warping massive transition.
        ball.is_massive = bool(ball.massive_before_cloak)
        ball.massive_before_cloak = None
        for owner in self._park.balls.values():
            for sensor in owner.sensors:
                sensor["members"] = [member for member in sensor.get("members", []) if member != ball.id]

    def invoke(
        self,
        canonical_title: str,
        operation: str = "call",
        target: str | None = None,
        args: tuple[Any, ...] | list[Any] = (),
        kwargs: dict[str, Any] | None = None,
    ) -> Any:
        if operation not in {"call", "construct", "get", "set"}:
            raise BackendCallError(f"invalid operation {operation!r}", code="invalid_request")
        if kwargs:
            raise BackendCallError("keyword arguments are not supported by the Destiny ABI", code="invalid_request")
        with self._lock:
            if canonical_title == "destiny.Ballpark.__init__":
                if operation != "construct" or target is not None or len(args) > 1:
                    raise BackendCallError(
                        "Ballpark construction requires operation=construct, a null target, and at most isMaster",
                        code="invalid_request",
                    )
                handle = f"park:{self._next_park}"
                self._next_park += 1
                from ._settings import settings

                configuration = settings.Get()
                if configuration.useNewOrbit:
                    raise BackendCallError(
                        "useNewOrbit is unsupported because Orbit is not implemented by this compatibility subset",
                        code="unsupported_setting",
                    )
                if configuration.disableDynamicalOrientationForMissiles:
                    raise BackendCallError(
                        "disableDynamicalOrientationForMissiles requires missile classification that this subset does not expose",
                        code="unsupported_setting",
                    )
                if configuration.useDynamicalOrientation:
                    raise BackendCallError(
                        "useDynamicalOrientation is unsupported until the Destiny angular controller state is implemented",
                        code="unsupported_setting",
                    )
                collision_substeps = _integer(configuration.collisionMaxIterations, "collisionMaxIterations")
                if not 1 <= collision_substeps <= MAX_COLLISION_SUBSTEPS:
                    raise BackendCallError(
                        f"collisionMaxIterations must be in 1..{MAX_COLLISION_SUBSTEPS}",
                        code="unsupported_setting",
                    )
                self._parks[handle] = _ParkState(
                    is_master=_boolean(args[0], "isMaster") if args else False,
                    collision_substeps=collision_substeps,
                    use_iterative_collision=configuration.useIterativeCollision,
                    use_dynamical_orientation=configuration.useDynamicalOrientation,
                    disable_dynamical_orientation_for_missiles=configuration.disableDynamicalOrientationForMissiles,
                    use_new_orbit=configuration.useNewOrbit,
                )
                self._active_park = handle
                return handle
            park_scoped = canonical_title.startswith(("destiny.Ballpark.", "dbc.compat.")) or canonical_title in {
                "destiny.net.server.NetworkInterface.singlecast",
                "destiny.net.server.NetworkInterface.narrowcast",
                "destiny.net.server.NetworkInterface.batch",
            }
            if park_scoped and (
                not isinstance(target, str)
                or target.count(":") != 1
                or not target.startswith("park:")
                or not target[5:].isdigit()
            ):
                raise BackendCallError("ballpark operation requires a park target", code="invalid_target")
            self._activate(target)
            if canonical_title.startswith("dbc.compat."):
                return self._invoke_compat(canonical_title, operation, args)
            if canonical_title in {
                "destiny.net.server.NetworkInterface.singlecast",
                "destiny.net.server.NetworkInterface.narrowcast",
                "destiny.net.server.NetworkInterface.batch",
            }:
                self._require_operation(operation, "call", canonical_title)
                self._require_arity(args, 1, canonical_title)
                envelope = args[0]
                expected_mode = canonical_title.rsplit(".", 1)[1]
                if not isinstance(envelope, dict):
                    raise BackendCallError("network message must be an object", code="invalid_protocol")
                if set(envelope) != {"protocol", "schema_version", "mode", "batch_id", "updates"}:
                    raise BackendCallError("Carbon network envelope fields are invalid", code="invalid_protocol")
                if (
                    envelope.get("protocol") != "destiny-carbon-update"
                    or type(envelope.get("schema_version")) is not int
                    or envelope.get("schema_version") != 2
                ):
                    raise BackendCallError("unsupported Carbon network protocol", code="invalid_protocol")
                batch_id = envelope.get("batch_id")
                if type(batch_id) is not int or not 1 <= batch_id <= I64_MAX:
                    raise BackendCallError("invalid Carbon batch identifier", code="invalid_protocol")
                updates = envelope.get("updates")
                valid_updates = (
                    isinstance(updates, dict)
                    and set(updates) == {"singlecasts", "narrowcasts"}
                    and all(isinstance(rows, list) for rows in updates.values())
                    if expected_mode == "batch"
                    else isinstance(updates, list)
                )
                if envelope.get("mode") != expected_mode or not valid_updates:
                    raise BackendCallError("invalid Carbon network envelope", code="invalid_protocol")
                try:
                    from destiny.net._codec import decode_carbon_value

                    decoded_updates = decode_carbon_value(updates)
                    sections = (
                        (
                            (decoded_updates["singlecasts"], False),
                            (decoded_updates["narrowcasts"], True),
                        )
                        if expected_mode == "batch"
                        else ((decoded_updates, expected_mode == "narrowcast"),)
                    )
                    expanded_rows = 0
                    for rows, narrowcast in sections:
                        if not isinstance(rows, (list, tuple)) or len(rows) > 100_000:
                            raise ValueError("Carbon update row limit exceeded")
                        for row in rows:
                            if not isinstance(row, (list, tuple)) or len(row) < 3:
                                raise ValueError("Carbon update row is malformed")
                            recipients = row[0] if narrowcast else (row[0],)
                            if narrowcast and not isinstance(recipients, (list, tuple)):
                                raise TypeError("Carbon narrowcast recipients must be a sequence")
                            if len(recipients) > 100_000 or any(
                                type(recipient) is not int or not I64_MIN <= recipient <= I64_MAX
                                for recipient in recipients
                            ):
                                raise ValueError("Carbon recipient list is invalid or oversized")
                            expanded_rows += len(recipients)
                            if expanded_rows > MAX_EXPANDED_NETWORK_ROWS:
                                raise ValueError("expanded Carbon update count exceeds the compatibility limit")
                            if not isinstance(row[1], str) or not row[1]:
                                raise TypeError("Carbon update action must be a non-empty string")
                            if not isinstance(row[2], (list, tuple)):
                                raise TypeError("Carbon update state must be a sequence")
                except (TypeError, ValueError, OverflowError, RecursionError) as exc:
                    raise BackendCallError(
                        f"invalid Carbon network update rows: {exc}",
                        code="invalid_protocol",
                    ) from exc
                try:
                    encoded_envelope = _bounded_json_bytes(
                        envelope,
                        MAX_OUTBOX_BYTES,
                        error_code="backpressure",
                        # JSON objects are unordered. Use one stable
                        # representation so an otherwise exact retry cannot
                        # conflict solely because a nested mapping was built
                        # with a different insertion order.
                        sort_keys=True,
                    )
                except (TypeError, ValueError) as exc:
                    raise BackendCallError("network envelope must contain finite JSON values", code="invalid_protocol") from exc
                if batch_id == self._park.network_last_batch_id:
                    if encoded_envelope == self._park.network_last_envelope:
                        return None
                    raise BackendCallError(
                        "Carbon batch identifier conflicts with a different prior envelope",
                        code="invalid_protocol",
                    )
                if batch_id < self._park.network_last_batch_id:
                    raise BackendCallError(
                        "Carbon batch identifiers must increase monotonically per runtime",
                        code="invalid_protocol",
                    )
                queue_name = {
                    "singlecast": "singlecasts",
                    "narrowcast": "narrowcasts",
                    "batch": "batches",
                }[expected_mode]
                if sum(len(queue) for queue in self._park.network_outbox.values()) >= MAX_OUTBOX_MESSAGES:
                    raise BackendCallError("network outbox limit exceeded", code="backpressure")
                if (
                    len(encoded_envelope) > MAX_OUTBOX_BYTES
                    or self._park.network_outbox_bytes + len(encoded_envelope) > MAX_OUTBOX_BYTES
                ):
                    raise BackendCallError("network outbox byte limit exceeded", code="backpressure")
                self._park.network_outbox[queue_name].append(json.loads(encoded_envelope))
                self._park.network_outbox_bytes += len(encoded_envelope)
                self._park.network_last_batch_id = batch_id
                self._park.network_last_envelope = encoded_envelope
                return None
            if canonical_title.startswith(("destiny.Ball.", "destiny.ClientBall.")):
                return self._invoke_ball(canonical_title, operation, target, args)
            if canonical_title.startswith("destiny.Ballpark."):
                return self._invoke_park(canonical_title, operation, args)
            raise UnsupportedTitleError.for_title(canonical_title)

    @staticmethod
    def _require_operation(actual: str, expected: str, title: str) -> None:
        if actual != expected:
            raise BackendCallError(
                f"{title} requires operation={expected!r}, got {actual!r}",
                code="invalid_request",
            )

    @staticmethod
    def _require_arity(args: tuple[Any, ...] | list[Any], allowed: int | set[int], title: str) -> None:
        allowed_set = {allowed} if isinstance(allowed, int) else allowed
        if len(args) not in allowed_set:
            expected = ", ".join(str(value) for value in sorted(allowed_set))
            raise TypeError(f"{title} expects {expected} argument(s), got {len(args)}")

    def _invoke_compat(self, title: str, operation: str, args: tuple[Any, ...] | list[Any]) -> Any:
        self._require_operation(operation, "call", title)
        if title == "dbc.compat.Ballpark.HasBall":
            self._require_arity(args, 1, title)
            return _integer(args[0], "ball_id") in self._park.balls
        if title == "dbc.compat.Ballpark.ListBalls":
            self._require_arity(args, 0, title)
            park_number = self._active_park.rsplit(":", 1)[1]
            return [f"ball:{park_number}:{ball_id}" for ball_id in sorted(self._park.balls)]
        if title == "dbc.compat.Ballpark.GetBall":
            self._require_arity(args, 1, title)
            ball_id = _integer(args[0], "ball_id")
            self._ball(ball_id)
            park_number = self._active_park.rsplit(":", 1)[1]
            return f"ball:{park_number}:{ball_id}"
        if title == "dbc.compat.Ballpark.BubbleMembership":
            self._require_arity(args, 0, title)
            eligible = {
                ball_id: ball
                for ball_id, ball in self._park.balls.items()
                if ball_id not in self._park.pending_removals
                and ball.new_bubble_id >= 0
                and ball.is_cloaked == 0
            }
            interactives: dict[int, list[int]] = defaultdict(list)
            for ball_id, ball in eligible.items():
                if ball.is_interactive:
                    interactives[ball.new_bubble_id].append(ball_id)
            bubbles = set(interactives)
            bubbles.update(ball.new_bubble_id for ball in eligible.values() if not ball.is_global)
            global_ids = {ball_id for ball_id, ball in eligible.items() if ball.is_global}
            members = {
                bubble_id: sorted(
                    global_ids
                    | {
                        ball_id
                        for ball_id, ball in eligible.items()
                        if not ball.is_global and ball.new_bubble_id == bubble_id
                    }
                )
                for bubble_id in sorted(bubbles)
            }
            observers = {
                ball_id: list(members.get(ball.new_bubble_id, ()))
                for ball_id, ball in eligible.items()
                if ball.is_interactive
            }
            return {
                "interactives": {str(key): sorted(value) for key, value in interactives.items()},
                "members": {str(key): value for key, value in members.items()},
                "observers": {str(key): value for key, value in observers.items()},
            }
        if title == "dbc.compat.Ballpark.CaptureSnapshot":
            self._require_arity(args, {0, 1}, title)
            source_id = _integer(args[0], "source_id") if args and args[0] not in (None, -1) else -1
            encoded = self._invoke_compat(
                "dbc.compat.Ballpark.Serialize",
                "call",
                (None, source_id),
            )
            return {"current_time": self._park.current_time, "snapshot": encoded}
        if title == "dbc.compat.Ballpark.Serialize":
            self._require_arity(args, {1, 2}, title)
            if args[0] is not None:
                if not isinstance(args[0], (list, tuple)):
                    raise TypeError("snapshot ball selector must be a list or tuple")
                if len(args[0]) > MAX_SNAPSHOT_BALLS:
                    raise BackendCallError(
                        "snapshot selector exceeds the ball-count limit",
                        code="snapshot_too_large",
                    )
                ids = sorted({_integer(value, "ball_id") for value in args[0]})
            else:
                ids = sorted(self._park.balls)
            source = None
            if len(args) == 2 and args[1] not in (None, -1):
                source_id = _integer(args[1], "source_id")
                if source_id in self._park.pending_removals:
                    raise BackendCallError("snapshot source is pending removal", code="invalid_request")
                source = self._ball(source_id)
            selected: list[_BallState] = []
            for ball_id in ids:
                ball = self._park.balls.get(ball_id)
                if ball is None:
                    continue
                if source is not None:
                    if ball_id in self._park.pending_removals:
                        continue
                    if not _same_visibility_partition(source, ball):
                        continue
                    if ball.is_cloaked and ball.id != source.id:
                        continue
                selected.append(ball)
            if len(selected) > MAX_SNAPSHOT_BALLS:
                raise BackendCallError("snapshot ball-count limit exceeded", code="snapshot_too_large")
            selected_ids = {ball.id for ball in selected}
            live_ids = set(self._park.balls)
            ball_rows = []
            for ball in selected:
                try:
                    validated = _validate_ball_state(copy.deepcopy(ball))
                    for sensor in validated.sensors:
                        for member in sensor["members"]:
                            if member == validated.id or member not in live_ids:
                                raise ValueError(
                                    f"sensor member {member} is self-referential or missing from the live park"
                                )
                except (TypeError, ValueError, OverflowError, RecursionError) as exc:
                    raise BackendCallError(
                        f"cannot emit invalid live ball {ball.id}: {exc}",
                        code="invalid_state",
                    ) from exc
                row = asdict(validated)
                for sensor in row["sensors"]:
                    sensor["members"] = sorted(
                        member
                        for member in sensor["members"]
                        if member in selected_ids and member != ball.id
                    )
                ball_rows.append(row)
            pending_removals = [
                {"ball_id": ball_id, "due_tick": due_tick, "reason": "delayed"}
                for ball_id, due_tick in sorted(self._park.pending_removals.items())
                if ball_id in selected_ids
            ]
            data = {
                "format": "destiny-bevy-compat-state-v3",
                "schema_version": 3,
                "park": {
                    "is_master": self._park.is_master,
                    "running": self._park.running,
                    "tick_interval_ms": self._park.tick_interval_ms,
                    "friction": self._park.friction,
                    "current_time": self._park.current_time,
                    "time": self._park.time,
                    # A filtered/subset snapshot must be independently
                    # restorable. Do not retain an ego reference to a ball the
                    # same snapshot intentionally omitted.
                    "ego": self._park.ego if self._park.ego in selected_ids else 0,
                    "collision_substeps": self._park.collision_substeps,
                    "use_iterative_collision": self._park.use_iterative_collision,
                    "use_dynamical_orientation": self._park.use_dynamical_orientation,
                    "disable_dynamical_orientation_for_missiles": self._park.disable_dynamical_orientation_for_missiles,
                    "use_new_orbit": self._park.use_new_orbit,
                    "pending_removals": pending_removals,
                    "snapshot_semantics": "logical-authoritative",
                },
                "balls": ball_rows,
            }
            raw = _bounded_json_bytes(data, MAX_SNAPSHOT_BYTES, error_code="snapshot_too_large")
            return base64.b64encode(raw).decode("ascii")
        if title == "dbc.compat.Ballpark.Deserialize":
            self._require_arity(args, {1, 2}, title)
            partial = _integer(args[1], "partial") if len(args) > 1 else 0
            if partial not in {0, 1, 2}:
                raise ValueError("partial must be one of 0, 1, or 2")
            if not isinstance(args[0], str):
                raise TypeError("snapshot must be a base64 string")
            try:
                encoded = args[0].encode("ascii")
                if len(encoded) > MAX_SNAPSHOT_ENCODED_BYTES:
                    raise BackendCallError("encoded snapshot exceeds the configured byte limit", code="snapshot_too_large")
                raw = base64.b64decode(encoded, validate=True)
                if base64.b64encode(raw) != encoded:
                    raise ValueError("snapshot base64 is not canonical")
                if len(raw) > MAX_SNAPSHOT_BYTES:
                    raise BackendCallError("snapshot exceeds the configured byte limit", code="snapshot_too_large")
                payload = json.loads(raw.decode("utf-8"), object_pairs_hook=_strict_object_pairs)
            except BackendCallError:
                raise
            except (UnicodeError, ValueError, binascii.Error, json.JSONDecodeError, RecursionError) as exc:
                raise BackendCallError("snapshot is not valid base64-encoded UTF-8 JSON", code="invalid_snapshot") from exc
            if not isinstance(payload, dict):
                raise BackendCallError("snapshot root must be an object", code="invalid_snapshot")
            if set(payload) != {"format", "schema_version", "park", "balls"}:
                raise BackendCallError("snapshot root fields do not match schema v3", code="invalid_snapshot")
            format_name = payload.get("format")
            if (
                format_name != "destiny-bevy-compat-state-v3"
                or type(payload.get("schema_version")) is not int
                or payload.get("schema_version") != 3
            ):
                raise BackendCallError(f"unsupported snapshot format {format_name!r}", code="invalid_snapshot")
            rows = payload.get("balls")
            if not isinstance(rows, list):
                raise BackendCallError("snapshot balls must be an array", code="invalid_snapshot")
            if len(rows) > MAX_SNAPSHOT_BALLS:
                raise BackendCallError("snapshot ball-count limit exceeded", code="snapshot_too_large")

            # Parse and validate every row before mutating live state.  This is
            # the transaction boundary: any error leaves the park untouched.
            parsed: dict[int, _BallState] = {}
            try:
                for row in rows:
                    if not isinstance(row, dict):
                        raise TypeError("every snapshot ball must be an object")
                    if set(row) != set(_BallState.__dataclass_fields__):
                        raise ValueError("snapshot ball fields do not match schema v3")
                    ball = _validate_ball_state(_BallState(**row))
                    if ball.id in parsed:
                        raise ValueError(f"duplicate ball id {ball.id}")
                    parsed[ball.id] = ball
                parsed_ids = set(parsed)
                for ball in parsed.values():
                    for sensor in ball.sensors:
                        for member in sensor["members"]:
                            if member == ball.id or member not in parsed_ids:
                                raise ValueError(
                                    f"sensor member {member} is self-referential or missing from the snapshot"
                                )
            except (TypeError, ValueError, OverflowError) as exc:
                raise BackendCallError(f"invalid snapshot ball: {exc}", code="invalid_snapshot") from exc

            if partial == 1:
                for ball_id, incoming in parsed.items():
                    existing = self._park.balls.get(ball_id)
                    if existing is not None:
                        incoming.minis = copy.deepcopy(existing.minis)
                        # User sensors are host/controller state in rollback
                        # mode, while cloak sensors are derived from the
                        # authoritative cloak transition in the checkpoint.
                        preserved_user = copy.deepcopy([
                            sensor
                            for sensor in existing.sensors
                            if not sensor.get("cloak_sensor", False)
                        ])
                        incoming_cloak = copy.deepcopy([
                            sensor
                            for sensor in incoming.sensors
                            if sensor.get("cloak_sensor", False)
                        ])
                        incoming.sensors = [*preserved_user, *incoming_cloak]
                    else:
                        # Legacy partial mode 1 consumes child-shape bytes but
                        # does not attach them to newly created balls.
                        incoming.minis = []
                        incoming.sensors = []
                # Rollback mode restores the authoritative entity set. Future
                # entities absent from the checkpoint must not survive.
                committed = parsed
                committed_ids = set(committed)
                for committed_ball in committed.values():
                    for sensor in committed_ball.sensors:
                        sensor["members"] = [
                            member
                            for member in sensor["members"]
                            if member != committed_ball.id and member in committed_ids
                        ]
            elif partial == 2:
                committed = dict(self._park.balls)
                committed.update(parsed)
            else:
                committed = parsed

            if len(committed) > MAX_LIVE_BALLS:
                raise BackendCallError("restored live ball limit exceeded", code="snapshot_too_large")
            descriptor_bytes = 0
            for committed_ball in committed.values():
                descriptor_bytes += _descriptor_bytes([*committed_ball.minis, *committed_ball.sensors])
                if descriptor_bytes > MAX_CHILD_DESCRIPTOR_BYTES:
                    raise BackendCallError(
                        "restored live child descriptor byte limit exceeded",
                        code="snapshot_too_large",
                    )

            park_row = payload.get("park")
            expected_park_fields = {
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
            if not isinstance(park_row, dict) or set(park_row) != expected_park_fields:
                raise BackendCallError("snapshot park fields do not match schema v3", code="invalid_snapshot")
            try:
                staged_park = {
                    "is_master": _strict_boolean(park_row["is_master"], "is_master"),
                    "running": _strict_boolean(park_row["running"], "running"),
                    "tick_interval_ms": _tick_interval(park_row["tick_interval_ms"], "tick_interval_ms"),
                    "friction": _finite_number(park_row["friction"], "friction"),
                    "current_time": _integer(park_row["current_time"], "current_time"),
                    "time": _integer(park_row["time"], "time"),
                    "ego": _integer(park_row["ego"], "ego"),
                    "collision_substeps": _integer(park_row["collision_substeps"], "collision_substeps"),
                    "use_iterative_collision": _strict_boolean(
                        park_row["use_iterative_collision"], "use_iterative_collision"
                    ),
                    "use_dynamical_orientation": _strict_boolean(
                        park_row["use_dynamical_orientation"], "use_dynamical_orientation"
                    ),
                    "disable_dynamical_orientation_for_missiles": _strict_boolean(
                        park_row["disable_dynamical_orientation_for_missiles"],
                        "disable_dynamical_orientation_for_missiles",
                    ),
                    "use_new_orbit": _strict_boolean(park_row["use_new_orbit"], "use_new_orbit"),
                }
                if park_row["snapshot_semantics"] != "logical-authoritative":
                    raise ValueError("unsupported snapshot semantics")
                pending_rows = park_row["pending_removals"]
                if not isinstance(pending_rows, list):
                    raise TypeError("pending_removals must be a list")
                staged_pending: dict[int, int] = {}
                for pending in pending_rows:
                    if not isinstance(pending, dict) or set(pending) != {"ball_id", "due_tick", "reason"}:
                        raise ValueError("pending removal fields do not match schema v3")
                    ball_id = _integer(pending["ball_id"], "pending ball_id")
                    due_tick = _integer(pending["due_tick"], "pending due_tick")
                    if pending["reason"] != "delayed" or ball_id in staged_pending:
                        raise ValueError("invalid or duplicate pending removal")
                    if ball_id not in parsed or due_tick < staged_park["current_time"]:
                        raise ValueError("pending removal references a missing ball or past tick")
                    staged_pending[ball_id] = due_tick
            except (KeyError, TypeError, ValueError, OverflowError) as exc:
                raise BackendCallError(f"invalid snapshot park: {exc}", code="invalid_snapshot") from exc
            if staged_park["friction"] < 0.0:
                raise BackendCallError("snapshot friction is out of range", code="invalid_snapshot")
            if staged_park["current_time"] < 0:
                raise BackendCallError("snapshot current_time must be non-negative", code="invalid_snapshot")
            if not 1 <= staged_park["collision_substeps"] <= MAX_COLLISION_SUBSTEPS:
                raise BackendCallError(
                    f"snapshot collision_substeps must be in 1..{MAX_COLLISION_SUBSTEPS}",
                    code="invalid_snapshot",
                )
            if (
                staged_park["use_dynamical_orientation"]
                or staged_park["use_new_orbit"]
                or staged_park["disable_dynamical_orientation_for_missiles"]
            ):
                raise BackendCallError(
                    "snapshot enables an unsupported orientation, orbit, or missile-orientation mode",
                    code="invalid_snapshot",
                )
            if partial == 2 and staged_park["current_time"] != self._park.current_time:
                raise BackendCallError(
                    "partial mode 2 requires a snapshot from the current simulation tick",
                    code="invalid_snapshot",
                )
            if partial in {0, 1} and staged_park["ego"] != 0 and staged_park["ego"] not in parsed:
                raise BackendCallError(
                    "snapshot ego does not reference a snapshot ball",
                    code="invalid_snapshot",
                )

            self._park.balls = committed
            if partial in {0, 1}:
                self._park.pending_removals = staged_pending
                self._park.proximity_events.clear()
                self._park.network_outbox = {"singlecasts": [], "narrowcasts": [], "batches": []}
                self._park.network_outbox_bytes = 0
                self._park.network_last_batch_id = 0
                self._park.network_last_envelope = None
                self._park.visual_angular_velocity = {
                    ball_id: velocity
                    for ball_id, velocity in self._park.visual_angular_velocity.items()
                    if ball_id in committed
                }
                if partial == 0:
                    for field_name, value in staged_park.items():
                        setattr(self._park, field_name, value)
                else:
                    # Dedicated rollback restore: time and lifecycle are
                    # authoritative while host/controller configuration stays.
                    self._park.current_time = staged_park["current_time"]
                    self._park.time = staged_park["time"]
                    self._park.ego = staged_park["ego"]
                    self._park.running = staged_park["running"]
            else:
                for ball_id in parsed:
                    self._park.pending_removals.pop(ball_id, None)
                self._park.pending_removals.update(staged_pending)
            return None
        if title == "dbc.compat.Proximity.DrainEvents":
            self._require_arity(args, 0, title)
            events, self._park.proximity_events = self._park.proximity_events, []
            return events
        if title == "dbc.compat.Network.DrainOutbox":
            self._require_arity(args, 0, title)
            result = self._park.network_outbox
            self._park.network_outbox = {"singlecasts": [], "narrowcasts": [], "batches": []}
            self._park.network_outbox_bytes = 0
            return result
        raise UnsupportedTitleError.for_title(title)

    def _invoke_ball(
        self,
        title: str,
        operation: str,
        target: str | None,
        args: tuple[Any, ...] | list[Any],
    ) -> Any:
        ball = self._ball(self._ball_id(target))
        name = title.rsplit(".", 1)[1]
        scalar_fields = {
            "mass": "mass",
            "radius": "radius",
            "maxVelocity": "max_velocity",
            "maxAngularVelocity": "max_angular_velocity",
            "isFree": "is_free",
            "isGlobal": "is_global",
            "isMassive": "is_massive",
            "isInteractive": "is_interactive",
            "isCloaked": "is_cloaked",
            "newBubbleId": "new_bubble_id",
            "oldBubbleId": "old_bubble_id",
            "effectStamp": "effect_stamp",
            "Agility": "agility",
            "speedFraction": "speed_fraction",
        }
        vector_fields = {
            "x": ("position", 0), "y": ("position", 1), "z": ("position", 2),
            "vx": ("velocity", 0), "vy": ("velocity", 1), "vz": ("velocity", 2),
            "wx": ("angular_velocity", 0), "wy": ("angular_velocity", 1), "wz": ("angular_velocity", 2),
            "rx": ("rotation", 0), "ry": ("rotation", 1), "rz": ("rotation", 2), "rw": ("rotation", 3),
        }
        if name == "id" and operation == "get":
            self._require_arity(args, 0, title)
            return ball.id
        if name == "ballpark" and operation == "get":
            return self._active_park
        if name in {"centerDist", "surfaceDist"} and operation == "get":
            ego_id = self._park.ego
            if ego_id <= 0 or ego_id == ball.id or ego_id not in self._park.balls:
                return 0.0
            ego = self._ball(ego_id)
            center = math.dist(ball.position, ego.position)
            if not math.isfinite(center):
                raise OverflowError("ball distance exceeds finite f64 range")
            if name == "centerDist":
                return center
            return max(center - ball.radius - ego.radius, 0.0)
        if name in scalar_fields:
            field_name = scalar_fields[name]
            if operation == "get":
                self._require_arity(args, 0, title)
                return getattr(ball, field_name)
            if operation == "set":
                self._require_arity(args, 1, title)
                if name in {"newBubbleId", "oldBubbleId", "effectStamp"}:
                    raise BackendCallError(f"{title} is read-only", code="read_only")
                value = args[0]
                if name in {"mass", "radius", "maxVelocity", "maxAngularVelocity"}:
                    value = max(0.0, _finite_number(value, name))
                    maximum = MAX_RADIUS if name == "radius" else MAX_MASS if name == "mass" else MAX_VELOCITY
                    if value > maximum:
                        raise ValueError(f"{name} exceeds the solver-safe limit")
                elif name == "Agility":
                    value = _finite_number(value, name)
                    if value <= 0.0:
                        value = 1.0
                    if value > MAX_AGILITY:
                        raise ValueError("Agility exceeds the solver-safe limit")
                elif name == "speedFraction":
                    value = min(1.0, max(0.0, _finite_number(value, name)))
                elif name == "isCloaked":
                    value = _integer(value, name)
                    if not 0 <= value <= 3:
                        raise ValueError("isCloaked must be in the range 0..3")
                elif name in {"isFree", "isGlobal", "isMassive", "isInteractive"}:
                    value = _boolean(value, name)
                    if name == "isMassive" and value and ball.is_cloaked:
                        raise ValueError("a cloaked ball cannot be made massive")
                if name == "isCloaked":
                    if value:
                        self._cloak_ball_state(ball, value)
                    else:
                        self._uncloak_ball_state(ball)
                    return None
                if name == "isFree" and not value:
                    ball.velocity = [0.0, 0.0, 0.0]
                    ball.angular_velocity = [0.0, 0.0, 0.0]
                setattr(ball, field_name, value)
                return None
        if name in vector_fields:
            field_name, index = vector_fields[name]
            values = getattr(ball, field_name)
            if operation == "get":
                self._require_arity(args, 0, title)
                return values[index]
            if operation == "set":
                self._require_arity(args, 1, title)
                incoming = _finite_number(args[0], name)
                maximum = MAX_COORDINATE if field_name == "position" else MAX_VELOCITY
                if field_name != "rotation" and abs(incoming) > maximum:
                    raise ValueError(f"{name} exceeds the solver-safe limit")
                staged_values = list(values)
                staged_values[index] = incoming
                if field_name == "rotation":
                    ball.rotation = _normalize_quaternion(staged_values)
                    ball.angular_velocity = [0.0, 0.0, 0.0]
                else:
                    values[index] = incoming
                if field_name == "velocity" and not self._park.use_dynamical_orientation and any(values):
                    ball.rotation = _quaternion_from_x_direction(values)
                return None
        if name in {"roll", "pitch", "yaw"}:
            euler = list(_euler_from_quaternion(ball.rotation))
            index = {"roll": 0, "pitch": 1, "yaw": 2}[name]
            if operation == "get":
                self._require_arity(args, 0, title)
                return euler[index]
            if operation == "set":
                raise BackendCallError(f"{title} is read-only", code="read_only")
        if name == "GetRotatedVector":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 1, title)
            result = _rotate_vector(ball.rotation, _vector(args[0], 3, "vector"))
            if not all(math.isfinite(value) for value in result):
                raise OverflowError("rotated vector exceeds finite f64 range")
            return result
        if name == "AddMiniBall":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 4, title)
            if len(ball.minis) + len(ball.sensors) >= MAX_CHILD_SHAPES_PER_BALL:
                raise BackendCallError("per-ball child-shape limit exceeded", code="limit_exceeded")
            x, y, z, radius = (_finite_number(value, "mini-ball value") for value in args)
            if radius <= 0.0:
                raise ValueError("Radius must be positive")
            descriptor = {"kind": "sphere", "position": [x, y, z], "radius": radius}
            _validate_mini_descriptor(descriptor)
            if _descriptor_bytes([*ball.minis, descriptor, *ball.sensors]) > MAX_CHILD_DESCRIPTOR_BYTES:
                raise BackendCallError("child descriptor byte limit exceeded", code="limit_exceeded")
            self._ensure_live_descriptor_budget(ball.id, [*ball.minis, descriptor], ball.sensors)
            ball.minis.append(descriptor)
            return None
        if name == "AddMiniCapsule":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 7, title)
            if len(ball.minis) + len(ball.sensors) >= MAX_CHILD_SHAPES_PER_BALL:
                raise BackendCallError("per-ball child-shape limit exceeded", code="limit_exceeded")
            *coordinates, radius = (_finite_number(value, "mini-capsule value") for value in args)
            if radius <= 0.0:
                raise ValueError("Radius must be positive")
            descriptor = {
                "kind": "capsule",
                "a": coordinates[:3],
                "b": coordinates[3:],
                "radius": radius,
            }
            _validate_mini_descriptor(descriptor)
            if _descriptor_bytes([*ball.minis, descriptor, *ball.sensors]) > MAX_CHILD_DESCRIPTOR_BYTES:
                raise BackendCallError("child descriptor byte limit exceeded", code="limit_exceeded")
            self._ensure_live_descriptor_budget(ball.id, [*ball.minis, descriptor], ball.sensors)
            ball.minis.append(descriptor)
            return None
        if name == "AddMiniBox":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 12, title)
            if len(ball.minis) + len(ball.sensors) >= MAX_CHILD_SHAPES_PER_BALL:
                raise BackendCallError("per-ball child-shape limit exceeded", code="limit_exceeded")
            basis = [_finite_number(value, "mini-box value") for value in args]
            axes = (basis[3:6], basis[6:9], basis[9:12])
            lengths = [math.sqrt(sum(component * component for component in axis)) for axis in axes]
            if any(length <= 0.0 for length in lengths):
                raise ValueError("mini-box axes must be non-zero")
            normalized = [[component / lengths[index] for component in axes[index]] for index in range(3)]
            for left, right in ((0, 1), (0, 2), (1, 2)):
                dot = sum(normalized[left][index] * normalized[right][index] for index in range(3))
                if abs(dot) > 1e-6:
                    raise ValueError("mini-box axes must be mutually orthogonal")
            determinant = (
                normalized[0][0] * (normalized[1][1] * normalized[2][2] - normalized[1][2] * normalized[2][1])
                - normalized[0][1] * (normalized[1][0] * normalized[2][2] - normalized[1][2] * normalized[2][0])
                + normalized[0][2] * (normalized[1][0] * normalized[2][1] - normalized[1][1] * normalized[2][0])
            )
            if determinant <= 0.0:
                raise ValueError("mini-box basis must be right-handed")
            descriptor = {"kind": "box", "basis": basis}
            _validate_mini_descriptor(descriptor)
            if _descriptor_bytes([*ball.minis, descriptor, *ball.sensors]) > MAX_CHILD_DESCRIPTOR_BYTES:
                raise BackendCallError("child descriptor byte limit exceeded", code="limit_exceeded")
            self._ensure_live_descriptor_budget(ball.id, [*ball.minis, descriptor], ball.sensors)
            ball.minis.append(descriptor)
            return None
        if name == "AddProximitySensor":
            self._require_operation(operation, "call", title)
            self._require_arity(args, {1, 2, 3, 4}, title)
            retained_cloak_sensors = [
                sensor for sensor in ball.sensors if sensor.get("cloak_sensor", False)
            ]
            if len(ball.minis) + len(retained_cloak_sensors) + 1 > MAX_CHILD_SHAPES_PER_BALL:
                raise BackendCallError("per-ball sensor limit exceeded", code="limit_exceeded")
            range_value = _finite_number(args[0], "range")
            period = _finite_number(args[1], "period") if len(args) > 1 else 2.0
            shuffle = _integer(args[2], "shuffle") if len(args) > 2 else 0
            sensor_reach = ball.radius + range_value
            if not math.isfinite(sensor_reach):
                raise OverflowError("sensor range plus ball radius overflowed")
            if sensor_reach < 0.0:
                return -1
            if period <= 0.0:
                raise ValueError("sensor period must be positive")
            descriptor = {
                "range": range_value,
                "period": period,
                "shuffle": shuffle,
                "only_interactives": _boolean(args[3], "onlyInteractives") if len(args) > 3 else False,
                "elapsed": 0.0 if shuffle == 0 else _deterministic_sensor_phase(ball.id, period),
                "members": [],
                "cloak_sensor": False,
            }
            _validate_sensor_descriptor(descriptor)
            sensors = retained_cloak_sensors
            if _descriptor_bytes([*ball.minis, *sensors, descriptor]) > MAX_CHILD_DESCRIPTOR_BYTES:
                raise BackendCallError("child descriptor byte limit exceeded", code="limit_exceeded")
            self._ensure_live_descriptor_budget(ball.id, ball.minis, [descriptor, *sensors])
            ball.sensors = [descriptor, *sensors]
            return 0
        if name == "ApplyImpulsiveForceAtPosition":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 2, title)
            force = _vector(args[0], 3, "force")
            # Destiny's ClientBall API receives a center-relative visual lever;
            # it must never change authoritative linear trajectory.
            lever = _vector(args[1], 3, "position")
            torque = [
                lever[1] * force[2] - lever[2] * force[1],
                lever[2] * force[0] - lever[0] * force[2],
                lever[0] * force[1] - lever[1] * force[0],
            ]
            if not all(math.isfinite(value) for value in torque):
                raise OverflowError("impulse-derived torque exceeds finite f64 range")
            inertia = max(0.4 * ball.mass * ball.radius * ball.radius, sys.float_info.min)
            delta = [0.0, 0.0, 0.0] if math.isinf(inertia) else [0.05 * value / inertia for value in torque]
            current = self._park.visual_angular_velocity.get(ball.id, [0.0, 0.0, 0.0])
            angular_velocity = [current[index] + delta[index] for index in range(3)]
            if not all(math.isfinite(value) for value in angular_velocity):
                raise OverflowError("impulse-derived angular velocity exceeds finite f64 range")
            if ball.max_angular_velocity > 0.0:
                angular_velocity = _clamp_vector_magnitude(angular_velocity, ball.max_angular_velocity)
            self._park.visual_angular_velocity[ball.id] = angular_velocity
            return None
        raise UnsupportedTitleError.for_title(title)

    def _invoke_park(self, title: str, operation: str, args: tuple[Any, ...] | list[Any]) -> Any:
        name = title.rsplit(".", 1)[1]
        park_properties = {
            "isRunning": "running",
            "tickInterval": "tick_interval_ms",
            "friction": "friction",
            "currentTime": "current_time",
            "time": "time",
            "isMaster": "is_master",
            "ego": "ego",
        }
        if name in park_properties:
            field_name = park_properties[name]
            if operation == "get":
                self._require_arity(args, 0, title)
                return getattr(self._park, field_name)
            if operation == "set":
                self._require_arity(args, 1, title)
                if name in {"isRunning", "currentTime", "isMaster"}:
                    raise BackendCallError(f"{title} is read-only", code="read_only")
                value = args[0]
                if name == "tickInterval":
                    value = _tick_interval(value, name)
                elif name == "friction":
                    value = _finite_number(value, name)
                    if value < 0.0:
                        raise ValueError("friction must be non-negative")
                elif name in {"time", "ego"}:
                    value = _integer(value, name)
                setattr(self._park, field_name, value)
                return None
        self._require_operation(operation, "call", title)
        if name == "AddBall":
            self._require_operation(operation, "call", title)
            return self._add_ball(args)
        if name == "Pause":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 0, title)
            self._park.running = False
            return None
        if name == "Start":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 0, title)
            self._park.running = True
            return None
        if name == "Evolve":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 0, title)
            self._evolve_one_tick()
            return None
        if name == "AdjustTimes":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 1, title)
            self._park.time = _checked_i64_add(
                self._park.time,
                _integer(args[0], "delta"),
                "time adjustment",
            )
            return None
        if name == "ClearAll":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 0, title)
            self._park.balls.clear()
            self._park.pending_removals.clear()
            self._park.ego = 0
            self._park.proximity_events.clear()
            self._park.network_outbox = {"singlecasts": [], "narrowcasts": [], "batches": []}
            self._park.network_outbox_bytes = 0
            self._park.network_last_batch_id = 0
            self._park.network_last_envelope = None
            self._park.visual_angular_velocity.clear()
            return None
        if name == "RemoveBall":
            self._require_operation(operation, "call", title)
            self._require_arity(args, {1, 2}, title)
            ball_id = _integer(args[0], "ball_id")
            delay = _integer(args[1], "delay") if len(args) == 2 else 0
            self._schedule_remove_ball(ball_id, delay)
            return None
        if name in {
            "SetBallPosition", "SetBallVelocity", "SetBallAngularVelocity", "SetBallRotation",
        }:
            self._require_operation(operation, "call", title)
            expected = 5 if name == "SetBallRotation" else 4
            self._require_arity(args, expected, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            values = [_finite_number(value, name) for value in args[1:]]
            target_field = {
                "SetBallPosition": "position",
                "SetBallVelocity": "velocity",
                "SetBallAngularVelocity": "angular_velocity",
                "SetBallRotation": "rotation",
            }[name]
            if name == "SetBallRotation":
                values = _normalize_quaternion(values)
            elif any(abs(value) > (MAX_COORDINATE if name == "SetBallPosition" else MAX_VELOCITY) for value in values):
                raise ValueError(f"{name} exceeds the solver-safe limit")
            setattr(ball, target_field, values)
            if name == "SetBallRotation":
                ball.angular_velocity = [0.0, 0.0, 0.0]
            elif name == "SetBallVelocity" and not self._park.use_dynamical_orientation:
                speed = math.sqrt(sum(value * value for value in values))
                if speed > 0.0:
                    ball.rotation = _quaternion_from_x_direction(values)
            return None
        if name in {"SetBallMass", "SetBallRadius", "SetMaxSpeed", "SetMaxAngularSpeed", "SetBallFree", "SetBallGlobal", "SetBallMassive", "SetBallInteractive", "SetSpeedFraction", "SetBallAgility"}:
            self._require_operation(operation, "call", title)
            self._require_arity(args, 2, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            field_name = {
                "SetBallMass": "mass",
                "SetBallRadius": "radius",
                "SetMaxSpeed": "max_velocity",
                "SetMaxAngularSpeed": "max_angular_velocity",
                "SetBallFree": "is_free",
                "SetBallGlobal": "is_global",
                "SetBallMassive": "is_massive",
                "SetBallInteractive": "is_interactive",
                "SetSpeedFraction": "speed_fraction",
                "SetBallAgility": "agility",
            }[name]
            value = args[1]
            if name in {"SetBallMass", "SetBallRadius", "SetMaxSpeed", "SetMaxAngularSpeed"}:
                value = max(0.0, _finite_number(value, name))
                maximum = MAX_RADIUS if name == "SetBallRadius" else MAX_MASS if name == "SetBallMass" else MAX_VELOCITY
                if value > maximum:
                    raise ValueError(f"{name} exceeds the solver-safe limit")
            elif name == "SetSpeedFraction":
                value = min(1.0, max(0.0, _finite_number(value, name)))
            elif name == "SetBallAgility":
                value = _finite_number(value, name)
                if value <= 0.0:
                    value = 1.0
                if value > MAX_AGILITY:
                    raise ValueError("SetBallAgility exceeds the solver-safe limit")
            else:
                value = _boolean(value, name)
                if name == "SetBallMassive" and value and ball.is_cloaked:
                    raise ValueError("a cloaked ball cannot be made massive")
            if name == "SetBallFree" and not value:
                ball.velocity = [0.0, 0.0, 0.0]
                ball.angular_velocity = [0.0, 0.0, 0.0]
            setattr(ball, field_name, value)
            return None
        if name == "Stop":
            self._require_arity(args, 1, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            ball.velocity = [0.0, 0.0, 0.0]
            ball.angular_velocity = [0.0, 0.0, 0.0]
            return None
        if name in {"GetCenterDist", "GetSurfaceDist"}:
            self._require_operation(operation, "call", title)
            self._require_arity(args, 2, title)
            first_id = _integer(args[0], "first_id")
            second_id = _integer(args[1], "second_id")
            first = self._park.balls.get(first_id)
            second = self._park.balls.get(second_id)
            if first is None or second is None:
                return None
            distance = math.dist(first.position, second.position)
            if not math.isfinite(distance):
                raise OverflowError("ball distance exceeds finite f64 range")
            if name == "GetSurfaceDist":
                distance -= first.radius + second.radius
                if not math.isfinite(distance):
                    raise OverflowError("surface distance exceeds finite f64 range")
            return distance
        if name in {"GetBallIdsInRange", "GetBallIdsAndDistInRange"}:
            return self._balls_in_range(args, include_distance=name.endswith("AndDistInRange"))
        if name == "GetBallIdsInCapsule":
            return self._balls_in_capsule(args)
        if name == "GetBallIdsInCone":
            return self._balls_in_cone(args)
        if name == "GetBallIdsInRangeOfTriangle":
            return self._balls_in_triangle(args)
        if name == "ScanCone":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 6, title)
            source_id = _integer(args[0], "source_id")
            source = self._park.balls.get(source_id)
            angle, range_value, dx, dy, dz = args[1:]
            angle = _finite_number(angle, "angle")
            if angle < 0.0:
                raise ValueError("angle must be non-negative")
            range_value = _finite_number(range_value, "range")
            if source is None or source_id in self._park.pending_removals or range_value <= 0.0:
                return None
            direction = [
                _finite_number(dx, "dx"),
                _finite_number(dy, "dy"),
                _finite_number(dz, "dz"),
            ]
            length = math.dist(direction, (0.0, 0.0, 0.0))
            if not math.isfinite(length) or length <= sys.float_info.epsilon:
                return None
            direction = [value / length for value in direction]
            half_angle = angle * 0.5
            sphere = half_angle > math.pi
            cosine = math.cos(half_angle)
            result = []
            for candidate in self._park.balls.values():
                if (
                    candidate.id == source_id
                    or candidate.id < 0
                    or candidate.id in self._park.pending_removals
                    or candidate.is_cloaked
                    or not _same_visibility_partition(source, candidate)
                ):
                    continue
                offset = [candidate.position[index] - source.position[index] for index in range(3)]
                distance = math.dist(offset, (0.0, 0.0, 0.0))
                projection = sum(direction[index] * offset[index] for index in range(3))
                if distance <= range_value and (
                    sphere or (projection >= 0.0 and projection >= cosine * distance)
                ):
                    result.append(candidate.id)
            return sorted(result)
        if name == "AddProximitySensor":
            self._require_operation(operation, "call", title)
            self._require_arity(args, {2, 3, 4, 5}, title)
            ball_id = _integer(args[0], "ball_id")
            self._ball(ball_id)
            park_number = self._active_park.rsplit(":", 1)[1]
            return self._invoke_ball("destiny.Ball.AddProximitySensor", "call", f"ball:{park_number}:{ball_id}", args[1:])
        if name == "RemoveProximitySensor":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 1, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            ball.sensors = [sensor for sensor in ball.sensors if sensor.get("cloak_sensor", False)]
            return None
        if name == "CloakBall":
            self._require_operation(operation, "call", title)
            self._require_arity(args, {2, 3}, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            cloak_mode = _integer(args[1], "cloak_mode")
            uncloak_range = args[2] if len(args) == 3 else None
            self._cloak_ball_state(ball, cloak_mode, uncloak_range)
            return None
        if name == "UncloakBall":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 1, title)
            ball = self._ball(_integer(args[0], "ball_id"))
            self._uncloak_ball_state(ball)
            return None
        if name == "CheckVisibility":
            self._require_operation(operation, "call", title)
            self._require_arity(args, 2, title)
            source_id = _integer(args[0], "source_id")
            destination_id = _integer(args[1], "destination_id")
            source = self._park.balls.get(source_id)
            destination = self._park.balls.get(destination_id)
            if (
                source is None
                or destination is None
                or source_id in self._park.pending_removals
                or destination_id in self._park.pending_removals
                or destination.is_cloaked
                or not _same_visibility_partition(source, destination)
            ):
                return -2
            segment = [destination.position[index] - source.position[index] for index in range(3)]
            if (
                not all(math.isfinite(value) for value in segment)
                or not math.isfinite(math.dist(segment, (0.0, 0.0, 0.0)))
            ):
                return -2
            if not any(segment):
                return 0
            blockers: list[tuple[float, int]] = []
            for candidate in self._park.balls.values():
                if candidate.id in {source_id, destination_id}:
                    continue
                if (
                    candidate.id in self._park.pending_removals
                    or not candidate.is_massive
                    or candidate.is_cloaked
                ):
                    continue
                if not candidate.is_global and candidate.new_bubble_id != source.new_bubble_id:
                    continue
                distance, parameter = _point_segment_distance(candidate.position, source.position, destination.position)
                if 0.0 < parameter < 1.0 and distance <= candidate.radius:
                    blockers.append((parameter, candidate.id))
            return min(blockers)[1] if blockers else 0
        if name == "GetCurrentEgoPos":
            self._require_arity(args, 0, title)
            if (
                self._park.ego <= 0
                or self._park.ego not in self._park.balls
                or self._park.ego in self._park.pending_removals
            ):
                return [0.0, 0.0, 0.0]
            return list(self._park.balls[self._park.ego].position)
        if name == "GetBoxCenter":
            self._require_arity(args, 4, title)
            from ._geometry import GetBoxCenter

            return list(GetBoxCenter(*args))
        if name in {"WriteFullStateToStream", "WriteBallsToStream", "ReadFullStateFromStream"}:
            raise BackendCallError(
                f"{name} is handled by the Python stream adapter and should not reach the backend",
                code="stream_adapter_error",
            )
        raise UnsupportedTitleError.for_title(title)

    def _remove_ball_now(self, ball_id: int) -> None:
        if ball_id not in self._park.balls:
            return
        self._park.balls.pop(ball_id, None)
        self._park.pending_removals.pop(ball_id, None)
        self._park.visual_angular_velocity.pop(ball_id, None)
        if self._park.ego == ball_id:
            self._park.ego = 0
        for owner in self._park.balls.values():
            for sensor in owner.sensors:
                sensor["members"] = [member for member in sensor.get("members", []) if member != ball_id]
        self._park.proximity_events = [
            event
            for event in self._park.proximity_events
            if event.get("owner_id") != ball_id and event.get("other_id") != ball_id
        ]

    def _schedule_remove_ball(self, ball_id: int, delay: int) -> None:
        if delay < 0:
            raise ValueError("removal delay must be non-negative")
        ball = self._park.balls.get(ball_id)
        if ball is None:
            return
        if delay == 0:
            self._remove_ball_now(ball_id)
            return
        due = _checked_i64_add(self._park.current_time, delay, "removal delay")
        ball.velocity = [0.0, 0.0, 0.0]
        ball.angular_velocity = [0.0, 0.0, 0.0]
        ball.is_massive = False
        ball.effect_stamp = due
        self._park.pending_removals[ball_id] = due

    def _remove_due_balls(self) -> None:
        due = [ball_id for ball_id, stamp in self._park.pending_removals.items() if stamp <= self._park.current_time]
        for ball_id in due:
            self._remove_ball_now(ball_id)

    def _evolve_one_tick(self) -> None:
        next_time = _checked_i64_add(self._park.current_time, 1, "current time")
        dt = self._park.tick_interval_ms / 1000.0
        planned_updates: list[tuple[_BallState, list[float], list[float], list[float]]] = []
        for ball in self._park.balls.values():
            if ball.id in self._park.pending_removals or not ball.is_free:
                continue
            old_velocity = list(ball.velocity)
            speed_limit = ball.max_velocity
            if speed_limit >= 0.0:
                old_velocity = _clamp_vector_magnitude(old_velocity, speed_limit)

            # Destiny's space-friction coefficient is a controller/damping
            # term, not an Avian contact material.  With no active steering
            # command this is the exact exponential STOP response.
            if self._park.friction:
                x = math.inf if ball.mass <= 0.0 else (self._park.friction / ball.mass / ball.agility) * dt
                decay = math.exp(-x) if math.isfinite(x) else 0.0
                effective_scale = -math.expm1(-x) / x if math.isfinite(x) and x != 0.0 else (1.0 if x == 0.0 else 0.0)
                integration = effective_scale * dt
            else:
                decay = 1.0
                integration = dt
            position = [ball.position[index] + old_velocity[index] * integration for index in range(3)]
            velocity = [value * decay for value in old_velocity]
            angular_velocity = list(ball.angular_velocity)
            if ball.max_angular_velocity > 0.0:
                angular_velocity = _clamp_vector_magnitude(
                    angular_velocity,
                    ball.max_angular_velocity,
                )
            if not all(math.isfinite(value) for value in (*position, *velocity, *angular_velocity)):
                raise OverflowError("evolution would produce non-finite ball state")
            if any(abs(value) > MAX_COORDINATE for value in position):
                raise OverflowError("evolution would exceed the solver-safe coordinate limit")
            if any(abs(value) > MAX_VELOCITY for value in (*velocity, *angular_velocity)):
                raise OverflowError("evolution would exceed the solver-safe velocity limit")
            planned_updates.append((ball, position, velocity, angular_velocity))

        # Commit only after every ball has passed the numeric preflight so an
        # overflowing update cannot leave a partially advanced park.
        for ball, position, velocity, angular_velocity in planned_updates:
            ball.position = position
            ball.velocity = velocity
            ball.angular_velocity = angular_velocity

        self._run_proximity_checks(dt)
        self._park.current_time = next_time
        for ball in self._park.balls.values():
            if ball.new_bubble_id == -1:
                ball.old_bubble_id = -1
                ball.new_bubble_id = 0
        self._remove_due_balls()

    def _run_proximity_checks(self, dt: float) -> None:
        candidates = list(self._park.balls.values())
        work = 0
        for owner in self._park.balls.values():
            if owner.id in self._park.pending_removals or owner.new_bubble_id < 0:
                continue
            for sensor_index, sensor in enumerate(owner.sensors):
                elapsed = _finite_sum(
                    _finite_number(sensor.get("elapsed", 0.0), "sensor elapsed"),
                    dt,
                    "sensor elapsed time",
                )
                period = _finite_number(sensor.get("period", 2.0), "sensor period")
                if elapsed + 1e-12 < period:
                    sensor["elapsed"] = elapsed
                    continue
                if work + len(candidates) > MAX_PROXIMITY_WORK_PER_TICK:
                    # Preserve a due sensor for the next tick instead of
                    # silently advancing an unbounded all-pairs scan.
                    sensor["elapsed"] = period
                    if len(self._park.proximity_events) < MAX_PROXIMITY_EVENTS:
                        self._park.proximity_events.append({
                            "kind": "overflow",
                            "owner_id": owner.id,
                            "sensor_index": sensor_index,
                            "reason": "work_budget",
                            "tick": self._park.current_time + 1,
                        })
                    continue
                work += len(candidates)
                old_members = {_integer(value, "sensor member") for value in sensor.get("members", [])}
                new_members: set[int] = set()
                range_value = _finite_number(sensor.get("range"), "sensor range")
                only_interactives = bool(sensor.get("only_interactives", False))
                for candidate in candidates:
                    if (
                        candidate.id == owner.id
                        or candidate.id in self._park.pending_removals
                        or candidate.is_cloaked
                    ):
                        continue
                    if only_interactives and not candidate.is_interactive:
                        continue
                    if not _same_visibility_partition(owner, candidate):
                        continue
                    reach = owner.radius + range_value + candidate.radius
                    if math.isfinite(reach) and math.dist(owner.position, candidate.position) <= reach:
                        new_members.add(candidate.id)
                changes = [
                    (entering, candidate_id)
                    for entering, members in (
                        (True, new_members - old_members),
                        (False, old_members - new_members),
                    )
                    for candidate_id in sorted(members)
                ]
                remaining = MAX_PROXIMITY_EVENTS - len(self._park.proximity_events)
                if len(changes) > remaining:
                    sensor["elapsed"] = period
                    if remaining > 0:
                        self._park.proximity_events.append({
                            "kind": "overflow",
                            "owner_id": owner.id,
                            "sensor_index": sensor_index,
                            "required_events": len(changes),
                            "reason": "event_backpressure",
                            "tick": self._park.current_time + 1,
                        })
                    continue
                sensor["elapsed"] = elapsed % period
                sensor["members"] = sorted(new_members)
                for entering, candidate_id in changes:
                    self._park.proximity_events.append({
                        "kind": "transition",
                        "owner_id": owner.id,
                        "other_id": candidate_id,
                        "sensor_index": sensor_index,
                        "entering": entering,
                        "tick": self._park.current_time + 1,
                    })

    def _add_ball(self, args: tuple[Any, ...] | list[Any]) -> str:
        expected = 19 if self._park.use_dynamical_orientation else 17
        if len(args) != expected:
            raise TypeError(
                f"AddBall expects {expected} arguments when useDynamicalOrientation="
                f"{self._park.use_dynamical_orientation}, got {len(args)}"
            )
        ball_id = _integer(args[0], "srcId")
        existing = self._park.balls.get(ball_id)
        if existing is None and len(self._park.balls) >= MAX_LIVE_BALLS:
            raise BackendCallError("live ball limit exceeded", code="limit_exceeded")
        ball = _BallState(
            id=ball_id,
            mass=_finite_number(args[1], "mass"),
            radius=_finite_number(args[2], "radius"),
            max_velocity=_finite_number(args[3], "maxVelocity"),
            is_free=_boolean(args[4], "isFree"),
            is_global=_boolean(args[5], "isGlobal"),
            is_massive=_boolean(args[6], "isMassive"),
            is_interactive=_boolean(args[7], "isInteractive"),
            is_space_junk=_boolean(args[8], "isSpaceJunk"),
            position=[_finite_number(value, "position") for value in args[9:12]],
            velocity=[_finite_number(value, "velocity") for value in args[12:15]],
            agility=_finite_number(args[15], "agility"),
            speed_fraction=_finite_number(args[16], "speedFraction"),
            max_angular_velocity=_finite_number(args[17], "maxAngularSpeed") if len(args) == 19 else MAX_VELOCITY,
            angular_agility=_finite_number(args[18], "angularAgility") if len(args) == 19 else 0.0,
        )
        _validate_ball_state(ball, normalize_api_values=True)
        if not self._park.use_dynamical_orientation and any(ball.velocity):
            ball.rotation = _quaternion_from_x_direction(ball.velocity)
        self._park.pending_removals.pop(ball_id, None)
        if existing is not None:
            # AddBall is an update/repair operation in Destiny. Preserve bound
            # compound geometry and sensors while replacing base state.
            ball.minis = existing.minis
            ball.sensors = existing.sensors
            if not any(ball.velocity):
                ball.rotation = existing.rotation
            ball.angular_velocity = existing.angular_velocity
            ball.is_cloaked = existing.is_cloaked
            ball.new_bubble_id = existing.new_bubble_id
            ball.old_bubble_id = existing.old_bubble_id
            if ball.is_cloaked:
                ball.massive_before_cloak = ball.is_massive
                ball.is_massive = False
            else:
                ball.massive_before_cloak = existing.massive_before_cloak
        self._park.balls[ball_id] = ball
        park_number = self._active_park.rsplit(":", 1)[1]
        return f"ball:{park_number}:{ball_id}"

    def _balls_in_range(self, args: tuple[Any, ...] | list[Any], include_distance: bool) -> list[Any]:
        if len(args) in {2, 3}:
            source = self._ball(_integer(args[0], "source_id"))
            center = source.position
            range_value = _finite_number(args[1], "range")
            include_cloaked = _boolean(args[2], "includeCloaked") if len(args) == 3 else False
            excluded = source.id
        elif len(args) in {4, 5}:
            center = [_finite_number(value, "center") for value in args[:3]]
            range_value = _finite_number(args[3], "range")
            include_cloaked = _boolean(args[4], "includeCloaked") if len(args) == 5 else False
            excluded = None
        else:
            raise TypeError("GetBallIdsInRange expects (id, range[, includeCloaked]) or (x, y, z, range[, includeCloaked])")
        if range_value < 0.0:
            raise ValueError("range must be non-negative")
        if excluded is not None and excluded in self._park.pending_removals:
            return []
        if excluded is not None and source.new_bubble_id < 0:
            return []
        result: list[Any] = []
        for ball_id in sorted(self._park.balls):
            ball = self._park.balls[ball_id]
            if (
                ball.id == excluded
                or ball.id in self._park.pending_removals
                or (ball.is_cloaked and not include_cloaked)
            ):
                continue
            if excluded is not None and not _same_visibility_partition(source, ball):
                continue
            reach = _finite_sum(range_value, ball.radius, "range plus ball radius")
            distance = math.dist(ball.position, center)
            if distance <= reach:
                squared = distance * distance
                if include_distance and not math.isfinite(squared):
                    raise OverflowError("squared query distance exceeds finite f64 range")
                result.append((squared, ball.id) if include_distance else ball.id)
        return result

    def _balls_in_capsule(self, args: tuple[Any, ...] | list[Any]) -> list[int]:
        self._require_arity(args, 5, "GetBallIdsInCapsule")
        source = self._ball(_integer(args[0], "source_id"))
        if source.id in self._park.pending_removals:
            return []
        if source.new_bubble_id < 0:
            return []
        segment = [_finite_number(value, "capsule segment") for value in args[1:4]]
        radius = _finite_number(args[4], "capsule radius")
        if radius < 0.0:
            raise ValueError("capsule radius must be non-negative")
        segment_length = math.dist(segment, (0.0, 0.0, 0.0))
        if not math.isfinite(segment_length):
            raise OverflowError("capsule segment length exceeds finite f64 range")
        direction = [value / segment_length for value in segment] if segment_length else [0.0, 0.0, 0.0]
        result = []
        for ball_id in sorted(self._park.balls):
            ball = self._park.balls[ball_id]
            if (
                ball.id == source.id
                or ball.id in self._park.pending_removals
                or ball.is_cloaked
                or not _same_visibility_partition(source, ball)
            ):
                continue
            reach = _finite_sum(radius, ball.radius, "capsule radius plus ball radius")
            offset = [ball.position[i] - source.position[i] for i in range(3)]
            if not all(math.isfinite(value) for value in offset):
                continue
            projection = sum(offset[i] * direction[i] for i in range(3))
            if not math.isfinite(projection):
                continue
            distance_along = max(
                0.0,
                min(segment_length, projection),
            )
            closest = [source.position[i] + distance_along * direction[i] for i in range(3)]
            if math.dist(ball.position, closest) <= reach:
                result.append(ball.id)
        return result

    def _balls_in_cone(self, args: tuple[Any, ...] | list[Any]) -> list[int]:
        self._require_arity(args, 5, "GetBallIdsInCone")
        source = self._ball(_integer(args[0], "source_id"))
        if source.id in self._park.pending_removals:
            return []
        if source.new_bubble_id < 0:
            return []
        vector = [_finite_number(value, "cone vector") for value in args[1:4]]
        angle = _finite_number(args[4], "cone angle")
        if not 0.0 <= angle <= math.pi:
            raise ValueError("cone angle must be in the range 0..pi")
        height = math.dist(vector, (0.0, 0.0, 0.0))
        if not math.isfinite(height):
            raise OverflowError("cone vector length exceeds finite f64 range")
        if height == 0.0:
            return []
        direction = [value / height for value in vector]
        result = []
        for ball_id in sorted(self._park.balls):
            ball = self._park.balls[ball_id]
            if (
                ball.id == source.id
                or ball.id in self._park.pending_removals
                or ball.is_cloaked
                or not _same_visibility_partition(source, ball)
            ):
                continue
            offset = [ball.position[i] - source.position[i] for i in range(3)]
            _finite_sum(height, ball.radius, "cone height plus ball radius")
            if _sphere_intersects_destiny_cone(offset, ball.radius, direction, height, angle):
                result.append(ball.id)
        return result

    def _balls_in_triangle(self, args: tuple[Any, ...] | list[Any]) -> list[int]:
        if len(args) != 8:
            raise TypeError("GetBallIdsInRangeOfTriangle expects 8 arguments")
        source = self._ball(_integer(args[0], "source_id"))
        if source.id in self._park.pending_removals:
            return []
        if source.new_bubble_id < 0:
            return []
        a = source.position
        b = [a[index] + _finite_number(args[index + 1], "triangle_u") for index in range(3)]
        c = [a[index] + _finite_number(args[index + 4], "triangle_v") for index in range(3)]
        if not all(math.isfinite(value) for value in (*b, *c)):
            raise OverflowError("triangle vertices exceed finite f64 range")
        u = [b[index] - a[index] for index in range(3)]
        v = [c[index] - a[index] for index in range(3)]
        scale = max(abs(value) for value in (*u, *v))
        if scale == 0.0:
            raise ValueError("triangle vertices must define a non-degenerate triangle")
        scaled_u = [value / scale for value in u]
        scaled_v = [value / scale for value in v]
        cross = [
            scaled_u[1] * scaled_v[2] - scaled_u[2] * scaled_v[1],
            scaled_u[2] * scaled_v[0] - scaled_u[0] * scaled_v[2],
            scaled_u[0] * scaled_v[1] - scaled_u[1] * scaled_v[0],
        ]
        if math.dist(cross, (0.0, 0.0, 0.0)) <= sys.float_info.epsilon * 16.0:
            raise ValueError("triangle vertices must define a non-degenerate triangle")
        range_value = _finite_number(args[7], "range")
        if range_value < 0.0:
            raise ValueError("range must be non-negative")
        result = []
        for ball_id in sorted(self._park.balls):
            ball = self._park.balls[ball_id]
            if (
                ball.id == source.id
                or ball.id in self._park.pending_removals
                or ball.is_cloaked
                or not _same_visibility_partition(source, ball)
            ):
                continue
            closest = _closest_point_on_triangle(ball.position, a, b, c)
            reach = _finite_sum(range_value, ball.radius, "triangle range plus ball radius")
            if math.dist(ball.position, closest) <= reach:
                result.append(ball.id)
        return result
