"""NeoTUI Python API bootstrap."""

from __future__ import annotations

import json
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable, Sequence

__all__ = [
    "__version__",
    "binding_available",
    "CallbackError",
    "doctor",
    "load",
    "loads_json",
    "loads_toml",
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


class CallbackError(RuntimeError):
    """Raised when a Python-side UI callback fails."""

    def __init__(self, component: "Component", event_name: str, cause: Exception) -> None:
        self.component_id = component.id
        self.component_kind = component.kind
        self.event_name = event_name
        self.__cause__ = cause
        target = component.id or component.kind
        super().__init__(f"callback `{event_name}` failed for component `{target}`: {cause}")


@dataclass(slots=True)
class Component:
    """Declarative Python-side component builder."""

    kind: str
    id: str | None = None
    props: dict[str, Any] = field(default_factory=dict)
    children: list["Component"] = field(default_factory=list)
    callbacks: dict[str, Callable[..., Any]] = field(default_factory=dict, repr=False, compare=False)

    def to_spec(self) -> dict[str, Any]:
        spec: dict[str, Any] = {"kind": self.kind}
        if self.id is not None:
            spec["id"] = self.id
        if self.props:
            spec["props"] = dict(self.props)
        if self.children:
            spec["children"] = [child.to_spec() for child in self.children]
        return spec

    @classmethod
    def from_spec(cls, spec: dict[str, Any]) -> "Component":
        return cls(
            kind=spec["kind"],
            id=spec.get("id"),
            props=dict(spec.get("props", {})),
            children=[cls.from_spec(child) for child in spec.get("children", [])],
        )

    def bind(self, event_name: str, callback: Callable[..., Any]) -> "Component":
        if not callable(callback):
            raise TypeError(f"callback for `{event_name}` must be callable")
        self.callbacks[event_name] = callback
        return self

    def invoke(self, event_name: str, *args: Any, **kwargs: Any) -> Any:
        callback = self.callbacks.get(event_name)
        if callback is None:
            target = self.id or self.kind
            raise LookupError(f"component `{target}` has no `{event_name}` callback registered")

        try:
            return callback(*args, **kwargs)
        except Exception as exc:
            raise CallbackError(self, event_name, exc) from exc

    def iter_components(self) -> Iterable["Component"]:
        yield self
        for child in self.children:
            yield from child.iter_components()

    def has_callbacks(self) -> bool:
        return bool(self.callbacks)


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

    @classmethod
    def from_spec(cls, spec: dict[str, Any]) -> "App":
        return cls(
            root=Component.from_spec(spec["root"]),
            schema_version=spec.get("schema_version", "0.1"),
            theme=spec.get("theme"),
        )

    def iter_components(self) -> Iterable[Component]:
        yield from self.root.iter_components()

    def callback_bindings(self) -> dict[str, list[str]]:
        bindings: dict[str, list[str]] = {}
        for component in self.iter_components():
            if component.callbacks:
                bindings[component.id or component.kind] = sorted(component.callbacks)
        return bindings

    def has_callbacks(self) -> bool:
        return any(component.has_callbacks() for component in self.iter_components())


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
    on_click: Callable[..., Any] | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    if variant is not None:
        props["variant"] = variant
    component = Component("Button", id=id, props=props)
    if on_click is not None:
        component.bind("click", on_click)
    return component


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
        "callback_contract_available": True,
        "runtime_callback_bridge": False,
    }


def load(path: str | Path) -> App:
    source_path = Path(path)
    suffix = source_path.suffix.lower()
    payload = source_path.read_text(encoding="utf-8")

    if suffix == ".json":
        return loads_json(payload)
    if suffix == ".toml":
        return loads_toml(payload)

    raise ValueError(f"unsupported NeoTUI DSL format for `{source_path}`; expected .json or .toml")


def loads_json(payload: str) -> App:
    return App.from_spec(json.loads(payload))


def loads_toml(payload: str) -> App:
    toml = _toml_module()
    return App.from_spec(toml.loads(payload))


def run(app: App, *, cargo_bin: str = "cargo", extra_args: Iterable[str] | None = None) -> subprocess.CompletedProcess[str]:
    """Execute a Python-built app through the NeoTUI CLI."""
    workspace_root = _workspace_root()
    if workspace_root is None:
        raise RuntimeError("could not locate the NeoTUI workspace root from the Python package")
    if app.has_callbacks():
        raise RuntimeError(
            "Python callbacks are declared on this app, but the runtime callback bridge is not implemented yet; "
            "invoke callbacks directly from Python tests/helpers for now"
        )

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


def _toml_module() -> Any:
    try:
        import tomllib  # type: ignore[attr-defined]

        return tomllib
    except ModuleNotFoundError:
        try:
            import tomli

            return tomli
        except ModuleNotFoundError as exc:
            raise RuntimeError(
                "TOML loading requires Python 3.11+ or the optional `tomli` package on older Python versions"
            ) from exc
