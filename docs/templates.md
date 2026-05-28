# Application Templates

NeoTUI templates are executable DSL files meant to be copied and adapted into new terminal apps. They are intentionally static for the MVP, but each one uses the same component and layout contracts as normal examples.

## Template Index

| File | Use when | Main components |
| ---- | -------- | --------------- |
| `templates/operational-dashboard.toml` | You need a service overview with queue, graph and action row | `Panel`, `VBox`, `HBox`, `Label`, `List`, `Graph`, `Button` |
| `templates/task-list.toml` | You need a focused task queue with detail copy and actions | `Panel`, `VBox`, `HBox`, `Label`, `List`, `TextBlock`, `Button` |
| `templates/metrics-monitor.toml` | You need KPI cards and a larger metric trend area | `Panel`, `VBox`, `HBox`, `Label`, `Graph`, `TextBlock` |

## Validate

Run these checks before adapting the files:

```bash
cargo run -p neotui-cli -- check templates/operational-dashboard.toml
cargo run -p neotui-cli -- check templates/task-list.toml
cargo run -p neotui-cli -- check templates/metrics-monitor.toml
```

Each command should print `check ok` with component counts, layout props and component IDs.

## Run

Run a template from an interactive terminal:

```bash
cargo run -p neotui-cli -- run templates/operational-dashboard.toml
```

Use `Ctrl+Q` to exit and restore the terminal. Templates with lists and buttons also support the interaction contract described in `docs/interactions.md`.

## Adapt

1. Copy one template file into your app or examples directory.
2. Change the root `id` and visible titles first.
3. Replace static `items`, `text` and `values` with your domain content.
4. Keep `Panel`, `VBox` and `HBox` wrappers while changing leaf widgets; this preserves predictable layout behavior.
5. Re-run `neotui check` after each structural change.

Use `templates/operational-dashboard.toml` for dense operations screens, `templates/task-list.toml` for interactive queue workflows and `templates/metrics-monitor.toml` for metric-heavy status monitors.

For visual quality checks and component selection guidance, see `docs/tui-design.md`.
