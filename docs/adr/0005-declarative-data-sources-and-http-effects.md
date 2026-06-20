# ADR-0005 - Declarative Data Sources and HTTP Effects

- Status: Accepted
- Date: 2026-05-28

## Context

NeoTUI screens need intent beyond static composition: dashboards should be able to declare backend data needs and render loading, ready and error states without embedding networking behavior inside widgets.

Adding HTTP directly to widgets or the renderer would couple data access to visual primitives and make future backends harder to add.

## Decision

NeoTUI supports declarative data sources through `data.sources`.

The first effect backend is HTTP, implemented as an optional blocking worker layer using `ureq`. The terminal runtime remains synchronous and responsive: HTTP requests run on worker threads, while the event loop collects updates on ticks and input events.

Sensitive header values must use environment references. Literal secret headers are rejected by the DSL parser.

Widgets remain pure render components. Data bindings such as `text_from`, `value_from`, `items_from`, `values_from`, `rows_from` and `status_from` are resolved into normal component props before registry instantiation.

## Consequences

- HTTP is an effects/data concern, not a widget concern.
- The MVP can consume JSON backends without introducing Tokio or async runtime.
- Data source failures become renderable `error` state, not terminal crashes.
- Logs may include source id and method, but must not include headers, request bodies or response payloads by default.
- WebSocket, SSE, gRPC, multipart uploads and native async backends remain future decisions.
