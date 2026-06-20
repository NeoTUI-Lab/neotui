# ADR-0003 - DSL Intermediate ComponentSpec

- Status: Accepted
- Date: 2026-05-28

## Context

NeoTUI supports declarative app definitions through TOML, JSON and eventually YAML or Python-generated specs. Directly instantiating widgets from parser-specific structures would mix parsing, validation and runtime construction, making error reporting weaker and format support harder to maintain.

The DSL also needs a stable validation surface for `neotui check`.

## Decision

All supported DSL formats parse into a neutral intermediate model before component instantiation.

The core model is:

```rust
pub struct AppSpec {
    pub schema_version: String,
    pub theme: Option<String>,
    pub root: ComponentSpec,
}

pub struct ComponentSpec {
    pub kind: String,
    pub id: Option<String>,
    pub props: Map<String, Value>,
    pub children: Vec<ComponentSpec>,
}
```

Parsing, validation and registry instantiation remain separate steps.

## Consequences

- `neotui check` can validate schema, component names, required props, prop types and child rules before runtime execution.
- TOML and JSON stay canonical core formats.
- YAML can be isolated later without making YAML parsing a fragile core dependency.
- Registry code receives one normalized component representation regardless of source format.
- Error messages can point to DSL paths such as `root.props.variant`.
