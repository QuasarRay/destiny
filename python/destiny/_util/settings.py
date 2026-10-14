# Copyright © 2023 CCP ehf. MIT license retained from original Destiny.
"""Settings helpers used by the original Carbon network modules and tests."""

from destiny._settings import settings


def is_dynamical_orientation_enabled() -> bool:
    return bool(settings.Get().useDynamicalOrientation)


def enable_dynamical_orientation() -> None:
    configuration = settings.Get()
    configuration.useDynamicalOrientation = True
    settings.Apply(configuration)


def disable_dynamical_orientation() -> None:
    configuration = settings.Get()
    configuration.useDynamicalOrientation = False
    settings.Apply(configuration)


def reset_settings_to_default() -> None:
    settings.Apply(settings.GetDefault())


def enable_new_orbit() -> None:
    configuration = settings.Get()
    configuration.useNewOrbit = True
    settings.Apply(configuration)


__all__ = ["is_dynamical_orientation_enabled", "enable_dynamical_orientation",
           "disable_dynamical_orientation", "reset_settings_to_default", "enable_new_orbit"]
