"""Errors shared by the Python facade and native ABI adapter."""

from __future__ import annotations

from typing import Any

from ._registry import RECORDS


class DestinyCompatError(RuntimeError):
    """Base class for compatibility-layer errors."""


class BackendUnavailableError(DestinyCompatError):
    """Raised when no compiled Bevy runtime can be found."""


class AbiMismatchError(DestinyCompatError):
    """Raised when the native library and Python package use different ABIs."""


class BackendCallError(DestinyCompatError):
    """Raised when a native or in-process backend rejects a request."""

    def __init__(self, message: str, *, code: str = "backend_error", details: Any = None) -> None:
        super().__init__(message)
        self.code = code
        self.details = details


class UnsupportedTitleError(DestinyCompatError):
    """Raised instead of silently inventing behavior for an unmapped Destiny title."""

    def __init__(
        self,
        canonical_title: str,
        verdict: str,
        material_difference: str = "",
        mapping_items: tuple[str, ...] = (),
    ) -> None:
        self.canonical_title = canonical_title
        self.verdict = verdict
        self.material_difference = material_difference
        self.mapping_items = mapping_items
        difference = f" Material difference: {material_difference}" if material_difference else ""
        mapped = f" Mapped APIs: {', '.join(mapping_items)}." if mapping_items else ""
        super().__init__(f"{canonical_title} is not implemented (mapping verdict {verdict}).{difference}{mapped}")

    @classmethod
    def for_title(cls, canonical_title: str) -> "UnsupportedTitleError":
        record = RECORDS.get(canonical_title)
        if record is None:
            return cls(canonical_title, "UNKNOWN", "The title is not present in the audited registry.")
        return cls(
            canonical_title,
            str(record["verdict"]),
            str(record["material_difference"]),
            tuple(str(item["api_title"]) for item in record["mapping_items"]),
        )


class BallNotFoundError(BackendCallError):
    """Raised when a Destiny ball ID is not present in a ballpark."""

    def __init__(self, ball_id: int) -> None:
        self.ball_id = int(ball_id)
        super().__init__(f"Ball {ball_id} is not in the ballpark", code="ball_not_found")
