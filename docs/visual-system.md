# NeoTUI Visual System 1.0

This document defines the first robust visual grammar for rich NeoTUI screens. It keeps the renderer terminal-first while making dense TUIs feel intentional, modern and readable.

## Design Rule

Use neon as information, not decoration.

Strong color should mark state, focus, danger or data emphasis. Structural chrome should usually be quieter than the content it frames.

## Hierarchy

| Level | Purpose | Typical components |
| ----- | ------- | ------------------ |
| Global status | Current app/session state | `StatusStrip` |
| Hero value | One dominant number, ticket or signal | `Panel variant="hero"`, `BigMetric` |
| Primary work area | Main entity, table or decision surface | `Panel variant="data"`, `Table`, `Metric` |
| Secondary telemetry | Supporting context | `Gauge`, `Sparkline`, `KeyValueRow` |
| Actions | Explicit commands | `Button` |

## Panel Variants

`Panel` now supports visual intent through `variant`, `density` and `chrome`.

```toml
[root.props]
variant = "data"
density = "spacious"
chrome = "technical"
```

| Prop | Values | Use |
| ---- | ------ | --- |
| `variant` | `plain`, `framed`, `data`, `alert`, `hero`, `danger`, `warning`, `success`, `info` | Semantic weight and border/surface tokens |
| `density` | `compact`, `normal`, `spacious` | Horizontal breathing room inside panels |
| `chrome` | `minimal`, `framed`, `technical`, `cinematic` | Amount and character of structural framing |

Use `plain`/`minimal` for shells and background bands. Use `data` for calm operational panels. Use `hero` for the one region that must dominate the screen. Use `alert` or `danger` sparingly.

## Token Families

The visual system is based on semantic tokens, not widget-local color guesses.

| Family | Examples |
| ------ | -------- |
| Surface | `surface.base`, `surface.panel`, `surface.raised`, `surface.recessed` |
| Border | `border.subtle`, `border.strong`, `border.alert` |
| Text | `text.primary`, `text.default`, `text.muted` |
| Accent | `accent.primary`, `accent.warning`, `accent.danger`, `accent.success` |
| Data | `data.track`, `data.fill`, `data.glow` |
| Panel | `panel.surface.data`, `panel.surface.hero`, `panel.border.data`, `panel.border.hero` |

Widget implementations should resolve semantic tokens from `Theme` first and add widget-specific tokens only when the semantic token is not enough.

## Composition Pattern

Start rich screens with this structure:

```text
Panel variant=plain chrome=minimal
  VBox
    StatusStrip
    HBox
      Panel variant=data
      Panel variant=hero
      Panel variant=framed
    Panel variant=data
    StatusStrip
```

This creates a stable reading order: status, subject, dominant signal, secondary queue, telemetry, footer.

## Reference Example

Run the visual-system showcase:

```bash
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml
```

It demonstrates controlled chrome, a single hero region, calmer data panels and semantic status color.
