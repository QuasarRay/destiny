"""Carbon-compatible global Destiny settings value objects."""

from __future__ import annotations

from dataclasses import dataclass, replace
from threading import RLock


MAX_COLLISION_SUBSTEPS = 1_024


@dataclass(slots=True)
class SettingsConfiguration:
    collisionMaxIterations: int = 20
    useIterativeCollision: bool = False
    useDynamicalOrientation: bool = False
    disableDynamicalOrientationForMissiles: bool = False
    useNewOrbit: bool = False

    def copy(self) -> "SettingsConfiguration":
        return replace(self)


class GlobalSettings:
    """Process-level settings object matching ``destiny.settings``."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._default = SettingsConfiguration()
        self._current = self._default.copy()

    def Apply(self, configuration: SettingsConfiguration) -> None:
        if not isinstance(configuration, SettingsConfiguration):
            raise TypeError("Apply expects SettingsConfiguration")
        iterations = configuration.collisionMaxIterations
        if isinstance(iterations, bool) or not isinstance(iterations, int):
            raise TypeError("collisionMaxIterations must be an integer")
        if not 1 <= iterations <= MAX_COLLISION_SUBSTEPS:
            raise ValueError(f"collisionMaxIterations must be in 1..{MAX_COLLISION_SUBSTEPS}")
        for field_name in (
            "useIterativeCollision",
            "useDynamicalOrientation",
            "disableDynamicalOrientationForMissiles",
            "useNewOrbit",
        ):
            if not isinstance(getattr(configuration, field_name), bool):
                raise TypeError(f"{field_name} must be a boolean")
        unsupported = [
            field_name
            for field_name in (
                "useDynamicalOrientation",
                "disableDynamicalOrientationForMissiles",
                "useNewOrbit",
            )
            if getattr(configuration, field_name)
        ]
        if unsupported:
            raise ValueError(
                "unsupported Destiny setting(s): " + ", ".join(unsupported)
            )
        with self._lock:
            self._current = configuration.copy()

    def Get(self) -> SettingsConfiguration:
        with self._lock:
            return self._current.copy()

    def GetDefault(self) -> SettingsConfiguration:
        with self._lock:
            return self._default.copy()

    def Reset(self) -> None:
        with self._lock:
            self._current = self._default.copy()


settings = GlobalSettings()
