# ADR-0002 - Core Public API Backend Neutrality

- Status: Accepted
- Date: 2026-05-28

## Context

NeoTUI uses implementation libraries such as Crossterm, GTK/VTE and possibly Ratatui-inspired internals to reduce MVP risk. Those libraries are useful, but exposing them directly in public NeoTUI APIs would make the framework hard to evolve toward future terminal, GUI, web or Python-facing runtimes.

The project needs its own stable vocabulary for components, events, layout, rendering and themes.

## Decision

NeoTUI public and core-facing APIs use NeoTUI-owned types for architectural concepts.

Examples include `Component`, `Event`, `EventResult`, `Frame`, `ScreenBuffer`, `Cell`, `Style`, `Color`, `Rect`, `Constraint`, `LayoutNode`, `Theme` and `ComponentSpec`.

Third-party backend types may be used internally behind adapters, but they should not leak into `neotui-core` public contracts.

## Consequences

- Backend dependencies can be replaced or isolated without breaking core users.
- The MVP can use Crossterm and GTK/VTE pragmatically while keeping future backend options open.
- New dependencies require extra scrutiny if they would appear in public APIs.
- Adapter code may be slightly more verbose, but architectural ownership stays with NeoTUI.
