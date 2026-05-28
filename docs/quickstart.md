# NeoTUI Quickstart

This guide gets a new developer from a fresh checkout to a running MVP example.
The reference environment for the MVP is Linux or WSL with a Rust stable toolchain.

## 1. Check the Toolchain

```bash
rustc --version
cargo --version
cargo metadata --format-version 1 --no-deps
```

Expected outcome:

- `rustc` and `cargo` print stable toolchain versions.
- `cargo metadata` succeeds from the repository root and lists `neotui-core`, `neotui-cli` and `neotui-gui`.

## 2. Install Linux GUI Prerequisites

The terminal runtime is the primary MVP path. The workspace also contains the GTK/VTE GUI binary, so full workspace or CLI builds need GTK development packages available.

On Ubuntu or WSL:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev libvte-2.91-gtk4-dev
```

Expected outcome:

- `pkg-config --modversion gtk4` prints a GTK version.
- `pkg-config --modversion vte-2.91-gtk4` prints a VTE version.

Windows without WSL is not an MVP target. If a direct Windows build fails with missing `pkg-config`, `dlltool`, GTK, GLib or VTE libraries, use WSL/Linux for the current MVP quickstart.

## 3. Build and Test

```bash
cargo build --workspace
cargo test --workspace
```

PowerShell helpers are available for the same checks:

```powershell
.\scripts\build.ps1
.\scripts\test.ps1
```

Expected outcome:

- The workspace compiles.
- The test suite exits successfully.

## 4. Validate Example DSL

```bash
cargo run -p neotui-cli -- check examples/hello.toml
cargo run -p neotui-cli -- check examples/dashboard.toml
cargo run -p neotui-cli -- check examples/list-demo.toml
cargo run -p neotui-cli -- check examples/rich-dashboard.toml
cargo run -p neotui-cli -- check examples/redline-dashboard.toml
cargo run -p neotui-cli -- check examples/table-demo.toml
cargo run -p neotui-cli -- check examples/cockpit-showcase.toml
cargo run -p neotui-cli -- check examples/tron-hud.toml
cargo run -p neotui-cli -- check examples/clinic-queue.toml
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
```

Expected outcome:

- Each command prints `check ok`.
- The summary includes the detected format, root component and component counts.

PowerShell shortcut:

```powershell
.\scripts\check.ps1 examples\hello.toml
```

## 5. Run the Terminal MVP

Use an interactive terminal, because `run` enters alternate screen mode and reads keyboard input.

```bash
cargo run -p neotui-cli -- run examples/hello.toml
```

Expected outcome:

- NeoTUI switches to an alternate terminal screen.
- The screen shows `Hello NeoTUI`.
- Press `Ctrl+Q` to exit.
- The terminal returns to its normal screen after exit.

For the dashboard demo:

```bash
cargo run -p neotui-cli -- run examples/dashboard.toml
```

For the list widget demo:

```bash
cargo run -p neotui-cli -- run examples/list-demo.toml
```

For the richer frontend composition demo:

```bash
cargo run -p neotui-cli -- run examples/rich-dashboard.toml
```

For the redline skin foundation demo:

```bash
cargo run -p neotui-cli -- run examples/redline-dashboard.toml
```

For the dense table widget demo:

```bash
cargo run -p neotui-cli -- run examples/table-demo.toml
```

For the richer instrumentation HUD:

```bash
cargo run -p neotui-cli -- run examples/cockpit-showcase.toml
```

For the redline sci-fi HUD reference:

```bash
cargo run -p neotui-cli -- run examples/tron-hud.toml
```

For a real-world queue display using large visual hierarchy:

```bash
cargo run -p neotui-cli -- run examples/clinic-queue.toml
```

For the Visual System 1.0 reference composition:

```bash
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
```

See `docs/examples.md` for the full official example catalog.

For reusable layout patterns and small fixtures, see `docs/layout-patterns.md`.

For focus, keyboard navigation, scroll and button activation behavior, see `docs/interactions.md`.

For copy-and-adapt starter files, see `docs/templates.md`.

For visual review guidance, see `docs/tui-design.md`.

For the redline visual skin, see `docs/redline-skin.md`.

For the robust rich-TUI visual grammar, see `docs/visual-system.md`.

For the visual showcase flow:

```bash
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
```

The complete demo checklist lives in `docs/showcase.md`.

For local Linux installation after the first run succeeds, see `docs/linux-install.md`.

## 6. Inspect Runtime Readiness

```bash
cargo run -p neotui-cli -- doctor
```

Expected outcome:

- The report shows terminal readiness fields such as TTY status, size, color support and GUI readiness.
- Environment variable values are not printed directly.

If GUI launch fails, run `doctor` first and check the `gui_*` fields before retrying.

## 7. Optional Debug Tracing

```bash
NEOTUI_DEBUG=1 cargo run -p neotui-cli -- check examples/hello.toml
```

Expected outcome:

- Extra subsystem diagnostics may be printed.
- Logs should include technical metadata only, not full component text payloads.

## 8. Optional Baseline Benchmarks

```bash
cargo test -p neotui-core --test benchmarks -- --ignored --nocapture
```

PowerShell shortcut:

```powershell
.\scripts\bench.ps1
```

Expected outcome:

- Lightweight benchmark timings are printed for layout, showcase render and frame diff baselines.

## Troubleshooting

| Symptom | Likely cause | Next step |
| ------- | ------------ | --------- |
| `pkg-config` not found while building GTK/GLib/VTE crates | Linux GUI development packages are missing | Install `pkg-config`, `libgtk-4-dev` and `libvte-2.91-gtk4-dev` |
| `dlltool.exe` not found on Windows | Direct Windows builds are outside the MVP target | Use WSL/Linux for the MVP quickstart |
| `run` exits or behaves oddly in an IDE output panel | The command needs an interactive terminal | Run it from a real terminal session |
| `check` rejects `.yaml` files | Core supports TOML and JSON as canonical formats right now | Use `.toml` or `.json` examples |
| GUI launch reports session missing | No Linux graphical session is available | Start a session with `DISPLAY` or `WAYLAND_DISPLAY`, then run `doctor` |
