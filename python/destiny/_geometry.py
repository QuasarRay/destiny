"""Immutable geometry descriptors used by supported compound-shape calls.

These are deliberately value objects, not fake runtime entities. Runtime-owned
mini geometry is created through ``Ball.AddMini*``; unsupported standalone
Capsule/OrientedBox lifecycle operations fail through the title registry.
"""

from __future__ import annotations

from dataclasses import dataclass
import math


@dataclass(frozen=True, slots=True)
class MiniBall:
    radius: float = 0.0
    x: float = 0.0
    y: float = 0.0
    z: float = 0.0
    id: int = 0


@dataclass(frozen=True, slots=True)
class MiniCapsule:
    radius: float = 0.0
    ax: float = 0.0
    ay: float = 0.0
    az: float = 0.0
    bx: float = 0.0
    by: float = 0.0
    bz: float = 0.0
    id: int = 0


@dataclass(frozen=True, slots=True)
class MiniBox:
    c0: float = 0.0
    c1: float = 0.0
    c2: float = 0.0
    x0: float = 1.0
    x1: float = 0.0
    x2: float = 0.0
    y0: float = 0.0
    y1: float = 1.0
    y2: float = 0.0
    z0: float = 0.0
    z1: float = 0.0
    z2: float = 1.0
    id: int = 0


@dataclass(frozen=True, slots=True)
class Capsule:
    id: int = 0
    ax: float = 0.0
    ay: float = 0.0
    az: float = 0.0
    bx: float = 0.0
    by: float = 0.0
    bz: float = 0.0
    radius: float = 0.0
    park: object | None = None
    isMoribund: bool = False


@dataclass(frozen=True, slots=True)
class OrientedBox:
    id: int = 0
    corner_x: float = 0.0
    park: object | None = None
    isMoribund: bool = False


def GetBoxCenter(level_or_tuple, x=None, y=None, z=None) -> tuple[float, float, float]:
    """Return the Destiny partition-cell center.

    Accepts either ``(level, x, y, z)`` or four positional arguments, matching
    both common Carbon call patterns.
    """

    if x is None and y is None and z is None:
        try:
            level, x, y, z = level_or_tuple
        except (TypeError, ValueError) as exc:
            raise TypeError("GetBoxCenter expects (level, x, y, z)") from exc
    else:
        level = level_or_tuple
    if isinstance(level, bool) or not isinstance(level, int):
        raise TypeError("level must be an integer")
    if not 0 <= level < 8:
        raise TypeError("Illegal level")
    grid_unit = 480.0
    big_box = (1 << 16) * grid_unit * 0.25
    width = big_box / (1 << (2 * level))
    grid = 1 << (2 * level - 16 + 39)

    def center(value: float) -> float:
        # Truncation matches the original C++ integer conversion only while the
        # quotient is representable as signed int64. Reject out-of-domain
        # coordinates explicitly instead of relying on Python's unbounded int
        # or Rust's saturating float-to-int cast, which would diverge.
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise TypeError("box coordinates must be finite numbers")
        value = float(value)
        if not math.isfinite(value):
            raise ValueError("box coordinates must be finite")
        quotient = (value + 0.5 * width * grid) / width
        if not math.isfinite(quotient) or not -(2**63) <= quotient < 2**63:
            raise ValueError("box coordinate maps outside the signed 64-bit grid")
        index = math.trunc(quotient)
        return index * width + width * 0.5 - width * grid * 0.5

    return center(x), center(y), center(z)
