## NeoTUI Python Package

Minimal Python API bootstrap for NeoTUI bindings.

Current scope:

- installable `neotui` package metadata
- importable `neotui` module from `src/`
- optional native extension hook via PyO3 and maturin
- declarative builders such as `App`, `Panel`, `VBox`, `HBox`, `Label`, `Divider`, `Spacer`
- Python builders for base and rich widgets, including `TextInput`, `TextBlock`, `Button`, `List`, `Graph`, `Metric`, `Gauge`, `Sparkline`, `Table`, `StatusStrip`, `BigMetric` and `Knob`
- Python-side data source, action and form definitions that serialize to the same neutral DSL shape used by TOML/JSON apps
- DSL loading from `.toml` and `.json` files into the Python-side `App` model
- safe Python-side callback contract for `Button(on_click=...)`
- declarative action bindings such as `Button("Submit", on_click="submit_incident")`
- `check(app)` support through the existing `neotui-cli check` path
- `run(app)` support through the existing `neotui-cli` runtime path
- tiny package-side `doctor()` helper for smoke validation

The native runtime bindings are intentionally small at this stage and will grow in the `US-010.x` tasks.

Example:

```python
from neotui import App, Panel, VBox, Label, run

app = App(
    Panel(
        VBox(
            Label("Hello NeoTUI", align="center"),
            gap=1,
            align="center",
        ),
        title="Python Demo",
    )
)

run(app)
```

You can also load an existing DSL file:

```python
from neotui import load

app = load("examples/hello.toml")
```

Validate a Python-built app through the CLI check path:

```python
from neotui import App, Label, check

result = check(App(Label("Hello")))
assert result.ok
```

Form-backed action payloads can be built from Python without coupling widgets to HTTP:

```python
from neotui import App, Button, Form, FormField, HttpAction, Panel, TextInput, VBox

app = App(
    Panel(
        VBox(
            TextInput(form="incident", field="summary", value_from="$forms.incident.summary"),
            Button("Submit", on_click="submit_incident"),
        )
    ),
    forms=[Form("incident", [FormField("summary", required=True)])],
    actions=[
        HttpAction(
            "submit_incident",
            "http://127.0.0.1:7878/ack",
            body={"json": {"summary": "$forms.incident.summary"}},
        )
    ],
)
```

Python-side callbacks are available on the package model:

```python
from neotui import Button

button = Button("Deploy", id="deploy", on_click=lambda: "ok")
assert button.invoke("click") == "ok"
```

At this stage, callbacks are intentionally not forwarded through `run(app)` yet. The package raises an explicit runtime error instead of silently dropping them.
