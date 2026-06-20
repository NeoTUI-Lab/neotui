# ADR-0008 - Form Intent and Action Payloads

- Status: Accepted
- Date: 2026-05-29

## Context

NeoTUI can now read remote data and execute declarative actions. That makes dashboards reactive, but user-entered values are still missing from the frontend model: an action body is currently static, and widgets do not have a backend-neutral way to update form state.

The next product step should preserve the same architecture used for data and actions: widgets express intent, runtime state owns lifecycle, and HTTP remains an optional backend effect.

## Decision

NeoTUI will add a declarative form state layer before expanding action payloads further.

Forms are represented as backend-neutral runtime state, addressable by stable binding paths such as `$forms.<form_id>.<field_id>`. Input widgets update form state through NeoTUI commands rather than directly mutating HTTP actions or data sources.

HTTP actions may render request bodies from form state, but payload rendering belongs to the action runtime boundary. Widgets remain pure UI components that know about focus, editing and intent, not network effects.

Validation is part of the form lifecycle. Required fields and simple validation errors should be exposed as bindable state so ordinary widgets can present feedback without special-case rendering logic.

## Consequences

- Forms become the bridge between human input and declarative actions.
- Existing `Command::Action(id)` remains valid; submit gating can happen before action execution.
- HTTP body templating can be tested without running a terminal UI.
- Secrets remain protected because environment-backed headers are not expanded into visible bindings or default logs.
- More advanced form features such as nested schemas, async validation, file uploads and multipart requests stay out of scope for the first implementation.
