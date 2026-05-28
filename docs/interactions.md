# Interaction Patterns

NeoTUI interaction patterns should stay predictable in a plain terminal first. The official composed interaction fixture is `examples/interactive-flow.toml`.

## Composed Flow

The fixture combines:

- a focusable `List` for selection and scroll behavior;
- two focusable `Button` actions;
- global runtime shortcuts that keep working from any focused component;
- a static detail area that explains the selected workflow without requiring callbacks yet.

Use it when validating the MVP interaction contract:

```bash
cargo run -p neotui-cli -- check examples/interactive-flow.toml
cargo run -p neotui-cli -- run examples/interactive-flow.toml
```

## Keyboard Contract

| Input | Expected behavior |
| ----- | ----------------- |
| `Tab` | Move focus to the next focusable component. |
| `Shift+Tab` | Move focus to the previous focusable component. |
| `Up` / `Down` | Move the selected item when the list is focused. |
| `Home` / `End` | Jump to the first or last list item when the list is focused. |
| `PageUp` / `PageDown` | Move the focused list selection in larger steps. |
| `Enter` | Activate the focused button. |
| `F1` | Request help through the global runtime shortcut. |
| `Ctrl+Q` | Exit and restore the terminal. |

The terminal runtime gives initial focus to the first focusable component in depth-first order. Component text and full UI payloads are not logged by default; diagnostics may include component IDs, event kinds and runtime state only.

## Mouse And Scroll

Mouse clicks are routed through the current layout to the deepest component under the pointer. Scroll events prefer the currently focused component, so a focused list can be scrolled even when the pointer is not directly over it.

Small terminals should still preserve the interaction contract. Text may clip and layout regions may shrink, but focus changes, global shortcuts and terminal restoration must remain reliable.
