## NeoTUI Python Package

Minimal package bootstrap for NeoTUI Python bindings.

Current scope:

- installable `neotui` package metadata
- importable `neotui` module from `src/`
- optional native extension hook via PyO3 and maturin
- tiny package-side `doctor()` helper for smoke validation

The native runtime bindings are intentionally small at this stage and will grow in the `US-010.x` tasks.
