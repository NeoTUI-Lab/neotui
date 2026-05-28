# NeoTUI Manual Release

This is the MVP manual release checklist for Linux builds. It is not a public release automation contract yet; it defines how to validate and stage a local Linux artifact before sharing it for review.

## Release Boundary

Current release status:

- supported artifact: staged Linux directory from `./scripts/package-linux.sh`;
- included binaries: `neotui` and `neotui-gui`;
- target platform: Ubuntu LTS or compatible Linux environment;
- package formats: `.deb` and AppImage are not produced by this checklist yet;
- Windows/macOS installers are outside MVP scope.

Treat `dist/neotui-linux` as an experimental MVP artifact, not as a stable distribution format.

## Preconditions

Run on Linux from the repository root:

```bash
rustc --version
cargo --version
pkg-config --modversion gtk4
pkg-config --modversion vte-2.91-gtk4
cargo metadata --format-version 1 --no-deps
```

Expected outcome:

- Rust stable is active.
- GTK4 and VTE development packages resolve through `pkg-config`.
- Cargo metadata lists `neotui-core`, `neotui-cli` and `neotui-gui`.

## Verification Checklist

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p neotui-core --test benchmarks -- --ignored --nocapture
```

Then validate examples:

```bash
cargo run -p neotui-cli -- check examples/hello.toml
cargo run -p neotui-cli -- check examples/dashboard.toml
cargo run -p neotui-cli -- check examples/dashboard.json
cargo run -p neotui-cli -- check examples/layout-demo.toml
cargo run -p neotui-cli -- check examples/layout-dense.toml
cargo run -p neotui-cli -- check examples/layout-sidebar.toml
cargo run -p neotui-cli -- check examples/layout-responsive.toml
cargo run -p neotui-cli -- check examples/interactive-flow.toml
cargo run -p neotui-cli -- check examples/list-demo.toml
cargo run -p neotui-cli -- check examples/rich-dashboard.toml
cargo run -p neotui-cli -- check examples/theme-demo.toml
cargo run -p neotui-cli -- check examples/showcase-layout.toml
cargo run -p neotui-cli -- check templates/operational-dashboard.toml
cargo run -p neotui-cli -- check templates/task-list.toml
cargo run -p neotui-cli -- check templates/metrics-monitor.toml
```

Manual runtime smoke checks:

```bash
cargo run -p neotui-cli -- doctor
cargo run -p neotui-cli -- run examples/hello.toml
cargo run -p neotui-cli -- run examples/interactive-flow.toml
cargo run -p neotui-cli -- run examples/showcase-layout.toml
```

Expected outcome:

- `doctor` reports terminal readiness or actionable degraded-state hints.
- `run` enters alternate screen and exits with `Ctrl+Q`.
- Terminal state is restored after exit.

GUI smoke check, when a Linux graphical session is available:

```bash
cargo run -p neotui-cli -- run examples/dashboard.toml --gui
```

## Stage the Artifact

```bash
./scripts/package-linux.sh
```

Expected staged files:

- `dist/neotui-linux/bin/neotui`
- `dist/neotui-linux/bin/neotui-gui`
- `dist/neotui-linux/VERSION.txt`
- `dist/neotui-linux/cargo-metadata.json`
- `dist/neotui-linux/RELEASE-CHECKLIST.md`

Validate the staged binaries:

```bash
dist/neotui-linux/bin/neotui --version
dist/neotui-linux/bin/neotui doctor
dist/neotui-linux/bin/neotui check examples/interactive-flow.toml
dist/neotui-linux/bin/neotui check examples/showcase-layout.toml
dist/neotui-linux/bin/neotui check templates/operational-dashboard.toml
dist/neotui-linux/bin/neotui run examples/showcase-layout.toml
```

## Sharing the Artifact

For MVP review, share the staged directory as a compressed archive:

```bash
tar -C dist -czf dist/neotui-linux.tar.gz neotui-linux
```

Before sharing, include:

- the git commit SHA used for the build;
- the Linux distribution and version;
- Rust and Cargo versions;
- whether GUI smoke testing passed;
- whether the visual demo passed the `docs/tui-design.md` review checklist;
- known limitations from `doctor` or the release checklist.

## Known Experimental Areas

- `.deb` packaging is not implemented yet.
- AppImage packaging is not implemented yet.
- GUI launch depends on local GTK/VTE runtime and graphical-session readiness.
- The staged archive is intended for developer/MVP validation only.
