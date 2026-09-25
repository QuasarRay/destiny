"""Settings helpers used by the original Carbon network modules."""

from destiny._settings import settings


def is_dynamical_orientation_enabled() -> bool:
    return bool(settings.Get().useDynamicalOrientation)


__all__ = ["is_dynamical_orientation_enabled"]
