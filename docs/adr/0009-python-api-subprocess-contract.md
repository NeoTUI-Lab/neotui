# ADR-0009 - Python API Subprocess Contract

- Status: Accepted
- Date: 2026-06-20

## Context

The Python package (`python/neotui-py`) exposes builders for the current DSL and the helpers `check(app)` and `run(app)`. Two delivery mechanisms exist in the tree:

1. a pure-Python model that serializes an `App` to the same neutral JSON the core accepts and drives the `neotui` CLI through `subprocess`;
2. an optional native extension (`neotui._native`) built with PyO3/maturin, imported best-effort and currently intentionally tiny.

Production readiness needs a single, stated public contract so consumers know what is supported, what is optional, and what guarantees the API makes. Without that, `run(app)` semantics, packaging requirements and the meaning of `binding_available` are ambiguous.

## Decision

The supported public contract for the Python API is the **subprocess JSON bridge**:

- `App` and its builders serialize to the neutral spec format validated by the core DSL parser. This serialization is the stable contract and is pinned by the shared `examples/python/form-intent.json` fixture, checked by both Python tests and the Rust parser.
- `check(app)` and `run(app)` route through the `neotui` CLI binary. The Python package therefore requires a resolvable `neotui` executable (on `PATH`, or passed explicitly) at call time, and does not embed the runtime.
- The native `neotui._native` extension is an **optional acceleration**, not a requirement. `binding_available` reports its presence, but no public helper may depend on it being present, and importing the package must always succeed without it.

Python callbacks remain local to the Python model and are not forwarded into the terminal runtime in this contract.

## Consequences

- The Python package is usable in development via `PYTHONPATH` and in distribution without compiling Rust, lowering the adoption barrier on machines without a native toolchain.
- Packaging stays simple: a pure-Python wheel plus a documented dependency on the `neotui` binary; no manylinux/PyO3 build matrix is required for the supported path.
- The cost is process-spawn latency per `check`/`run` and the requirement that the CLI binary be installed alongside the Python package. This is acceptable for the operational/dashboard use cases NeoTUI targets.
- A future native, in-process runtime binding can supersede this ADR, but until then no API may silently change behavior based on whether `_native` is available.
- `binding_available == False` is a normal, supported state and must not be treated as an error by callers or tests.
