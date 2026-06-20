# ADR-0007 - Declarative Actions and HTTP Mutations

- Status: Accepted
- Date: 2026-05-29

## Context

NeoTUI now has declarative HTTP data sources and a data lifecycle. A robust frontend also needs user intent: buttons and selectable widgets should emit actions without embedding backend logic inside the widget implementation.

## Decision

NeoTUI supports top-level declarative `actions`.

The first action backend is HTTP. Components bind events to action ids through props such as `on_click` and `on_select`. Widgets emit `Command::Action(id)`; the runtime owns execution, status and follow-up refresh.

An HTTP action can define method, URL, headers, body, timeout, retry and `refresh_sources`. When an action succeeds, listed data sources are refreshed through the data runtime. Actions run in worker threads so input/render stays responsive. Action lifecycle can be projected back into normal widget props through bindings such as `$actions.refresh_now.$status`.

## Consequences

- Widgets stay backend-neutral and only emit declarative action ids.
- Data refresh after mutation is explicit in DSL.
- HTTP mutation behavior shares the same safe header/body parsing and retry primitives as data sources.
- Optimistic update, forms and payload templating remain future extensions.
