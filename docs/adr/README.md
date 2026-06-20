# Architecture Decision Records

This directory stores NeoTUI Architecture Decision Records (ADRs).

ADRs are intentionally short, dated and versioned with the repository. Use them for decisions that shape architecture, public contracts, runtime strategy, dependency boundaries or long-lived product constraints.

## Status Values

- `Proposed`: under discussion.
- `Accepted`: current project direction.
- `Superseded`: replaced by a newer ADR.
- `Deprecated`: no longer recommended, but not replaced by one specific ADR.

## Index

| ADR | Status | Decision |
| --- | ------ | -------- |
| [0001](0001-terminal-first-runtime-and-embedded-vte-gui.md) | Accepted | Terminal-first runtime with GTK/VTE embedded GUI for MVP |
| [0002](0002-core-public-api-backend-neutrality.md) | Accepted | Keep core public APIs backend-neutral |
| [0003](0003-dsl-intermediate-component-spec.md) | Accepted | Parse DSL into an intermediate `ComponentSpec` model before instantiation |
| [0004](0004-visual-system-semantic-tokens-and-panel-intents.md) | Accepted | Visual System 1.0 uses semantic tokens and panel visual intent props |
| [0005](0005-declarative-data-sources-and-http-effects.md) | Accepted | Declarative data sources and optional blocking HTTP effects |
| [0006](0006-data-lifecycle-cache-stale-and-retry.md) | Accepted | Data lifecycle cache, stale state, retry and stale-response protection |
| [0007](0007-declarative-actions-and-http-mutations.md) | Accepted | Declarative actions and HTTP mutations |
| [0008](0008-form-intent-and-action-payloads.md) | Accepted | Form intent and action payloads |
| [0009](0009-python-api-subprocess-contract.md) | Accepted | Python API supported contract is the subprocess JSON bridge; native PyO3 is optional |

## Template

```markdown
# ADR-NNNN - Title

- Status: Proposed | Accepted | Superseded | Deprecated
- Date: YYYY-MM-DD
- Supersedes: ADR-NNNN, if applicable
- Superseded by: ADR-NNNN, if applicable

## Context

What problem, constraint or opportunity forced a decision?

## Decision

What are we deciding?

## Consequences

What becomes easier, harder, explicitly allowed or explicitly avoided?
```
