# NeoTUI Linux Install

This page documents the initial local Linux installation path for the MVP. It is intentionally simple: build release binaries, stage them in a local directory and optionally copy them into a user-owned prefix.

## Supported Target

Reference target:

- Ubuntu LTS or compatible Linux distribution
- Rust stable toolchain
- interactive terminal for `neotui run`
- GTK4/VTE development packages for building the GUI binary
- graphical Linux session for `neotui run <file> --gui`

Windows and macOS packaging are outside the MVP scope.

## Prerequisites

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev libvte-2.91-gtk4-dev
rustc --version
cargo --version
pkg-config --modversion gtk4
pkg-config --modversion vte-2.91-gtk4
```

Expected outcome:

- Rust and Cargo print stable toolchain versions.
- `pkg-config` can resolve GTK4 and VTE.

## Build Release Binaries

From the repository root:

```bash
cargo build --workspace --release
```

Expected output files:

- `target/release/neotui`
- `target/release/neotui-gui`

Validate the built CLI:

```bash
target/release/neotui doctor
target/release/neotui check examples/hello.toml
target/release/neotui run examples/hello.toml
```

Press `Ctrl+Q` to exit `run`.

## Stage a Local Install

Use the package helper to build release binaries and stage them under `dist/neotui-linux/bin`:

```bash
./scripts/package-linux.sh
```

Expected staged files:

- `dist/neotui-linux/bin/neotui`
- `dist/neotui-linux/bin/neotui-gui`

The helper also writes `dist/neotui-linux/VERSION.txt` with the staged version.

## Optional User Install

Copy the staged binaries into a user-owned prefix:

```bash
mkdir -p "$HOME/.local/bin"
cp dist/neotui-linux/bin/neotui "$HOME/.local/bin/neotui"
cp dist/neotui-linux/bin/neotui-gui "$HOME/.local/bin/neotui-gui"
```

Make sure the prefix is on `PATH`:

```bash
echo "$PATH" | tr ':' '\n' | grep -x "$HOME/.local/bin"
```

Then validate:

```bash
neotui doctor
neotui check examples/dashboard.toml
neotui run examples/dashboard.toml
```

## GUI Validation

The GUI MVP requires a Linux graphical session with GTK/VTE available:

```bash
neotui doctor
neotui run examples/dashboard.toml --gui
```

Expected outcome:

- `doctor` reports GUI support as ready or explains the missing session/prerequisite.
- `--gui` opens the embedded terminal window and runs the same terminal app.

## Current Packaging Boundary

The current MVP installation path is a local release build plus staged binaries. The manual release checklist lives in `docs/release.md`.

Do not publish the staged directory as a stable distribution format yet; it is a developer-facing install artifact for MVP validation.
