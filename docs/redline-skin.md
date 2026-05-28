# Redline Skin

`redline` is the first rich NeoTUI skin. It is built for dense, cyber-technical control panels with high-contrast failure states.

## Direction

- Near-black blue background.
- Red and coral panel lines.
- Cold off-white primary text.
- Muted blue-gray secondary text.
- Small cyan accents for secondary telemetry.
- Strong red focused and selected states.

## Usage

Set the app theme in TOML or JSON:

```toml
schema_version = "0.1"
theme = "redline"
```

Run the foundation example:

```bash
cargo run -p neotui-cli -- check examples/redline-dashboard.toml
cargo run -p neotui-cli -- run examples/redline-dashboard.toml
```

Run the richer HUD references:

```bash
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
cargo run -p neotui-cli -- check examples/tron-hud.toml
cargo run -p neotui-cli -- run examples/tron-hud.toml
cargo run -p neotui-cli -- check examples/clinic-queue.toml
cargo run -p neotui-cli -- run examples/clinic-queue.toml
```

The skin currently covers `Panel`, `Label`, `TextBlock`, `Divider`, `Button`, `List`, `Graph`, `Table`, `Metric`, `Gauge`, `Sparkline`, `BigMetric`, `Knob`, `StatusStrip` and `KeyValueRow`. New rich widgets should reuse the same semantic tokens before adding widget-specific style names.

Visual System 1.0 adds semantic panel intent on top of the skin:

```toml
[root.props]
variant = "data"
density = "spacious"
chrome = "technical"
```

Use `plain`/`minimal` shells to reduce chrome noise, `data` panels for calm instrumentation, `hero` panels for one dominant signal and `alert`/`danger` only for true exceptional states.

## Token Map

| Token | Purpose |
| ----- | ------- |
| `screen.default` | Base terminal surface |
| `surface.base` | Root terminal surface |
| `surface.panel` | Standard panel surface |
| `surface.raised` | Higher-emphasis surface |
| `surface.recessed` | Quieter data surface |
| `text.default` | Normal text |
| `text.primary` | High-priority labels and headings |
| `text.muted` | Secondary telemetry copy |
| `panel.border` | Technical red panel framing |
| `panel.border.subtle` | Low-noise framing |
| `panel.border.data` | Data/instrument panel framing |
| `panel.border.alert` | High-priority framing |
| `panel.border.hero` | Dominant region framing |
| `panel.surface` | Standard panel fill |
| `panel.surface.data` | Calm data panel fill |
| `panel.surface.hero` | Dominant region fill |
| `panel.title` | Panel headings |
| `divider.default` | Dense separators |
| `button.default` | Resting command state |
| `button.focused` | Active command state |
| `list.default` | List rows |
| `list.selected` | Selected/focused data row |
| `graph.default` | Current bar graph style |
| `table.header` | Table header style |
| `table.row` | Table row style |
| `table.selected` | Selected table row style |
| `metric.default` | Compact metric block style |
| `gauge.track` | Gauge background track style |
| `gauge.filled` | Gauge filled segment style |
| `sparkline.default` | Sparkline trend style |
| `knob.default` | Dial/knob indicator style |
| `status.normal` | Neutral status strip style |
| `status.warning` | Warning status strip style |
| `status.critical` | Critical status strip style |
| `status.info` | Informational status strip style |

For composition rules and hierarchy guidance, see `docs/visual-system.md`.
