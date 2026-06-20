# Embedded Device Control Panel

`examples/embedded-device-control.toml` is NeoTUI's first curated end-to-end application example for embedded Linux and appliance-style environments.

The example demonstrates:

- live HTTP telemetry from `/device/status`;
- CPU, board temperature and load-history visualization;
- network interface state in a table;
- editable runtime configuration through `TextInput`;
- form-backed action payloads using `$forms.device.hostname` and `$forms.device.mode`;
- backend-neutral actions for applying config and restarting an agent;
- action status display through `$actions.<id>.$status`.

## Run

Start the mock backend:

```bash
python3 scripts/mock-http-backend.py
```

Run the control panel from another terminal:

```bash
cargo run -p neotui-cli -- run examples/embedded-device-control.toml
```

Use `Tab` to move focus, edit the hostname or operating mode inputs, then press `Enter` on `Apply Config` or `Restart Agent`.

The backend should print a payload similar to:

```text
device action /device/apply: {"hostname": "edge-gateway-07", "intent": "apply_config", "mode": "maintenance-window"}
```

## Validate

```bash
cargo run -p neotui-cli -- check examples/embedded-device-control.toml
```

Expected output includes `check ok`, `TextInput=2`, `Button=2`, `Table=1` and `StatusStrip=3`.

For the automated verification path, run:

```bash
bash scripts/test-embedded-device.sh
```

The helper runs the core/CLI fixture checks, validates the app with `neotui check`, starts the mock backend, verifies `/device/status`, posts apply/restart action payloads, and confirms the backend printed the edited hostname.

This does not replace the interactive smoke. It verifies the app contract and backend wiring before the final terminal UI run.

The helper starts the mock backend on a free ephemeral port. The interactive app uses the default `127.0.0.1:7878` URLs declared in the TOML fixture, so stop any previous mock backend before starting the final manual smoke.

## Product Narrative

This example is intentionally closer to an operational product than a visual showcase. It represents a device reachable by local console or SSH, where an operator needs a lightweight UI for health, network state and controlled actions without deploying a web application.
