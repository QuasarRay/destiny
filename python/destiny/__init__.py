"""Destiny API compatibility facade backed by Bevy, Avian, Lightyear, and Replicon.

Unlike the original package, this module does not call ``blue.LoadExtension``.
Carbon code imports the same public names while the facade crosses a stable C ABI
into the Rust Bevy runtime.
"""

from ._backend import ABI_VERSION, Backend, InMemoryBackend, NativeBackend
from ._enums import LEGACY_CONSTANTS, DstBallMode, DstConstants, DstEventType
from ._errors import (
    AbiMismatchError,
    BackendCallError,
    BackendUnavailableError,
    BallNotFoundError,
    DestinyCompatError,
    UnsupportedTitleError,
)
from ._facade import (
    Ball,
    Ballpark,
    ClientBall,
    clear_backend,
    get_backend,
    set_backend,
    use_in_memory_backend,
    use_native_backend,
)
from ._geometry import Capsule, GetBoxCenter, MiniBall, MiniBox, MiniCapsule, OrientedBox
from ._settings import GlobalSettings, SettingsConfiguration, settings


__all__ = [
    "ABI_VERSION",
    "AbiMismatchError",
    "Backend",
    "BackendCallError",
    "BackendUnavailableError",
    "Ball",
    "BallNotFoundError",
    "Ballpark",
    "Capsule",
    "ClientBall",
    "DestinyCompatError",
    "DstBallMode",
    "DstConstants",
    "DstEventType",
    "GetBoxCenter",
    "GlobalSettings",
    "InMemoryBackend",
    "MiniBall",
    "MiniBox",
    "MiniCapsule",
    "NativeBackend",
    "OrientedBox",
    "SettingsConfiguration",
    "UnsupportedTitleError",
    "clear_backend",
    "get_backend",
    "set_backend",
    "settings",
    "use_in_memory_backend",
    "use_native_backend",
]

globals().update(LEGACY_CONSTANTS)
__all__.extend(sorted(LEGACY_CONSTANTS))
del LEGACY_CONSTANTS
