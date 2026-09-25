"""Compatibility utilities retained at Carbon's original import paths."""

from .settings import is_dynamical_orientation_enabled
from .signal import Signal
from .timing import TimedFunction

__all__ = ["Signal", "TimedFunction", "is_dynamical_orientation_enabled"]
