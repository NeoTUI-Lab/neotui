#!/usr/bin/env bash
set -euo pipefail

# Build a Debian (.deb) package for NeoTUI using cargo-deb.
#
# This produces an installable artifact (CLI + GUI launcher) for Ubuntu/Debian,
# superseding the staged-directory-only flow in scripts/package-linux.sh.
#
# Prerequisites:
#   - Linux with the Rust stable toolchain.
#   - GTK4/VTE development packages (libgtk-4-dev libvte-2.91-gtk4-dev).
#   - cargo-deb (installed automatically below if missing).

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "NeoTUI .deb packaging must run on Linux." >&2
  exit 1
fi

if ! command -v cargo-deb >/dev/null 2>&1; then
  echo "cargo-deb not found; installing it with cargo install cargo-deb..."
  cargo install cargo-deb --locked
fi

echo "Building NeoTUI release binaries (workspace)..."
cargo build --workspace --release

echo "Building .deb package for the neotui-cli package..."
# --no-build: we built the whole workspace above so both binaries (neotui and
# neotui-gui) exist in target/release before cargo-deb collects the assets.
cargo deb --package neotui-cli --no-build

DEB_PATH="$(ls -t target/debian/*.deb 2>/dev/null | head -n1 || true)"
if [[ -z "${DEB_PATH}" ]]; then
  echo "Expected a .deb under target/debian/ but found none." >&2
  exit 1
fi

cat <<EOF
NeoTUI Debian package built successfully.

Artifact:
  ${DEB_PATH}

Inspect:
  dpkg-deb --info "${DEB_PATH}"
  dpkg-deb --contents "${DEB_PATH}"

Install locally:
  sudo apt install "./${DEB_PATH}"

After install, validate:
  neotui --version
  neotui doctor
  neotui check examples/hello.toml

Note: the package license is currently the placeholder
"LicenseRef-Proprietary" in crates/neotui-cli/Cargo.toml. Set the real license
before distributing the .deb publicly. See docs/release.md.
EOF
