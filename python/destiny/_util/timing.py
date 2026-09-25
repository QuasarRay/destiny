"""No-dependency timing decorator preserving the Carbon API contract."""

from __future__ import annotations

from functools import wraps


def TimedFunction(_name):
    def decorator(function):
        @wraps(function)
        def wrapped(*args, **kwargs):
            return function(*args, **kwargs)

        return wrapped

    return decorator


__all__ = ["TimedFunction"]
