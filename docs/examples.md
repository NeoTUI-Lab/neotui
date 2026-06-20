# NeoTUI Official Examples

These examples are the current MVP fixtures for validating and demonstrating NeoTUI from a terminal-first workflow.

## Example Index

| File | Purpose | Main components |
| ---- | ------- | --------------- |
| `examples/hello.toml` | Smallest runnable application | `Label` |
| `examples/dashboard.toml` | Canonical TOML dashboard smoke demo | `Panel`, `Label`, `Divider` |
| `examples/dashboard.json` | JSON DSL parity for the dashboard shape | `Panel`, `Label`, `Divider`, `Spacer` |
| `examples/layout-demo.toml` | Nested vertical and horizontal layout demo | `VBox`, `HBox`, `Label` |
| `examples/layout-dense.toml` | Header/body/footer layout for dense screens | `Panel`, `VBox`, `HBox`, `Label`, `Divider`, `List`, `TextBlock`, `Button` |
| `examples/layout-sidebar.toml` | Fixed sidebar with growing main content | `Panel`, `HBox`, `VBox`, `List`, `Label`, `Divider`, `TextBlock` |
| `examples/layout-responsive.toml` | Compact layout with one growing primary region | `VBox`, `Panel`, `Label`, `TextBlock`, `HBox`, `Button` |
| `examples/interactive-flow.toml` | Focus, list navigation, scroll and button activation flow | `Panel`, `VBox`, `HBox`, `Label`, `TextBlock`, `List`, `Button` |
| `examples/list-demo.toml` | Focusable list widget demo | `Panel`, `Label`, `Divider`, `List` |
| `examples/rich-dashboard.toml` | Official rich dashboard for frontend composition | `Panel`, `VBox`, `HBox`, `Label`, `TextBlock`, `Button`, `List`, `Graph`, `Divider` |
| `examples/redline-dashboard.toml` | Redline skin foundation demo | `Panel`, `VBox`, `HBox`, `Button`, `List`, `Graph`, `Table`, `BigMetric`, `Gauge`, `Sparkline`, `Knob`, `StatusStrip`, `KeyValueRow` |
| `examples/table-demo.toml` | Dense table/grid widget demo | `Panel`, `VBox`, `Label`, `Table`, `TextBlock` |
| `examples/cockpit-showcase.toml` | Instrumentation HUD with rich frontend widgets | `Panel`, `StatusStrip`, `Metric`, `Gauge`, `Sparkline`, `Table`, `KeyValueRow` |
| `examples/tron-hud.toml` | Redline sci-fi HUD reference skin | `Panel`, `StatusStrip`, `BigMetric`, `Gauge`, `Sparkline`, `Table`, `Knob`, `KeyValueRow`, `Button` |
| `examples/clinic-queue.toml` | Real-world queue display with large hierarchy | `Panel`, `StatusStrip`, `BigMetric`, `Gauge`, `Metric`, `List`, `KeyValueRow` |
| `examples/visual-system-showcase.toml` | Visual System 1.0 reference composition | `Panel`, `StatusStrip`, `BigMetric`, `Metric`, `Gauge`, `Sparkline`, `Table`, `Knob`, `List` |
| `examples/http-dashboard.toml` | Declarative HTTP data source and action fixture | `Panel`, `StatusStrip`, `Metric`, `Gauge`, `Sparkline`, `List`, `Button` |
| `examples/form-intent.toml` | Declarative form state, `$forms` binding and form-driven action payload fixture | `Panel`, `VBox`, `Label`, `TextInput`, `TextBlock`, `StatusStrip`, `Button` |
| `examples/embedded-device-control.toml` | End-to-end embedded Linux device control panel with telemetry, form-backed config and HTTP actions | `Panel`, `VBox`, `HBox`, `Metric`, `Gauge`, `Sparkline`, `Table`, `TextInput`, `StatusStrip`, `Button`, `KeyValueRow` |
| `examples/python/form_intent.py` | Python API version of the form intent fixture, serializing forms and actions to the current DSL contract | `App`, `Form`, `HttpAction`, `TextInput`, `StatusStrip`, `Button` |
| `examples/python/form-intent.json` | JSON contract emitted by the Python form intent example and validated by the core DSL parser | `Panel`, `VBox`, `TextInput`, `StatusStrip`, `Button` |
| `examples/theme-demo.toml` | Theme preset smoke demo | `Panel`, `Label`, `Divider` |
| `examples/showcase-layout.toml` | Richer terminal showcase layout | `Panel`, `VBox`, `HBox`, `Label`, `Divider` |

## Validation

Validate all official examples before a demo or release pass:

