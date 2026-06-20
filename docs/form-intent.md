# Form Intent and Action Payloads

NeoTUI forms capture user input as runtime state and keep widgets backend-neutral. A widget emits intent, the runtime updates `FormStore`, and actions render payloads from the current form state when they are triggered.

## Contract

- Forms are declared with top-level `[[forms]]`.
- Fields are declared with `[[forms.fields]]`.
- Form values are addressed with `$forms.<form_id>.<field_id>`.
- Widgets do not call HTTP directly.
- `TextInput` emits `Command::SetFormValue`.
- HTTP actions render exact `$forms...` strings inside `body.text` or `body.json` at trigger time.
- Required fields referenced by an action payload are checked before HTTP dispatch.
- Invalid submit updates the action lifecycle to `error` and does not spawn the HTTP worker.

## Minimal DSL

```toml
[[forms]]
id = "incident"

[[forms.fields]]
id = "summary"
kind = "text"
initial = "Disk full"
required = true

[[actions]]
id = "submit_incident"
kind = "http"
url = "http://127.0.0.1:7878/ack"
method = "POST"

[actions.body]
json = { summary = "$forms.incident.summary" }
```

Bind the field to an input:

```toml
[root.children.children.props]
form = "incident"
field = "summary"
value_from = "$forms.incident.summary"
```

## Runtime Flow

1. `TextInput` receives keyboard input while focused.
2. It emits `Command::SetFormValue`.
3. The CLI runtime writes the new value into `StateStore.forms`.
4. Runtime bindings rebuild the effective component tree.
5. `Button.on_click` emits `Command::Action`.
6. The HTTP action runtime validates required form fields referenced by the payload.
7. The action runtime renders `$forms...` templates into the HTTP body.
8. The worker posts the rendered payload.
9. `$actions.<id>.$status` bindings expose `loading`, `ready` or `error`.

## Verification

Run the fixture:

```bash
python3 scripts/mock-http-backend.py
cargo run -p neotui-cli -- check examples/form-intent.toml
cargo run -p neotui-cli -- run examples/form-intent.toml
```

Edit the incident summary, focus `Submit Incident`, then press `Enter`.

Expected evidence:

- `check` prints `check ok`.
- The input accepts text and stays focusable with `Tab`.
- The action status strip changes through the action lifecycle.
- The mock backend prints `ack payload` with the edited `summary`.

Full test gate:

```bash
cargo test -p neotui-core --features http
cargo test --workspace
```
