#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist/neotui-linux"
BIN_DIR="${DIST_DIR}/bin"

cd "${ROOT_DIR}"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "NeoTUI Linux packaging must run on Linux." >&2
  exit 1
fi

echo "Building NeoTUI release binaries..."
cargo build --workspace --release

echo "Staging binaries in ${BIN_DIR}..."
rm -rf "${DIST_DIR}"
mkdir -p "${BIN_DIR}"

install -m 0755 "${ROOT_DIR}/target/release/neotui" "${BIN_DIR}/neotui"
install -m 0755 "${ROOT_DIR}/target/release/neotui-gui" "${BIN_DIR}/neotui-gui"

cargo metadata --format-version 1 --no-deps > "${DIST_DIR}/cargo-metadata.json"
"${BIN_DIR}/neotui" --version > "${DIST_DIR}/VERSION.txt"

cat > "${DIST_DIR}/RELEASE-CHECKLIST.md" <<'CHECKLIST'
# NeoTUI Staged Release Checklist

- [ ] Built on Linux.
- [ ] `cargo fmt --check` passed.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passed.
- [ ] `cargo test --workspace` passed.
- [ ] Official examples passed `neotui check`.
- [ ] Layout pattern examples passed `neotui check`.
- [ ] `examples/interactive-flow.toml` passed `neotui check`.
- [ ] `examples/rich-dashboard.toml` passed `neotui check`.
- [ ] Application templates passed `neotui check`.
- [ ] `neotui doctor` output reviewed.
- [ ] `neotui run examples/hello.toml` restored terminal after `Ctrl+Q`.
- [ ] `neotui run examples/interactive-flow.toml` restored terminal after `Ctrl+Q`.
- [ ] `neotui run examples/showcase-layout.toml` restored terminal after `Ctrl+Q`.
- [ ] Visual demo reviewed with `docs/tui-design.md`.
- [ ] `neotui run examples/dashboard.toml --gui` tested or documented as skipped.
- [ ] Known limitations recorded before sharing the artifact.

See `docs/release.md` for the full manual release process.
CHECKLIST

cat <<EOF
NeoTUI staged successfully.

Staged files:
  ${BIN_DIR}/neotui
  ${BIN_DIR}/neotui-gui
  ${DIST_DIR}/RELEASE-CHECKLIST.md

Validate with:
  ${BIN_DIR}/neotui doctor
  ${BIN_DIR}/neotui check examples/hello.toml
  ${BIN_DIR}/neotui run examples/hello.toml

Release checklist:
  ${DIST_DIR}/RELEASE-CHECKLIST.md

Optional user install:
  mkdir -p "\$HOME/.local/bin"
  cp "${BIN_DIR}/neotui" "\$HOME/.local/bin/neotui"
  cp "${BIN_DIR}/neotui-gui" "\$HOME/.local/bin/neotui-gui"
EOF
