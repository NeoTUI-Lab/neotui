# NeoTUI Showcase

This is the current MVP showcase path for a terminal-first demo. It uses real DSL fixtures and the same `neotui run` path a developer uses locally.

## Primary Demo

Use `examples/visual-system-showcase.toml` as the current rich visual showcase. It is calmer than the raw HUD reference and demonstrates Visual System 1.0: controlled chrome, a single hero region, semantic panel variants and data-focused color.

```bash
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
```

Expected outcome:

- `check` prints `check ok`.
- The app opens in alternate screen mode.
- The terminal shows a modern redline visual-system composition.
- The screen combines `StatusStrip`, `BigMetric`, metrics, gauges, sparklines, table data, a knob and a list.
- `Ctrl+Q` exits and restores the terminal.

Recommended terminal size for review or recording: at least `100x28`.

Use `examples/showcase-layout.toml` as the stable MVP layout showcase:

```bash
cargo run -p neotui-cli -- check examples/showcase-layout.toml
cargo run -p neotui-cli -- run examples/showcase-layout.toml
```

Expected outcome:

- `check` prints `check ok`.
- The app opens in alternate screen mode.
- The terminal shows an `Operations Board` panel.
- The content includes `Cluster Overview`, service status cells and the `All critical services responding` footer.
- `Ctrl+Q` exits and restores the terminal.

Recommended terminal size for review or recording: at least `80x24`.

## Demo Sequence

For a short MVP review, run these in order:

```bash
cargo run -p neotui-cli -- doctor
cargo run -p neotui-cli -- check examples/dashboard.toml
cargo run -p neotui-cli -- check examples/list-demo.toml
cargo run -p neotui-cli -- check examples/rich-dashboard.toml
cargo run -p neotui-cli -- check examples/showcase-layout.toml
cargo run -p neotui-cli -- check examples/cockpit-showcase.toml
cargo run -p neotui-cli -- check examples/tron-hud.toml
cargo run -p neotui-cli -- check examples/clinic-queue.toml
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
cargo run -p neotui-cli -- run examples/rich-dashboard.toml
cargo run -p neotui-cli -- run examples/showcase-layout.toml
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
```

This sequence demonstrates:

- environment readiness without printing sensitive environment values;
- TOML DSL validation;
- basic dashboard composition;
- focusable list coverage through an official fixture;
- richer frontend composition with KPIs, graph, notes and actions;
- a richer terminal layout for the stable MVP moment;
- instrumentation widgets through cockpit and redline HUD examples;
- real-world visual hierarchy through the clinic queue example;
- final visual-system composition with reduced chrome noise and stronger hierarchy.

Before recording or sharing the demo, review `docs/visual-system.md` and the visual checklist in `docs/tui-design.md`.

## PowerShell Shortcut

```powershell
.\scripts\showcase.ps1
```

The script runs `doctor`, validates the showcase fixture and starts the terminal runtime.

## Recording Recipe

On Linux, an asciinema recording can be captured with:

```bash
asciinema rec docs/showcase.cast --command "cargo run -p neotui-cli -- run examples/visual-system-showcase.toml"
```

Recording checklist:

- use a clean terminal profile with a readable monospace font;
- resize to at least `80x24`;
- start from the repository root;
- wait for the first frame to settle;
- exit with `Ctrl+Q`;
- do not record shells containing secrets or unrelated environment output.

The `.cast` file is not required for the MVP source tree yet; this page is the reproducible recording contract.

## GUI Preview

After Linux GTK/VTE readiness is confirmed with `doctor`, the same fixture can be used for the embedded GUI path:

```bash
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml --gui
```

If GUI launch fails, keep the terminal recording as the primary MVP artifact and use the `gui_*` fields from `doctor` to diagnose the Linux desktop session.

For a staged local Linux install before recording, follow `docs/linux-install.md`.
