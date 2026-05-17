"""NeoTUI Python package bootstrap."""

from __future__ import annotations

__all__ = ["__version__", "binding_available", "doctor"]

try:
    from . import _native as _native
except ImportError:
    _native = None

__version__ = getattr(_native, "__version__", "0.1.0")
binding_available = _native is not None


def doctor() -> dict[str, object]:
    """Return a tiny package-side diagnostic summary."""
    return {
        "package": "neotui",
        "version": __version__,
        "binding_available": binding_available,
    }
