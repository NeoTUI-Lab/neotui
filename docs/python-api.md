# Python API

NeoTUI's Python package is a developer-facing entrypoint for building the same neutral app specs accepted by TOML and JSON files.

## Supported contract

The supported public contract is the **subprocess JSON bridge** (see
[ADR-0009](adr/0009-python-api-subprocess-contract.md)):

- builders serialize an `App` to the neutral spec format validated by the core
  DSL parser; this serialization is the stable contract, pinned by the shared
  `examples/python/form-intent.json` fixture;
- `check(app)` and `run(app)` route through the `neotui` CLI binary, so a
  resolvable `neotui` executable is required at call time (on `PATH`, or passed
  explicitly via `--neotui-bin`);
- the native `neotui._native` extension is an **optional acceleration**.
  `binding_available` reports its presence, but importing the package and using
  every public helper must work with `binding_available == False`. No helper may
  change behavior based on whether the native module is present.

Python callbacks remain local to the Python model and are not forwarded into the
terminal runtime in this contract.

Current scope:

- component builders for base and rich widgets;
- form declarations and `$forms.<form>.<field>` bindings;
- HTTP data source and action declarations;
- declarative action bindings such as `Button("Submit", on_click="submit_incident")`;
- `check(app)` and `run(app)` helpers that route through the existing CLI runtime.

Python callbacks remain local to the Python model. They can be invoked directly in tests and helpers, but they are not forwarded into the terminal runtime yet.

## Form Intent Example

The Python version of the form intent fixture lives at `examples/python/form_intent.py`.

Print the serialized app JSON:

```bash
PYTHONPATH=python/neotui-py/src python examples/python/form_intent.py --json
```

The expected contract fixture lives at `examples/python/form-intent.json`. It is intentionally checked by both the Python tests and the core DSL parser so changes to Python serialization cannot drift away from the runtime format.

Validate only:

```bash
PYTHONPATH=python/neotui-py/src python examples/python/form_intent.py --check-only
```

Use a prebuilt binary if you do not want `check(app)` to invoke Cargo:

```bash
PYTHONPATH=python/neotui-py/src python examples/python/form_intent.py --check-only --neotui-bin ./target/debug/neotui
```

Run the interactive example against the mock backend:

```bash
python3 scripts/mock-http-backend.py
PYTHONPATH=python/neotui-py/src python examples/python/form_intent.py
```

## Verification

Pure Python contract tests can run without building the native extension:

```bash
./scripts/test-python.sh
```

The Linux/WSL helper uses `uv` when available, otherwise it falls back to `python` or `python3`. In `--native` mode without `uv`, it creates and activates `target/python-test-venv`, then installs the required Python build/test packages there before running the native gate, avoiding system Python package writes on PEP 668 distributions.

On Windows, the repository helper runs the same pure-Python contract:

```powershell
.\scripts\test-python.ps1
```

Native extension verification requires a functional Rust/Python build environment. On Windows, install Visual Studio Build Tools with the Visual C++ linker. On Linux or WSL, use the standard Rust toolchain plus Python development headers.

```bash
./scripts/test-python.sh --native
```

The native gate builds the PyO3 extension, reruns the Python tests with the extension available, builds `neotui-cli`, and validates the Python form intent app through `neotui check`.

Or, from PowerShell in an environment with the native linker:

```powershell
.\scripts\test-python.ps1 -Native
```

The native extension is intentionally tiny today; the Python model remains usable in development through `PYTHONPATH=python/neotui-py/src`.

## Native Build Troubleshooting

On Windows with the default `x86_64-pc-windows-msvc` Rust host, `maturin` needs `link.exe`. If the native gate reports that `link.exe` is missing, install Visual Studio Build Tools with the Visual C++ workload or run the gate in WSL/Linux.

Using the GNU Rust toolchain on Windows requires MinGW binutils. If a GNU attempt reports `dlltool.exe` is missing, install a MinGW toolchain that provides `dlltool`, or prefer the supported MSVC/WSL/Linux paths.

The PowerShell helper checks these prerequisites before invoking `maturin`, so a native gate failure should point at the missing linker tool instead of a later PyO3 build error.
