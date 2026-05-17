"""NeoTUI Python API bootstrap."""

from __future__ import annotations

import json
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Sequence

__all__ = [
    "__version__",
    "binding_available",
    "doctor",
    "run",
    "App",
    "Component",
    "Panel",
    "VBox",
    "HBox",
    "Label",
    "Divider",
    "Spacer",
    "Button",
    "List",
    "Graph",
]

try:
    from . import _native as _native
except ImportError:
    _native = None

__version__ = getattr(_native, "__version__", "0.1.0")
binding_available = _native is not None


@dataclass(slots=True)
class Component:
    """Declarative Python-side component builder."""

    kind: str
    id: str | None = None
    props: dict[str, Any] = field(default_factory=dict)
    children: list["Component"] = field(default_factory=list)

    def to_spec(self) -> dict[str, Any]:
        spec: dict[str, Any] = {"kind": self.kind}
        if self.id is not None:
            spec["id"] = self.id
        if self.props:
            spec["props"] = dict(self.props)
        if self.children:
            spec["children"] = [child.to_spec() for child in self.children]
        return spec


@dataclass(slots=True)
class App:
    """Python-side application wrapper."""

    root: Component
    schema_version: str = "0.1"
    theme: str | None = "minimal"

    def to_spec(self) -> dict[str, Any]:
        spec: dict[str, Any] = {
            "schema_version": self.schema_version,
            "root": self.root.to_spec(),
        }
        if self.theme is not None:
            spec["theme"] = self.theme
        return spec

    def to_json(self) -> str:
        return json.dumps(self.to_spec(), indent=2)


def Label(
    text: str,
    *,
    id: str | None = None,
    align: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    if align is not None:
        props["align"] = align
    if width is not None:
        props["width"] = width
    if height is not None:
        props["height"] = height
    return Component("Label", id=id, props=props)


def Divider(
    *,
    id: str | None = None,
    orientation: str | None = None,
    symbol: str | None = None,
) -> Component:
    props: dict[str, Any] = {}
    if orientation is not None:
        props["orientation"] = orientation
    if symbol is not None:
        props["symbol"] = symbol
    return Component("Divider", id=id, props=props)


def Spacer(*, id: str | None = None) -> Component:
    return Component("Spacer", id=id)


def Panel(
    *children: Component,
    id: str | None = None,
    title: str | None = None,
) -> Component:
    props: dict[str, Any] = {}
    if title is not None:
        props["title"] = title
    return Component("Panel", id=id, props=props, children=list(children))


def VBox(
    *children: Component,
    id: str | None = None,
    gap: int | None = None,
    align: str | None = None,
    justify: str | None = None,
) -> Component:
    props = _stack_props(gap=gap, align=align, justify=justify)
    return Component("VBox", id=id, props=props, children=list(children))


def HBox(
    *children: Component,
    id: str | None = None,
    gap: int | None = None,
    align: str | None = None,
    justify: str | None = None,
) -> Component:
    props = _stack_props(gap=gap, align=align, justify=justify)
    return Component("HBox", id=id, props=props, children=list(children))


def Button(
    text: str,
    *,
    id: str | None = None,
    variant: str | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    if variant is not None:
        props["variant"] = variant
    return Component("Button", id=id, props=props)


def List(
    items: Sequence[str],
    *,
    id: str | None = None,
    title: str | None = None,
) -> Component:
    props: dict[str, Any] = {"items": list(items)}
    if title is not None:
        props["title"] = title
    return Component("List", id=id, props=props)


def Graph(
    values: Sequence[int | float],
    *,
    id: str | None = None,
    title: str | None = None,
) -> Component:
    props: dict[str, Any] = {"values": list(values)}
    if title is not None:
        props["title"] = title
    return Component("Graph", id=id, props=props)


def doctor() -> dict[str, object]:
    """Return a tiny package-side diagnostic summary."""
    return {
        "package": "neotui",
        "version": __version__,
        "binding_available": binding_available,
        "workspace_root_found": _workspace_root() is not None,
    }


def run(app: App, *, cargo_bin: str = "cargo", extra_args: Iterable[str] | None = None) -> subprocess.CompletedProcess[str]:
    """Execute a Python-built app through the NeoTUI CLI."""
    workspace_root = _workspace_root()
    if workspace_root is None:
        raise RuntimeError("could not locate the NeoTUI workspace root from the Python package")

    with tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".json",
        prefix="neotui-python-",
        delete=False,
        encoding="utf-8",
    ) as handle:
        handle.write(app.to_json())
        temp_path = Path(handle.name)

    command = [
        cargo_bin,
        "run",
        "-p",
        "neotui-cli",
        "--",
        "run",
        str(temp_path),
    ]
    if extra_args is not None:
        command.extend(extra_args)

    try:
        return subprocess.run(
            command,
            cwd=workspace_root,
            text=True,
            check=False,
        )
    finally:
        temp_path.unlink(missing_ok=True)


def _stack_props(
    *,
    gap: int | None,
    align: str | None,
    justify: str | None,
) -> dict[str, Any]:
    props: dict[str, Any] = {}
    if gap is not None:
        props["gap"] = gap
    if align is not None:
        props["align"] = align
    if justify is not None:
        props["justify"] = justify
    return props


def _workspace_root() -> Path | None:
    current = Path(__file__).resolve()
    for candidate in [current.parent, *current.parents]:
        if (candidate / "AGENTS.md").exists() and (candidate / "Cargo.toml").exists():
            return candidate
    return None
