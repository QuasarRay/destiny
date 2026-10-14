"""Construct a native facade before running its legacy Python decorator init.

The original metaclass allocates the Blue object independently of the decorator's
__init__. Returning the facade base alone would let an overriding __init__ skip
native allocation entirely. Blue wrapper/class/interpreter identity remains an
explicit correspondence boundary of this portable adapter.
"""


def WrapBlueClass(name):
    from destiny import Ball, Ballpark, ClientBall

    classes = {
        "destiny.Ball": Ball,
        "destiny.Ballpark": Ballpark,
        "destiny.ClientBall": ClientBall,
    }
    try:
        base = classes[name]
    except KeyError as exc:
        raise ValueError(f"unknown Destiny class {name!r}") from exc

    class DecoratorMeta(type):
        def __call__(cls, inst=None, initDict=None, *args, **kwargs):
            result = cls.__new__(cls)
            if inst is None:
                base.__init__(result)
            elif isinstance(inst, Ball) and issubclass(base, Ball):
                base.__init__(result, inst)
            else:
                raise TypeError("wrapping this existing object is not implemented")
            try:
                if initDict is not None:
                    if not isinstance(initDict, dict):
                        raise TypeError("initDict must be a dictionary")
                    for key, value in initDict.items():
                        setattr(result, key, value)
                initializer = cls.__init__
                if initializer is not base.__init__:
                    initializer(result)
                return result
            except Exception:
                if isinstance(result, Ballpark):
                    result.close()
                raise

    class Wrapped(base, metaclass=DecoratorMeta):
        __slots__ = ()
        __cid__ = name

    return Wrapped


__all__ = ["WrapBlueClass"]
