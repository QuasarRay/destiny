"""Canonical, bounded JSON wire codec shared by Carbon client and server facades."""

from __future__ import annotations

import base64
import binascii
import math


MAX_CARBON_BINARY_BYTES = 32 * 1024 * 1024
MAX_CARBON_TOTAL_BINARY_BYTES = 48 * 1024 * 1024
MAX_CARBON_VALUE_NODES = 200_000
MAX_CARBON_VALUE_DEPTH = 64
MAX_CARBON_STRING_BYTES = 32 * 1024 * 1024
MAX_CARBON_TOTAL_STRING_BYTES = 48 * 1024 * 1024
I64_MIN = -(2**63)
I64_MAX = 2**63 - 1
_BINARY_TAG = "$destiny_bevy_bytes_v1"
_TUPLE_TAG = "$destiny_bevy_tuple_v1"


def _bounded_utf8_length(value: str) -> int:
    """Count UTF-8 bytes without materializing one unbounded encoded copy."""

    if len(value) > MAX_CARBON_STRING_BYTES:
        raise ValueError("Carbon string exceeds the byte limit")
    total = 0
    for start in range(0, len(value), 64 * 1024):
        try:
            total += len(value[start:start + 64 * 1024].encode("utf-8"))
        except UnicodeEncodeError as exc:
            raise ValueError("Carbon strings must contain valid Unicode scalar values") from exc
        if total > MAX_CARBON_STRING_BYTES:
            raise ValueError("Carbon string exceeds the byte limit")
    return total


def encode_carbon_value(value, *, _depth=0, _budget=None):
    """Convert Carbon tuples and bytes into a canonical bounded JSON value."""
    if _budget is None:
        _budget = {"nodes": 0, "bytes": 0, "strings": 0}
    _budget["nodes"] += 1
    if _budget["nodes"] > MAX_CARBON_VALUE_NODES or _depth > MAX_CARBON_VALUE_DEPTH:
        raise ValueError("Carbon value exceeds the nesting/node limit")
    if isinstance(value, (bytes, bytearray, memoryview)):
        size = value.nbytes if isinstance(value, memoryview) else len(value)
        if size > MAX_CARBON_BINARY_BYTES or _budget["bytes"] + size > MAX_CARBON_TOTAL_BINARY_BYTES:
            raise ValueError("Carbon binary value exceeds the byte limit")
        raw = value.tobytes() if isinstance(value, memoryview) else bytes(value)
        if len(raw) != size:
            raise ValueError("Carbon binary view changed size while encoding")
        _budget["bytes"] += size
        return {_BINARY_TAG: base64.b64encode(raw).decode("ascii")}
    if isinstance(value, tuple):
        # The canonical representation contains both the reserved tag object
        # and an inner JSON array. Count both so Python and Rust enforce the
        # same node/depth boundary.
        _budget["nodes"] += 1
        if _budget["nodes"] > MAX_CARBON_VALUE_NODES or _depth + 1 > MAX_CARBON_VALUE_DEPTH:
            raise ValueError("Carbon value exceeds the nesting/node limit")
        return {
            _TUPLE_TAG: [
                encode_carbon_value(item, _depth=_depth + 2, _budget=_budget)
                for item in value
            ]
        }
    if isinstance(value, list):
        return [encode_carbon_value(item, _depth=_depth + 1, _budget=_budget) for item in value]
    if isinstance(value, dict):
        if any(not isinstance(key, str) for key in value):
            raise TypeError("Carbon wire object keys must be strings")
        if _BINARY_TAG in value or _TUPLE_TAG in value:
            raise ValueError("Carbon wire object uses a reserved codec key")
        for key in value:
            try:
                _budget["strings"] += _bounded_utf8_length(key)
            except ValueError as exc:
                raise ValueError("Carbon wire object key exceeds the string byte limit") from exc
            if _budget["strings"] > MAX_CARBON_TOTAL_STRING_BYTES:
                raise ValueError("Carbon value exceeds the total string byte limit")
        return {
            key: encode_carbon_value(item, _depth=_depth + 1, _budget=_budget)
            for key, item in value.items()
        }
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, int):
        if value < I64_MIN or value > I64_MAX:
            raise OverflowError("Carbon integer must fit a signed 64-bit value")
        return value
    if isinstance(value, str):
        _budget["strings"] += _bounded_utf8_length(value)
        if _budget["strings"] > MAX_CARBON_TOTAL_STRING_BYTES:
            raise ValueError("Carbon value exceeds the total string byte limit")
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ValueError("Carbon numeric values must be finite")
        return value
    raise TypeError(f"Carbon wire value of type {type(value).__name__} is not supported")


