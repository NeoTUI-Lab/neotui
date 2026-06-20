"""NeoTUI Python API bootstrap."""

from __future__ import annotations

import json
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable, Mapping, Sequence

__all__ = [
    "__version__",
    "binding_available",
    "CallbackError",
    "CheckResult",
    "check",
    "doctor",
    "load",
    "loads_json",
    "loads_toml",
    "run",
    "App",
    "DataSource",
    "Action",
    "Form",
    "FormField",
    "Component",
    "Panel",
    "VBox",
    "HBox",
    "Label",
    "TextBlock",
    "TextInput",
    "Divider",
    "Spacer",
    "Button",
    "List",
    "Graph",
    "Metric",
    "Gauge",
    "Sparkline",
    "Table",
    "StatusStrip",
    "BigMetric",
    "Knob",
    "HttpDataSource",
    "HttpAction",
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


@dataclass(frozen=True, slots=True)
class CheckResult:
    """Result from validating a Python-built app through the NeoTUI CLI."""

    returncode: int
    stdout: str
    stderr: str

    @property
    def ok(self) -> bool:
        return self.returncode == 0


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
    data_sources: list["DataSource"] = field(default_factory=list)
    actions: list["Action"] = field(default_factory=list)
    forms: list["Form"] = field(default_factory=list)

    def to_spec(self) -> dict[str, Any]:
        spec: dict[str, Any] = {
            "schema_version": self.schema_version,
            "root": self.root.to_spec(),
        }
        if self.theme is not None:
            spec["theme"] = self.theme
        if self.data_sources:
            spec["data"] = {"sources": [source.to_spec() for source in self.data_sources]}
        if self.actions:
            spec["actions"] = [action.to_spec() for action in self.actions]
        if self.forms:
            spec["forms"] = [form.to_spec() for form in self.forms]
        return spec

    def to_json(self) -> str:
        return json.dumps(self.to_spec(), indent=2)

    @classmethod
    def from_spec(cls, spec: dict[str, Any]) -> "App":
        return cls(
            root=Component.from_spec(spec["root"]),
            schema_version=spec.get("schema_version", "0.1"),
            theme=spec.get("theme"),
            data_sources=[
                DataSource.from_spec(source)
                for source in spec.get("data", {}).get("sources", [])
            ],
            actions=[Action.from_spec(action) for action in spec.get("actions", [])],
            forms=[Form.from_spec(form) for form in spec.get("forms", [])],
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


@dataclass(slots=True)
class DataSource:
    """Declarative data source contract matching the NeoTUI DSL."""

    id: str
    url: str
    kind: str = "http"
    method: str = "GET"
    headers: dict[str, Any] = field(default_factory=dict)
    body: Any | None = None
    timeout_ms: int | None = None
    refresh_ms: int | None = None
    retry_count: int | None = None
    retry_backoff_ms: int | None = None

    def to_spec(self) -> dict[str, Any]:
        spec = _http_effect_spec(
            id=self.id,
            kind=self.kind,
            url=self.url,
            method=self.method,
            headers=self.headers,
            body=self.body,
            timeout_ms=self.timeout_ms,
            retry_count=self.retry_count,
            retry_backoff_ms=self.retry_backoff_ms,
        )
        if self.refresh_ms is not None:
            spec["refresh_ms"] = self.refresh_ms
        return spec

    @classmethod
    def from_spec(cls, spec: Mapping[str, Any]) -> "DataSource":
        return cls(
            id=str(spec["id"]),
            url=str(spec["url"]),
            kind=str(spec.get("kind", "http")),
            method=str(spec.get("method", "GET")),
            headers=dict(spec.get("headers", {})),
            body=spec.get("body"),
            timeout_ms=spec.get("timeout_ms"),
            refresh_ms=spec.get("refresh_ms"),
            retry_count=spec.get("retry_count"),
            retry_backoff_ms=spec.get("retry_backoff_ms"),
        )


@dataclass(slots=True)
class Action:
    """Declarative action contract matching the NeoTUI DSL."""

    id: str
    url: str
    kind: str = "http"
    method: str = "POST"
    headers: dict[str, Any] = field(default_factory=dict)
    body: Any | None = None
    timeout_ms: int | None = None
    retry_count: int | None = None
    retry_backoff_ms: int | None = None
    refresh_sources: list[str] = field(default_factory=list)

    def to_spec(self) -> dict[str, Any]:
        spec = _http_effect_spec(
            id=self.id,
            kind=self.kind,
            url=self.url,
            method=self.method,
            headers=self.headers,
            body=self.body,
            timeout_ms=self.timeout_ms,
            retry_count=self.retry_count,
            retry_backoff_ms=self.retry_backoff_ms,
        )
        if self.refresh_sources:
            spec["refresh_sources"] = list(self.refresh_sources)
        return spec

    @classmethod
    def from_spec(cls, spec: Mapping[str, Any]) -> "Action":
        return cls(
            id=str(spec["id"]),
            url=str(spec["url"]),
            kind=str(spec.get("kind", "http")),
            method=str(spec.get("method", "POST")),
            headers=dict(spec.get("headers", {})),
            body=spec.get("body"),
            timeout_ms=spec.get("timeout_ms"),
            retry_count=spec.get("retry_count"),
            retry_backoff_ms=spec.get("retry_backoff_ms"),
            refresh_sources=list(spec.get("refresh_sources", [])),
        )


@dataclass(slots=True)
class FormField:
    """Declarative form field contract."""

    id: str
    kind: str = "text"
    initial: Any | None = None
    required: bool = False

    def to_spec(self) -> dict[str, Any]:
        spec: dict[str, Any] = {"id": self.id, "kind": self.kind}
        if self.initial is not None:
            spec["initial"] = self.initial
        if self.required:
            spec["required"] = True
        return spec

    @classmethod
    def from_spec(cls, spec: Mapping[str, Any]) -> "FormField":
        return cls(
            id=str(spec["id"]),
            kind=str(spec.get("kind", "text")),
            initial=spec.get("initial"),
            required=bool(spec.get("required", False)),
        )


@dataclass(slots=True)
class Form:
    """Declarative form state definition."""

    id: str
    fields: list[FormField] = field(default_factory=list)

    def to_spec(self) -> dict[str, Any]:
        return {"id": self.id, "fields": [field.to_spec() for field in self.fields]}

    @classmethod
    def from_spec(cls, spec: Mapping[str, Any]) -> "Form":
        return cls(
            id=str(spec["id"]),
            fields=[FormField.from_spec(field) for field in spec.get("fields", [])],
        )


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


def TextBlock(
    text: str,
    *,
    id: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    if width is not None:
        props["width"] = width
    if height is not None:
        props["height"] = height
    return Component("TextBlock", id=id, props=props)


def TextInput(
    *,
    form: str,
    field: str,
    id: str | None = None,
    value: str | None = None,
    value_from: str | None = None,
    placeholder: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"form": form, "field": field}
    if value is not None:
        props["value"] = value
    if value_from is not None:
        props["value_from"] = value_from
    if placeholder is not None:
        props["placeholder"] = placeholder
    if width is not None:
        props["width"] = width
    if height is not None:
        props["height"] = height
    return Component("TextInput", id=id, props=props)


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
    on_click: str | Callable[..., Any] | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    if variant is not None:
        props["variant"] = variant
    component = Component("Button", id=id, props=props)
    if isinstance(on_click, str):
        component.props["on_click"] = on_click
    elif on_click is not None:
        component.bind("click", on_click)
    return component


def List(
    items: Sequence[str],
    *,
    id: str | None = None,
    title: str | None = None,
    items_from: str | None = None,
    on_select: str | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"items": list(items)}
    if title is not None:
        props["title"] = title
    if items_from is not None:
        props["items_from"] = items_from
    if on_select is not None:
        props["on_select"] = on_select
    if height is not None:
        props["height"] = height
    return Component("List", id=id, props=props)


def Graph(
    values: Sequence[int | float],
    *,
    id: str | None = None,
    title: str | None = None,
    values_from: str | None = None,
) -> Component:
    props: dict[str, Any] = {"values": list(values)}
    if title is not None:
        props["title"] = title
    if values_from is not None:
        props["values_from"] = values_from
    return Component("Graph", id=id, props=props)


def Metric(
    value: str | int | float,
    *,
    id: str | None = None,
    title: str | None = None,
    unit: str | None = None,
    status: str | None = None,
    value_from: str | None = None,
    status_from: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"value": value}
    _add_optional_props(
        props,
        title=title,
        unit=unit,
        status=status,
        value_from=value_from,
        status_from=status_from,
        width=width,
        height=height,
    )
    return Component("Metric", id=id, props=props)


def Gauge(
    value: int | float,
    *,
    id: str | None = None,
    title: str | None = None,
    min: int | float | None = None,
    max: int | float | None = None,
    value_from: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"value": value}
    _add_optional_props(
        props,
        title=title,
        min=min,
        max=max,
        value_from=value_from,
        width=width,
        height=height,
    )
    return Component("Gauge", id=id, props=props)


def Sparkline(
    values: Sequence[int | float],
    *,
    id: str | None = None,
    title: str | None = None,
    values_from: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {"values": list(values)}
    _add_optional_props(
        props,
        title=title,
        values_from=values_from,
        width=width,
        height=height,
    )
    return Component("Sparkline", id=id, props=props)


def Table(
    columns: Sequence[Mapping[str, Any]],
    rows: Sequence[Sequence[Any] | Mapping[str, Any]],
    *,
    id: str | None = None,
    title: str | None = None,
    rows_from: str | None = None,
    height: int | None = None,
) -> Component:
    props: dict[str, Any] = {
        "columns": [dict(column) for column in columns],
        "rows": [dict(row) if isinstance(row, Mapping) else list(row) for row in rows],
    }
    _add_optional_props(props, title=title, rows_from=rows_from, height=height)
    return Component("Table", id=id, props=props)


def StatusStrip(
    text: str,
    *,
    id: str | None = None,
    status: str | None = None,
    text_from: str | None = None,
    status_from: str | None = None,
) -> Component:
    props: dict[str, Any] = {"text": text}
    _add_optional_props(
        props,
        status=status,
        text_from=text_from,
        status_from=status_from,
    )
    return Component("StatusStrip", id=id, props=props)


def BigMetric(
    value: str | int | float,
    *,
    id: str | None = None,
    unit: str | None = None,
    font: str | None = None,
    scale: str | None = None,
    value_from: str | None = None,
) -> Component:
    props: dict[str, Any] = {"value": value}
    _add_optional_props(props, unit=unit, font=font, scale=scale, value_from=value_from)
    return Component("BigMetric", id=id, props=props)


def Knob(
    value: int | float,
    *,
    id: str | None = None,
    title: str | None = None,
    min: int | float | None = None,
    max: int | float | None = None,
    value_from: str | None = None,
) -> Component:
    props: dict[str, Any] = {"value": value}
    _add_optional_props(props, title=title, min=min, max=max, value_from=value_from)
    return Component("Knob", id=id, props=props)


def HttpDataSource(
    id: str,
    url: str,
    *,
    method: str = "GET",
    headers: Mapping[str, Any] | None = None,
    body: Any | None = None,
    timeout_ms: int | None = None,
    refresh_ms: int | None = None,
    retry_count: int | None = None,
    retry_backoff_ms: int | None = None,
) -> DataSource:
    return DataSource(
        id=id,
        url=url,
        method=method,
        headers=dict(headers or {}),
        body=body,
        timeout_ms=timeout_ms,
        refresh_ms=refresh_ms,
        retry_count=retry_count,
        retry_backoff_ms=retry_backoff_ms,
    )


def HttpAction(
    id: str,
    url: str,
    *,
    method: str = "POST",
    headers: Mapping[str, Any] | None = None,
    body: Any | None = None,
    timeout_ms: int | None = None,
    retry_count: int | None = None,
    retry_backoff_ms: int | None = None,
    refresh_sources: Sequence[str] | None = None,
) -> Action:
    return Action(
        id=id,
        url=url,
        method=method,
        headers=dict(headers or {}),
        body=body,
        timeout_ms=timeout_ms,
        retry_count=retry_count,
        retry_backoff_ms=retry_backoff_ms,
        refresh_sources=list(refresh_sources or []),
    )


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


def check(
    app: App,
    *,
    neotui_bin: str | Path | None = None,
    cargo_bin: str = "cargo",
) -> CheckResult:
    """Validate a Python-built app through the NeoTUI CLI check command."""
    completed = _run_cli_with_temp_app(
        app,
        command_builder=lambda temp_path: _check_command(
            temp_path,
            neotui_bin=neotui_bin,
            cargo_bin=cargo_bin,
        ),
        capture_output=True,
    )
    return CheckResult(
        returncode=completed.returncode,
        stdout=completed.stdout or "",
        stderr=completed.stderr or "",
    )


def run(app: App, *, cargo_bin: str = "cargo", extra_args: Iterable[str] | None = None) -> subprocess.CompletedProcess[str]:
    """Execute a Python-built app through the NeoTUI CLI."""
    if app.has_callbacks():
        raise RuntimeError(
            "Python callbacks are declared on this app, but the runtime callback bridge is not implemented yet; "
            "invoke callbacks directly from Python tests/helpers for now"
        )

    return _run_cli_with_temp_app(
        app,
        command_builder=lambda temp_path: [
            cargo_bin,
            "run",
            "-p",
            "neotui-cli",
            "--",
            "run",
            str(temp_path),
            *(list(extra_args) if extra_args is not None else []),
        ],
        capture_output=False,
    )


def _run_cli_with_temp_app(
    app: App,
    *,
    command_builder: Callable[[Path], list[str]],
    capture_output: bool,
) -> subprocess.CompletedProcess[str]:
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

    try:
        return subprocess.run(
            command_builder(temp_path),
            cwd=workspace_root,
            text=True,
            check=False,
            capture_output=capture_output,
        )
    finally:
        temp_path.unlink(missing_ok=True)


def _check_command(
    temp_path: Path,
    *,
    neotui_bin: str | Path | None,
    cargo_bin: str,
) -> list[str]:
    if neotui_bin is not None:
        return [str(neotui_bin), "check", str(temp_path)]
    return [
        cargo_bin,
        "run",
        "-p",
        "neotui-cli",
        "--",
        "check",
        str(temp_path),
    ]


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


def _http_effect_spec(
    *,
    id: str,
    kind: str,
    url: str,
    method: str,
    headers: Mapping[str, Any],
    body: Any | None,
    timeout_ms: int | None,
    retry_count: int | None,
    retry_backoff_ms: int | None,
) -> dict[str, Any]:
    spec: dict[str, Any] = {"id": id, "kind": kind, "url": url, "method": method}
    if headers:
        spec["headers"] = dict(headers)
    if body is not None:
        spec["body"] = body
    if timeout_ms is not None:
        spec["timeout_ms"] = timeout_ms
    if retry_count is not None:
        spec["retry_count"] = retry_count
    if retry_backoff_ms is not None:
        spec["retry_backoff_ms"] = retry_backoff_ms
    return spec


def _add_optional_props(props: dict[str, Any], **values: Any) -> None:
    for key, value in values.items():
        if value is not None:
            props[key] = value


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
