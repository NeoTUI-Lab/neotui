# NeoTUI Layout Patterns

This guide captures the first practical layout patterns for building richer terminal frontends with the current MVP components.

## Building Blocks

Use these components as layout primitives:

- `VBox` for vertical page flow.
- `HBox` for horizontal rows, sidebars and metric strips.
- `Panel` for framed regions with a title.
- `Divider` for visual separation inside a flow.
- `Spacer` for intentional empty space.

Leaf widgets such as `Label`, `TextBlock`, `List`, `Graph` and `Button` should sit inside those layout primitives.

## Constraints

The DSL supports a small set of layout hints on components:

| Prop | Use |
| ---- | --- |
| `width` | Reserve a fixed column width for a child. |
| `height` | Reserve a fixed row height for a child. |
| `width_pct` | Reserve a percentage of the parent width. |
| `height_pct` | Reserve a percentage of the parent height. |
| `grow` | Give remaining space to a child in stack layouts. |
| `gap` | Add fixed spacing between stack children. |
| `align` | Cross-axis alignment for stack children, or text alignment for `Label`. |
| `justify` | Main-axis placement for stack children. |

Prefer fixed sizes for toolbars, footers, buttons and metric tiles. Use `grow` for the main content region. Use percentages sparingly, mainly when a sidebar or split pane should stay proportional.

## Header Body Footer

Use a root `VBox` or a `Panel` containing a `VBox`:

```text
Panel
  VBox
    Label      fixed height header
    Divider    fixed height separator
    HBox       growing body
    HBox       fixed height action/footer row
```

Official fixture: `examples/layout-dense.toml`.

## Sidebar Layout

Use `HBox` for the main body. Give the sidebar a fixed `width`, then give the content panel `grow = 1`.

```text
Panel
  HBox
    Panel      fixed sidebar
      List
    Panel      grow content
      TextBlock or Graph
```

Official fixture: `examples/layout-sidebar.toml`.

## Responsive Minimum

For a layout that degrades better in narrow terminals:

- keep the header and footer short;
- avoid long unbroken text;
- give fixed widths only to controls that need them;
- use `grow` for the primary content;
- accept clipping over panic.

Official fixture: `examples/layout-responsive.toml`.

## Small Terminal Guidance

When targeting compact terminals:

- keep panel titles short;
- prefer one strong body region over many tiny columns;
- use `Divider` only when it clarifies scanning;
- keep button text short enough for the fixed width;
- test at `80x24`, then inspect at smaller sizes such as `60x18`.

## Pattern Checklist

Before adding a rich TUI screen:

- Validate with `neotui check`.
- Confirm every fixed width has a reason.
- Confirm one region can absorb remaining space with `grow`.
- Confirm the screen still communicates when labels are clipped.
- Run from an interactive terminal and exit with `Ctrl+Q`.

