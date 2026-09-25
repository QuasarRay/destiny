"""Carbon-compatible weak signal/slot implementation."""

from __future__ import annotations

import inspect
import logging
import weakref


class Signal:
    _monitor_enabled = False
    _monitor_callback = None

    def __init__(self, signalName=None):
        self._signalName = signalName
        self._functions = weakref.WeakSet()
        self._methods = weakref.WeakKeyDictionary()

    def __repr__(self):
        prefix = f"{self._signalName} " if self._signalName else ""
        connected = ",".join(str(slot) for slot in self)
        suffix = f" connected to {connected}>" if connected else ">"
        return prefix + object.__repr__(self)[:-1] + suffix

    def __call__(self, *args, **kwargs):
        if type(self)._monitor_enabled and type(self)._monitor_callback is not None:
            type(self)._monitor_callback(self._signalName, args, kwargs)
        for slot in self:
            try:
                slot(*args, **kwargs)
            except Exception:  # noqa: BLE001 - one Carbon subscriber must not abort the signal
                logging.exception("Exception in signal handler: %s", slot)

    def __len__(self):
        return len(self._functions) + sum(len(methods) for methods in self._methods.values())

    def __iter__(self):
        callables = list(self._functions)
        for owner, functions in tuple(self._methods.items()):
            callables.extend(function.__get__(owner) for function in tuple(functions))
        return iter(callables)

    def connect(self, slot):
        if inspect.ismethod(slot):
            self._methods.setdefault(slot.__self__, weakref.WeakSet()).add(slot.__func__)
        elif inspect.isfunction(slot) and slot.__name__ == "<lambda>":
            raise TypeError("Signal cannot connect lambda methods")
        elif callable(slot):
            self._functions.add(slot)
        else:
            raise TypeError("Signal connect requires a callable slot")

    def disconnect(self, slot):
        if inspect.ismethod(slot):
            methods = self._methods.get(slot.__self__)
            if methods is not None:
                methods.discard(slot.__func__)
        else:
            self._functions.discard(slot)

    def clear(self):
        self._functions.clear()
        self._methods.clear()

    @property
    def debugDisplayName(self):
        return self._signalName

    @classmethod
    def StartSignalMonitoring(cls, callback):
        if not callable(callback):
            raise TypeError("signal monitor must be callable")
        cls._monitor_enabled = True
        cls._monitor_callback = callback

    @classmethod
    def StopSignalMonitoring(cls):
        cls._monitor_enabled = False
        cls._monitor_callback = None


__all__ = ["Signal"]
