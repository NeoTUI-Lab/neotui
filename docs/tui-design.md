# TUI Design Guide

NeoTUI screens should feel dense, legible and operational. A good rich TUI helps users scan state, move focus predictably and recover the terminal cleanly.

## Principles

- Put the most important state in the first visible region.
- Prefer compact hierarchy over decorative framing.
- Use panels for grouped meaning, not for every individual line.
- Keep actions close to the content they affect.
- Preserve keyboard access even when mouse support is available.
- Clip text calmly in small terminals instead of relying on perfect dimensions.

## Contrast

Use contrast to separate status, focus and structure.

| Element | Recommendation |
| ------- | -------------- |
| Normal text | Keep readable on both minimal and dark themes. |
| Focused list item | Use the selected style and the `>` marker as the primary signal. |
| Button focus | Use bold or theme focus styling; do not rely only on color. |
| Graphs | Keep values easy to compare before adding extra decoration. |
| Dividers | Use low-noise separators such as `-`, `=` or a single blank spacer. |

Avoid putting essential information only in color. Terminal color support varies, and logs must not include full UI payloads by default.

## Width And Spacing

Use `width` for stable navigation/sidebar regions and `grow` for the primary work area. Use `width_pct` when sibling regions should divide available space.

Recommended starting points:

| Region | Width guidance |
| ------ | -------------- |
| Sidebar list | 22 to 34 columns |
| Action button | 10 to 16 columns |
| KPI panel | 24 columns or one third of the row |
| Detail text | `grow = 1` inside the main area |

Use `gap = 1` for compact screens and `gap = 2` when two regions need clearer separation. Repeated gaps larger than 2 usually waste terminal space.

## Hierarchy

Start with this structure for most rich screens:

```text
Panel
  VBox
    Label       title or status line
    HBox/VBox   primary content
    HBox        actions or footer
```

The first row should identify the screen. The middle region should carry the main workflow. The last row should contain actions or concise state.

For cinematic or HUD-inspired screens, prefer the Visual System 1.0 structure:

```text
Panel variant="plain" chrome="minimal"
  VBox
    StatusStrip
    HBox
      Panel variant="data"
      Panel variant="hero"
      Panel variant="framed"
    Panel variant="data"
    StatusStrip
```

This keeps one dominant region and makes the surrounding chrome quieter than the data.

## Component Use

| Component | Use when | Avoid when |
| --------- | -------- | ---------- |
| `Panel` | A region has a name, boundary or ownership. | You only need spacing. |
| `Divider` | A visual break improves scanning. | A `gap` already makes the structure clear. |
| `List` | Users choose or scan multiple items. | The content is a paragraph or fixed label set. |
| `Graph` | Relative change matters more than exact values. | The data needs table precision. |
| `Table` | Dense comparisons need aligned columns. | A few values would scan better as metrics. |
| `Metric` | A compact value needs label, value and status. | The value must dominate the whole screen. |
| `BigMetric` | A ticket, counter or headline value must be readable at distance. | The value is secondary or needs long prose. |
| `Gauge` | Bounded progress, load or capacity should be scanned quickly. | Exact history matters more than current level. |
| `Sparkline` | A small trend matters inside a dense panel. | Axis labels, exact values or comparison precision are required. |
| `Knob` | A compact dial conveys a bounded setting or level. | A linear gauge would be clearer. |
| `StatusStrip` | A whole row should carry current state or alert context. | The message is long body copy. |
| `KeyValueRow` | Operational metadata needs aligned key/value scanning. | The value needs interaction or wrapping. |
| `Button` | A focused action can be activated with `Enter`. | The action is not wired or would imply unsupported callbacks. |
| `TextBlock` | Copy needs wrapping and multiple lines. | A short status line fits in `Label`. |

## Panel Intent

Use `variant`, `density` and `chrome` as layout language, not decoration:

| Prop | Recommendation |
| ---- | -------------- |
| `variant = "plain"` | Root shells and background bands. |
| `variant = "data"` | Calm panels for tables, metrics and telemetry. |
| `variant = "hero"` | The one region that should win the first glance. |
| `variant = "alert"` or `"danger"` | Exceptional states only. |
| `density = "compact"` | Tight dashboards and sidebars. |
| `density = "spacious"` | Hero/data panels that need breathing room. |
| `chrome = "minimal"` | Remove framing when a surrounding layout already provides structure. |
| `chrome = "technical"` or `"cinematic"` | Use for references and demos, sparingly in operational screens. |

## Review Checklist

Before recording a demo or sharing a template:

- `neotui check` passes for the DSL file.
- The screen is readable around 80x24.
- The first viewport shows the app purpose immediately.
- Focus starts on the first useful interactive control.
- `Tab`, `Shift+Tab`, arrow keys, scroll and `Enter` behave as documented when relevant.
- `Ctrl+Q` exits and restores the terminal.
- No debug output includes full labels, text blocks, environment values or serialized app specs.
- Small terminals clip text without panicking.
- Strong color is tied to state or data emphasis, not permanent decoration.
- The screen has only one hero region.

## Current Limits

The MVP now includes the first dense-data and HUD widgets, but it does not yet include forms, tabs, modals, command palettes, animation or data binding. Prefer static examples and templates until those capabilities are explicitly added.

Good next widget candidates are:

- `Input` for forms and command prompts;
- `Tabs` for multi-view tools;
- `StatusBar` for persistent runtime hints;
- `Progress` for long-running operations.

Keep those as future widgets unless a roadmap item explicitly promotes them into scope.
