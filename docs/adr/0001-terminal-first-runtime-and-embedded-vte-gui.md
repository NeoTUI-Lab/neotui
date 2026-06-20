# ADR-0001 - Terminal-First Runtime and Embedded VTE GUI

- Status: Accepted
- Date: 2026-05-28

## Context

NeoTUI must support real terminal environments, including TTY and SSH sessions, while still offering a Linux desktop window for the MVP. The terminal path has the highest product risk because it must handle raw mode, alternate screen, resize, input, ANSI rendering and terminal cleanup correctly.

Building a native GUI renderer or WebView bridge first would add font rendering, toolkit behavior, IPC, PTY and security complexity before the core terminal model is proven.

## Decision

The MVP has one real runtime: a terminal-first ANSI runtime.

The CLI path runs directly in the user's terminal. The GUI MVP opens a Linux GTK window with a VTE embedded terminal and launches the same `neotui run <file>` flow inside it.

The core renderer emits terminal-oriented frames and ANSI output. The GUI crate remains a host for the terminal runtime, not a native GUI renderer.

## Consequences

- The terminal path is the reference implementation for MVP behavior.
- GUI mode reuses the same component tree, layout, event and renderer path.
- `neotui-core` must remain independent from GTK and VTE.
- Native GUI, WebView and xterm.js backends remain future options, not MVP scope.
- Linux GTK/VTE packaging work is isolated to the GUI binary and install docs.
