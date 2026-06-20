# ADR-0006 - Data Lifecycle Cache, Stale State and Retry

- Status: Accepted
- Date: 2026-05-29

## Context

HTTP data sources need frontend-grade lifecycle behavior. A TUI should not freeze old requests into the UI, block input while loading, or erase the last useful value when a refresh fails.

## Decision

NeoTUI data sources keep a small runtime lifecycle:

- initial fetch publishes `loading`;
- refresh with an existing value publishes `stale` while preserving cached data;
- successful responses publish `ready`;
- failed responses publish `error` and keep the last cached value when available;
- `retry_count` and `retry_backoff_ms` are handled inside the worker thread;
- every request has a generation id, and late responses from older generations are ignored.

Widget bindings still resolve into ordinary props before component instantiation. When binding to a visual `status` prop, lifecycle states map to visual statuses: `ready -> success`, `loading/idle -> info`, `stale -> warning`, and `error -> danger`.

## Consequences

- Refresh failures do not blank dashboards that already had valid data.
- Slow or late responses cannot overwrite fresher data.
- Input/render remains non-blocking because retry/backoff happens in worker threads.
- Data lifecycle is now richer than raw HTTP success/failure, but still independent from widget implementations.