def decode_carbon_value(value, *, _depth=0, _budget=None):
    """Restore canonical tuples and bytes from a bounded JSON wire value."""
    if _budget is None:
        _budget = {"nodes": 0, "bytes": 0, "strings": 0}
    _budget["nodes"] += 1
    if _budget["nodes"] > MAX_CARBON_VALUE_NODES or _depth > MAX_CARBON_VALUE_DEPTH:
        raise ValueError("Carbon value exceeds the nesting/node limit")
    if isinstance(value, list):
        return [decode_carbon_value(item, _depth=_depth + 1, _budget=_budget) for item in value]
    if isinstance(value, dict):
        if any(not isinstance(key, str) for key in value):
            raise TypeError("Carbon wire object keys must be strings")
        reserved = set(value) & {_BINARY_TAG, _TUPLE_TAG}
        if reserved and len(value) != 1:
            raise ValueError("Carbon wire object mixes a reserved codec key with data")
        if set(value) == {_BINARY_TAG}:
            encoded = value[_BINARY_TAG]
            if not isinstance(encoded, str) or len(encoded) > MAX_CARBON_BINARY_BYTES * 4 // 3 + 16:
                raise ValueError("invalid Carbon binary tag")
            try:
                raw = base64.b64decode(encoded.encode("ascii"), validate=True)
            except (UnicodeError, binascii.Error) as exc:
                raise ValueError("invalid Carbon binary tag") from exc
            if base64.b64encode(raw).decode("ascii") != encoded:
                raise ValueError("invalid Carbon binary tag")
            _budget["bytes"] += len(raw)
            if len(raw) > MAX_CARBON_BINARY_BYTES or _budget["bytes"] > MAX_CARBON_TOTAL_BINARY_BYTES:
                raise ValueError("Carbon binary value exceeds the byte limit")
            return raw
        if set(value) == {_TUPLE_TAG}:
            encoded = value[_TUPLE_TAG]
            if not isinstance(encoded, list):
                raise ValueError("invalid Carbon tuple tag")
            _budget["nodes"] += 1
            if _budget["nodes"] > MAX_CARBON_VALUE_NODES or _depth + 1 > MAX_CARBON_VALUE_DEPTH:
                raise ValueError("Carbon value exceeds the nesting/node limit")
            return tuple(
                decode_carbon_value(item, _depth=_depth + 2, _budget=_budget)
                for item in encoded
            )
        for key in value:
            try:
                _budget["strings"] += _bounded_utf8_length(key)
            except ValueError as exc:
                raise ValueError("Carbon wire object key exceeds the string byte limit") from exc
            if _budget["strings"] > MAX_CARBON_TOTAL_STRING_BYTES:
                raise ValueError("Carbon value exceeds the total string byte limit")
        return {
            key: decode_carbon_value(item, _depth=_depth + 1, _budget=_budget)
            for key, item in value.items()
        }
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, int):
        if value < I64_MIN or value > I64_MAX:
            raise OverflowError("Carbon integer must fit a signed 64-bit value")
        return value
    if isinstance(value, str):
        _budget["strings"] += _bounded_utf8_length(value)
        if _budget["strings"] > MAX_CARBON_TOTAL_STRING_BYTES:
            raise ValueError("Carbon value exceeds the total string byte limit")
        return value
    if isinstance(value, float) and math.isfinite(value):
        return value
    raise TypeError("invalid Carbon JSON wire value")


__all__ = [
    "MAX_CARBON_BINARY_BYTES",
    "MAX_CARBON_STRING_BYTES",
    "MAX_CARBON_TOTAL_STRING_BYTES",
    "MAX_CARBON_TOTAL_BINARY_BYTES",
    "MAX_CARBON_VALUE_DEPTH",
    "MAX_CARBON_VALUE_NODES",
    "decode_carbon_value",
    "encode_carbon_value",
]