```bash
cargo run -p neotui-cli -- check examples/hello.toml
cargo run -p neotui-cli -- check examples/dashboard.toml
cargo run -p neotui-cli -- check examples/dashboard.json
cargo run -p neotui-cli -- check examples/layout-demo.toml
cargo run -p neotui-cli -- check examples/layout-dense.toml
cargo run -p neotui-cli -- check examples/layout-sidebar.toml
cargo run -p neotui-cli -- check examples/layout-responsive.toml
cargo run -p neotui-cli -- check examples/interactive-flow.toml
cargo run -p neotui-cli -- check examples/list-demo.toml
cargo run -p neotui-cli -- check examples/rich-dashboard.toml
cargo run -p neotui-cli -- check examples/redline-dashboard.toml
cargo run -p neotui-cli -- check examples/table-demo.toml
cargo run -p neotui-cli -- check examples/cockpit-showcase.toml
cargo run -p neotui-cli -- check examples/tron-hud.toml
cargo run -p neotui-cli -- check examples/clinic-queue.toml
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
cargo run -p neotui-cli -- check examples/http-dashboard.toml
cargo run -p neotui-cli -- check examples/form-intent.toml
cargo run -p neotui-cli -- check examples/embedded-device-control.toml
cargo run -p neotui-cli -- check examples/python/form-intent.json
cargo run -p neotui-cli -- check examples/theme-demo.toml
cargo run -p neotui-cli -- check examples/showcase-layout.toml
```

Each command should print `check ok` and include a structural summary of the app.

## Running

Run examples from an interactive terminal:

```bash
cargo run -p neotui-cli -- run examples/hello.toml
cargo run -p neotui-cli -- run examples/dashboard.toml
cargo run -p neotui-cli -- run examples/layout-sidebar.toml
cargo run -p neotui-cli -- run examples/layout-responsive.toml
cargo run -p neotui-cli -- run examples/interactive-flow.toml
cargo run -p neotui-cli -- run examples/list-demo.toml
cargo run -p neotui-cli -- run examples/rich-dashboard.toml
cargo run -p neotui-cli -- run examples/redline-dashboard.toml
cargo run -p neotui-cli -- run examples/table-demo.toml
cargo run -p neotui-cli -- run examples/cockpit-showcase.toml
cargo run -p neotui-cli -- run examples/tron-hud.toml
cargo run -p neotui-cli -- run examples/clinic-queue.toml
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
cargo run -p neotui-cli -- run examples/http-dashboard.toml
cargo run -p neotui-cli -- run examples/form-intent.toml
cargo run -p neotui-cli -- run examples/embedded-device-control.toml
cargo run -p neotui-cli -- run examples/showcase-layout.toml
```

Expected runtime behavior:

- NeoTUI enters alternate screen mode.
- The example renders within the terminal.
- `Tab` and `Shift+Tab` move focus across interactive controls when the example contains them.
- `Ctrl+Q` exits and restores the terminal.

The MVP GUI path should use the same examples once Linux GTK/VTE readiness is confirmed with `doctor`:

```bash
cargo run -p neotui-cli -- run examples/dashboard.toml --gui
```

For the primary visual demo flow, see `docs/showcase.md`.

For reusable composition guidance, see `docs/layout-patterns.md`.

For focus, scroll and activation behavior, see `docs/interactions.md`.

For the form-driven action fixture, run `python3 scripts/mock-http-backend.py`, then run `examples/form-intent.toml`. Editing the `TextInput` and pressing `Submit Incident` posts a JSON body whose `summary` field is rendered from `$forms.incident.summary`; the mock backend prints the received `ack payload`.

For the embedded device control panel, run `python3 scripts/mock-http-backend.py`, then run `examples/embedded-device-control.toml`. The app polls `/device/status`, renders telemetry and interface state, lets you edit `hostname` and `mode`, and posts dynamic JSON payloads to `/device/apply` or `/device/restart`; the mock backend prints each `device action` payload for smoke verification.

For an automated pre-smoke check of the embedded device app and mock backend, run `bash scripts/test-embedded-device.sh`.

For the underlying form/action contract, see `docs/form-intent.md`.

The same form/action screen is available through the Python API in `examples/python/form_intent.py`. It builds the app with Python objects, validates it through `check(app)`, then runs the serialized app through the same CLI runtime.

For Python builders, validation helpers and native extension verification, see `docs/python-api.md`.

For reusable starter apps built from the same DSL, see `docs/templates.md`.

For visual review guidance before demos, see `docs/tui-design.md`.

For the first rich visual skin, see `docs/redline-skin.md`.

For the shared visual grammar behind rich TUI screens, see `docs/visual-system.md`.
