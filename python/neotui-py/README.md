## NeoTUI Python Package

Minimal Python API bootstrap for NeoTUI bindings.

Current scope:

- installable `neotui` package metadata
- importable `neotui` module from `src/`
- optional native extension hook via PyO3 and maturin
- declarative builders such as `App`, `Panel`, `VBox`, `HBox`, `Label`, `Divider`, `Spacer`
- forward-compatible Python builders for `Button`, `List` and `Graph`
- DSL loading from `.toml` and `.json` files into the Python-side `App` model
- safe Python-side callback contract for `Button(on_click=...)`
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

Python-side callbacks are available on the package model:

```python
from neotui import Button

button = Button("Deploy", id="deploy", on_click=lambda: "ok")
assert button.invoke("click") == "ok"
```

At this stage, callbacks are intentionally not forwarded through `run(app)` yet. The package raises an explicit runtime error instead of silently dropping them.
