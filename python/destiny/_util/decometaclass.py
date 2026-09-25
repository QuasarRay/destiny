"""Legacy helper retained for imports; native Blue wrapping is unnecessary."""


def WrapBlueClass(name):
    from destiny import Ball, Ballpark, ClientBall

    classes = {
        "destiny.Ball": Ball,
        "destiny.Ballpark": Ballpark,
        "destiny.ClientBall": ClientBall,
    }
    try:
        return classes[name]
    except KeyError as exc:
        raise ValueError(f"unknown Destiny class {name!r}") from exc


__all__ = ["WrapBlueClass"]
