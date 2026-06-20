# ADR-0004 - Visual System Semantic Tokens and Panel Intents

- Status: Accepted
- Date: 2026-05-28

## Context

EPIC-022 introduced Visual System TUI 1.0 to make rich NeoTUI screens calmer, more readable and more reusable. Before this, visual richness risked becoming widget-local color choices and excessive chrome.

The project needs visual primitives that work in terminal constraints while still supporting modern dashboards, operational displays and demos.

## Decision

NeoTUI Visual System 1.0 is based on semantic theme tokens and explicit panel intent.

Themes expose semantic token families such as surface, border, text, accent, data and panel tokens. Widgets resolve semantic tokens from `Theme` first and only add widget-specific tokens when the semantic vocabulary is not enough.

`Panel` supports visual intent through:

- `variant`: semantic weight, such as `plain`, `framed`, `data`, `alert`, `hero`, `danger`, `warning`, `success` or `info`.
- `density`: content breathing room, such as `compact`, `normal` or `spacious`.
- `chrome`: structural framing level, such as `minimal`, `framed`, `technical` or `cinematic`.

## Consequences

- Visual hierarchy becomes part of the DSL and component contract, not a one-off showcase style.
- Strong color is reserved for state, focus, danger and data emphasis.
- Root shells can use `plain` and `minimal` to reduce visual noise.
- `hero` should be used sparingly for the one region that must dominate the first glance.
- Future themes can change appearance while preserving semantic intent.
