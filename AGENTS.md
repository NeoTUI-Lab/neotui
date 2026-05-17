# AGENTS.md — NeoTUI

## Last Executed Task

**TASK-009.2.8** - Destacar componentes mais recorrentes no `check`
- Status: Completed
- Date: 2026-05-17
- Summary: Extended successful `neotui check` output with a compact dominant-component summary so the command now highlights the main composition pattern of each DSL fixture at a glance.

---

## Next Task

**TASK-009.2.9** - Sinalizar distribuição estrutural compacta no `check`
- Status: Pending
- Priority: P0
- Depends on: TASK-009.2.8
- Date: 2026-05-17
- Summary: Extend the success-side summary with one more compact structural hint, such as balance between containers and leaves or dominant layout traits, without turning `check` into a verbose inspection dump.

### Objective

Configure the root Rust workspace so all MVP crates are managed from a single `Cargo.toml`.

### Scope

- Create/update root `Cargo.toml` with:
  - `[workspace]`
  - `resolver = "2"`
  - members:
    - `crates/neotui-core`
    - `crates/neotui-cli`
    - `crates/neotui-gui`

### Acceptance Criteria

1. `Cargo.toml` exists at repository root with valid workspace syntax.
2. `cargo metadata` runs successfully from repository root.
3. Workspace members resolve without path errors.
4. No GUI/Python dependency is required by workspace root itself.

### Out of Scope

- Implementing runtime features.
- Adding non-essential dependencies.
- Restructuring directories beyond workspace membership.

### Environment Recovery (Windows) (Reference)

Run these commands in PowerShell to install Rust/Cargo and revalidate the task:

```powershell
# 1) Install Rust toolchain (includes cargo)
winget install --id Rustlang.Rustup -e

# 2) Restart terminal (or open a new PowerShell session), then verify
rustc --version
cargo --version

# 3) Revalidate workspace from repository root
cd C:\dev\neotui
cargo metadata --format-version 1 --no-deps
```

If `winget` is unavailable, install from:

- https://rustup.rs/

Then run:

```powershell
rustup default stable
cd C:\dev\neotui
cargo metadata --format-version 1 --no-deps
```

---

## 1. Mission

NeoTUI is a modern reactive, declarative and extensible TUI framework.

Its goal is to allow developers to build rich terminal user interfaces that can run in:

1. a real terminal, including TTY and SSH sessions;
2. a Linux desktop window using an embedded terminal;
3. future desktop/web/cross-platform runtimes without rewriting the application model.

The MVP is terminal-first.

The GUI mode for the MVP must reuse the terminal renderer through an embedded terminal window. Do not implement a native GUI renderer in the MVP.

---

## 2. Product Direction

NeoTUI should feel like a lightweight “React for the terminal”, but without copying React internals unnecessarily.

The framework must provide:

- declarative component composition;
- reactive state/event flow;
- terminal rendering with ANSI;
- keyboard, mouse, scroll, resize and focus support;
- themes and style tokens;
- a YAML/TOML/JSON DSL;
- a Python API;
- extensibility for future custom widgets/plugins;
- predictable behavior in terminal environments;
- safe logs without leaking sensitive data.

The MVP must optimize for:

1. minimum technical risk;
2. working terminal runtime;
3. strong demo value;
4. clean architecture for future backends;
5. small but reliable component set.

---

## 3. Architectural Decision

The chosen MVP architecture is:

Terminal-first runtime + GUI as embedded terminal.

That means:

- the core renderer outputs ANSI;
- the same runtime runs in terminal and GUI mode;
- GUI mode launches a desktop window with an embedded terminal widget;
- the core must remain independent from GTK/VTE;
- future native GUI or WebView backends must not be blocked by MVP decisions.

The core architecture should follow this conceptual pipeline:

```text
AppSpec / Python API
        │
        ▼
ComponentTree
        │
        ▼
StateStore + EventLoop
        │
        ▼
LayoutEngine
        │
        ▼
FrameBuffer
        │
        ▼
ANSI Renderer
        │
        ▼
Terminal / SSH / Embedded VTE Window
```

Do not couple public NeoTUI APIs directly to Crossterm, Ratatui, GTK, VTE or any other third-party library.

Third-party libraries are implementation details unless explicitly approved.

---

## 4. Locked Stack for MVP

Use the following stack unless a task explicitly says otherwise.

### Core

- Language: Rust
- Edition: Rust 2021
- Rust channel: stable
- Crate: `neotui-core`
- Responsibilities:

  - component model;
  - layout engine;
  - event model;
  - render buffer;
  - ANSI renderer;
  - themes;
  - DSL model;
  - testing helpers.

### Terminal Backend

- Primary terminal library: `crossterm`
- Responsibilities:

  - raw mode;
  - alternate screen;
  - keyboard events;
  - mouse events;
  - resize events;
  - terminal cleanup.

### Rendering Internals

Ratatui may be used as internal inspiration or helper only when it reduces risk.

Rules:

- Do not expose Ratatui types in NeoTUI public APIs.
- Do not make NeoTUI merely a wrapper around Ratatui.
- Prefer NeoTUI-owned abstractions: `Frame`, `Cell`, `Style`, `Component`, `Event`.

### GUI MVP

- Toolkit: GTK
- Embedded terminal: VTE
- Crate/binary: `neotui-gui`
- Responsibility:

  - open a Linux desktop window;
  - host an embedded terminal;
  - spawn `neotui run <file>` inside the embedded terminal;
  - support `neotui run <file> --gui`.

Do not implement a native GUI renderer in MVP.

### CLI

- Crate: `neotui-cli`
- CLI library: `clap`
- Required commands:

  - `neotui run <file>`;
  - `neotui run <file> --gui`;
  - `neotui check <file>`;
  - `neotui doctor`;
  - `neotui help`.

### Python Bindings

- Binding technology: PyO3
- Build tool: maturin
- Python version: 3.8+
- Package location: `python/neotui-py`
- Public module: `neotui`

### DSL

Supported MVP formats:

- TOML;
- JSON;
- YAML if safely isolated.

Important rule:

TOML/JSON should be preferred as canonical core formats.

YAML support may be implemented through a Python-side loader or isolated parser. Do not make YAML parsing a fragile hard dependency of the core if it creates maintenance risk.

### Packaging

MVP target:

- Linux Ubuntu;
- local release build;
- experimental `.deb`;
- AppImage only if low-risk.

Windows, macOS, `.exe`, `.msi`, `.dmg`, Tauri and WebView are not MVP scope.

---

## 5. Repository Structure

Use this structure:

```text
.
├── AGENTS.md
├── README.md
├── Cargo.toml
├── crates/
│   ├── neotui-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── component/
│   │       ├── runtime/
│   │       ├── event/
│   │       ├── layout/
│   │       ├── render/
│   │       ├── style/
│   │       ├── theme/
│   │       ├── dsl/
│   │       ├── registry/
│   │       ├── widgets/
│   │       └── testing/
│   ├── neotui-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── neotui-gui/
│       ├── Cargo.toml
│       └── src/
├── python/
│   └── neotui-py/
│       ├── pyproject.toml
│       ├── README.md
│       ├── src/
│       │   └── neotui/
│       └── tests/
├── examples/
│   ├── hello.toml
│   ├── dashboard.toml
│   ├── dashboard.yaml
│   ├── list-demo.toml
│   ├── theme-demo.toml
│   └── python/
├── docs/
│   ├── quickstart.md
│   ├── architecture.md
│   ├── dsl.md
│   ├── widget-authoring.md
│   └── roadmap.md
├── scripts/
│   ├── dev-check.sh
│   └── package-linux.sh
└── .github/
    └── workflows/
        └── ci.yml
```

If the repository already has a different structure, adapt minimally and preserve existing working code.

Do not perform large reorganizations unless the task explicitly asks for it.

---

## 6. Core Concepts

### 6.1 Component

Every widget or layout element must be modeled as a component.

A component should support:

- identity;
- layout calculation;
- rendering;
- event handling;
- optional internal state.

Preferred conceptual contract:

```rust
pub trait Component {
    fn id(&self) -> ComponentId;

    fn layout(&self, ctx: &LayoutContext, area: Rect) -> LayoutNode;

    fn render(&self, ctx: &RenderContext, frame: &mut Frame);

    fn on_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult;
}
```

Do not over-engineer the trait prematurely. If Rust object-safety or ownership issues appear, prefer a simpler working design and document the trade-off.

### 6.2 Event

Events must be normalized into NeoTUI-owned types.

Required MVP events:

- key;
- mouse click;
- mouse scroll;
- resize;
- focus gained;
- focus lost;
- tick;
- quit requested;
- help requested.

Preferred conceptual model:

```rust
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Scroll(ScrollEvent),
    Resize { width: u16, height: u16 },
    FocusGained(ComponentId),
    FocusLost(ComponentId),
    Tick,
    QuitRequested,
    HelpRequested,
}
```

### 6.3 Event Result

Event handling must indicate whether the event was ignored, consumed or produced a command.

Preferred conceptual model:

```rust
pub enum EventResult {
    Ignored,
    Consumed,
    RequestRender,
    Command(Command),
}
```

### 6.4 Render Model

Rendering must happen through an intermediate frame buffer.

Required types:

- `Frame`;
- `ScreenBuffer`;
- `Cell`;
- `Style`;
- `Color`;
- `DirtyRegion` or equivalent future hook.

Do not write directly to stdout from individual components.

Only the renderer/backend should flush output.

### 6.5 Layout Model

Layout must use NeoTUI-owned geometry types.

Required types:

- `Rect`;
- `Size`;
- `Position`;
- `Constraint`;
- `LayoutNode`.

Required layout components:

- `VBox`;
- `HBox`;
- `Panel`;
- `Spacer`;
- `Divider`.

### 6.6 DSL Model

The DSL must be converted to a neutral intermediate model before component instantiation.

Preferred conceptual model:

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

DSL parsing and component instantiation must remain separate.

Validation must happen before runtime execution.

---

## 7. MVP Components

Implement the following components first:

### Layout Components

- `VBox`
- `HBox`
- `Panel`
- `Spacer`
- `Divider`

### Widgets

- `Label`
- `TextBlock`
- `Button`
- `List`
- `Graph`

Do not implement advanced widgets before the MVP components are stable.

Avoid adding modals, command palette, forms, tables, tabs, animations or plugin widgets before the MVP baseline is complete.

---

## 8. Required CLI Behavior

### 8.1 `neotui run <file>`

Must:

1. read the file;
2. detect format;
3. parse into `AppSpec`;
4. validate the spec;
5. instantiate `ComponentTree`;
6. start terminal runtime;
7. restore terminal on exit.

### 8.2 `neotui run <file> --gui`

Must:

1. validate that GUI support is available;
2. launch `neotui-gui`;
3. open a desktop window;
4. embed a terminal;
5. run the same app inside the embedded terminal.

### 8.3 `neotui check <file>`

Must:

1. parse the file;
2. validate schema;
3. validate component names;
4. validate required properties;
5. validate property types;
6. return non-zero exit code for invalid files;
7. print actionable errors.

### 8.4 `neotui doctor`

Must check:

- terminal dimensions;
- terminal type;
- color capability if detectable;
- mouse support if detectable;
- GUI dependencies when relevant;
- debug configuration.

Do not print sensitive environment variable values.

---

## 9. Coding Standards

### Rust

Use:

- Rust stable;
- edition 2021;
- `cargo fmt`;
- `cargo clippy`;
- `cargo test`.

Rules:

- Prefer explicit types in public APIs.
- Prefer small modules with clear ownership.
- Avoid global mutable state.
- Avoid panics in runtime paths.
- Use `Result` for recoverable errors.
- Use domain-specific errors instead of stringly typed errors.
- Keep public API independent from third-party backend types.
- Do not introduce async runtime unless explicitly required.
- Do not use nightly features.

### Python

Use:

- Python 3.8+;
- black;
- isort;
- pytest;
- maturin for bindings.

Rules:

- Keep Python API ergonomic and small.
- Keep comments and documentation in English.
- Do not expose unstable Rust internals through Python.
- Capture Python callback errors and return controlled runtime errors.

### Documentation

Write project documentation, code comments and README content in English unless the task explicitly asks for Portuguese.

---

## 10. Error Handling

Use structured errors.

Required error categories:

- `DslError`;
- `ValidationError`;
- `RuntimeError`;
- `LayoutError`;
- `RenderError`;
- `BackendError`;
- `GuiError`;
- `PythonBindingError`.

Default user-facing errors must be readable and actionable.

Do not print raw stack traces by default.

Verbose/debug mode may print additional technical details.

---

## 11. Logging and Security Rules

Security is mandatory even for MVP.

Rules:

- Never log full UI payloads by default.
- Never log sensitive text props by default.
- Never dump environment variables.
- Never log secrets, tokens, API keys, passwords or personal data.
- Debug logs may include component IDs, component kinds, event types and timings.
- Debug logs must not include arbitrary component text unless explicitly enabled in a future unsafe diagnostic mode.

Acceptable log fields:

- `component_id`;
- `component_kind`;
- `event_type`;
- `layout_duration_ms`;
- `render_duration_ms`;
- `frame_cells_changed`;
- `terminal_size`;
- `error_category`.

Not acceptable by default:

- full label text;
- full text block content;
- environment variable values;
- command output containing user data;
- serialized full app spec.

---

## 12. Testing Requirements

Every feature must include tests unless impractical.

### Required test categories

For core:

- unit tests for geometry;
- unit tests for layout;
- unit tests for event mapping;
- snapshot tests for rendering;
- DSL validation tests;
- component behavior tests.

For CLI:

- command parsing tests;
- `check` command tests;
- invalid file tests;
- exit code tests.

For Python:

- import smoke test;
- component construction test;
- callback success test;
- callback failure test.

For GUI:

- smoke test documentation is acceptable in MVP;
- automated GUI tests are optional for MVP.

### Snapshot Tests

Use snapshot/golden tests for:

- `Label`;
- `Panel`;
- `Button`;
- `List`;
- `Graph`;
- composed dashboard layout.

### Fixtures

Keep DSL fixtures under:

```text
crates/neotui-core/tests/fixtures/
```

Recommended structure:

```text
fixtures/
├── valid/
│   ├── hello.toml
│   ├── dashboard.toml
│   └── nested-layout.toml
└── invalid/
    ├── unknown-component.toml
    ├── missing-required-prop.toml
    ├── invalid-prop-type.toml
    └── invalid-children.toml
```

---

## 13. Performance Targets

Initial targets for MVP:

- p95 input-to-render latency under 30 ms for simple dashboards;
- p95 frame render under 16 ms for 120x40 dashboard;
- idle CPU below 3% for non-animated dashboards;
- no full-screen redraw when a small frame diff is possible;
- memory target below 40 MB for terminal-only runtime;
- memory target below 100 MB when Python bindings are involved.

These are goals, not blockers for early tasks.

If performance targets are not met, add benchmarks and document findings before optimizing aggressively.

---

## 14. Accessibility and Terminal Compatibility

MVP must behave well in common Linux terminal environments.

Target environments:

- GNOME Terminal;
- Alacritty;
- Kitty;
- Linux TTY where possible;
- SSH session.

Rules:

- Keyboard navigation must work even if mouse is unavailable.
- Resize must not crash the app.
- Very small terminal sizes must degrade gracefully.
- Unicode width must be handled carefully.
- Text clipping is better than layout panic.

---

## 15. Agent Execution Rules

When executing a task, follow this sequence:

1. Read the task objective.
2. Identify affected modules.
3. Inspect existing files before editing.
4. Make the smallest coherent change.
5. Add or update tests.
6. Run relevant tests.
7. Run formatting.
8. Update docs/examples if behavior changed.
9. Summarize what changed and what remains.
10. **Record the last executed task in this file** by updating the `LAST_EXECUTED_TASK` field below.

Do not rewrite unrelated files.

Do not perform broad refactors unless the task explicitly asks for refactoring.

Do not introduce new dependencies without a clear reason.

Do not change public API casually.

Do not remove tests to make the build pass.

Do not silence warnings without understanding them.

---

## 16. Stop Conditions

Stop and report clearly if any of the following happens:

1. The task requires a product decision not defined in this file.
2. The task would require replacing the chosen MVP architecture.
3. A dependency introduces incompatible licensing.
4. A dependency forces nightly Rust.
5. A dependency prevents Linux terminal-first execution.
6. A change would expose sensitive data in logs.
7. A change requires implementing native GUI rendering for MVP.
8. A change requires WebView/Tauri/xterm.js in MVP.
9. A test failure appears unrelated to the current change.
10. The requested implementation conflicts with an existing public API.

When stopping, provide:

- what was attempted;
- what blocked progress;
- files inspected;
- recommended next decision.

---

## 17. Dependency Policy

Before adding a dependency, verify:

- maintenance status;
- license compatibility;
- Linux support;
- impact on binary size;
- whether it leaks into public APIs;
- whether it supports Rust stable.

Preferred dependency behavior:

- isolated behind NeoTUI abstractions;
- optional features for GUI/Python;
- minimal transitive complexity.

Do not add a dependency just to avoid writing a small amount of core logic if the logic is central to NeoTUI identity.

---

## 18. Feature Flags

Use Cargo features to keep modules optional.

Recommended features:

```toml
[features]
default = ["terminal"]
terminal = []
gui = []
python = []
devtools = []
```

Rules:

- `neotui-core` should not require GUI dependencies.
- `neotui-cli` may depend on terminal runtime.
- `neotui-gui` may depend on GTK/VTE.
- Python bindings must not be required for terminal-only apps.

---

## 19. Branch and Commit Convention

Use small commits.

Preferred commit format:

```text
type(scope): short description
```

Examples:

```text
feat(core): add screen buffer
feat(layout): implement vertical layout constraints
feat(cli): add check command
test(render): add panel snapshot
docs(readme): add quickstart
fix(terminal): restore raw mode on panic
```

Allowed types:

- `feat`
- `fix`
- `test`
- `docs`
- `refactor`
- `chore`
- `build`
- `ci`

---

## 20. Definition of Ready

A task is ready when it has:

- clear objective;
- target module;
- expected behavior;
- acceptance criteria;
- test expectation;
- known constraints;
- explicit out-of-scope items.

If a task is not ready, make the smallest safe assumption and document it.

If the assumption could change architecture, stop.

---

## 21. Definition of Done

A task is done when:

1. Code compiles.
2. Relevant tests pass.
3. Formatting passes.
4. Lint passes or warnings are justified.
5. Public behavior is documented when needed.
6. Examples are updated when needed.
7. Errors are handled without panic in normal paths.
8. Logs do not leak sensitive data.
9. Terminal state is preserved if terminal runtime is touched.
10. The change is summarized clearly.

---

## 22. MVP Definition of Done

The MVP is done when:

1. `neotui run examples/dashboard.toml` works in a Linux terminal.
2. `neotui run examples/dashboard.toml --gui` works in a Linux desktop window.
3. The app supports keyboard, mouse, scroll, resize, focus and Ctrl+Q.
4. The framework includes:

   - `VBox`;
   - `HBox`;
   - `Panel`;
   - `Label`;
   - `Button`;
   - `List`;
   - `Graph`;
   - `Spacer`;
   - `Divider`.
5. The framework includes at least three themes:

   - `minimal`;
   - `dark`;
   - `cyberpunk`.
6. `neotui check <file>` validates DSL files.
7. `neotui doctor` provides useful diagnostics.
8. A minimal Python API works.
9. Terminal state is restored after normal exit and controlled failure.
10. Logs do not expose sensitive payloads.
11. Core layout/render/event/DSL tests exist.
12. At least three examples exist.
13. README and quickstart exist.
14. A showcase demo exists.
15. Linux packaging path is documented or experimentally available.

---

## 23. Out of Scope for MVP

Do not implement these unless explicitly requested after MVP baseline:

- native GUI renderer;
- Tauri/WebView runtime;
- xterm.js bridge;
- remote browser execution;
- plugin marketplace;
- WASM plugins;
- Lua plugins;
- visual builder;
- animations;
- modal/window manager;
- advanced table widget;
- forms framework;
- Windows support;
- macOS support;
- `.exe`, `.msi`, `.dmg` packaging;
- cloud deployment;
- SaaS runtime.

---

## 24. Recommended Implementation Order

Follow this order:

1. Bootstrap monorepo.
2. Terminal lifecycle.
3. Frame buffer.
4. ANSI renderer.
5. Component trait/model.
6. Layout engine.
7. Label and Panel.
8. CLI `run` minimal.
9. Button and List.
10. Event dispatch and focus.
11. DSL model and validation.
12. CLI `check`.
13. Themes.
14. Graph.
15. Python bindings.
16. GUI embedded mode.
17. Doctor command.
18. Tests and benchmarks.
19. Examples and showcase.
20. Packaging.

Do not start GUI before terminal runtime is stable.

Do not start Python bindings before the component model is stable.

Do not start advanced widgets before DSL and base widgets are stable.

---

## 25. First Execution Package

The first agent execution should target the smallest vertical slice.

Goal:

Run a minimal NeoTUI application in a real terminal.

Required deliverables:

- Rust workspace;
- `neotui-core`;
- `neotui-cli`;
- terminal session lifecycle;
- screen buffer;
- ANSI renderer;
- `Label`;
- minimal DSL fixture;
- `neotui run examples/hello.toml`;
- Ctrl+Q exit;
- terminal restore on exit;
- tests for buffer/render basics;
- README instructions.

Expected command:

```bash
cargo run -p neotui-cli -- run examples/hello.toml
```

Expected behavior:

- enter alternate screen;
- render “Hello NeoTUI”;
- exit with Ctrl+Q;
- restore terminal.

---

## 26. Example Minimal DSL

Use this as the first fixture target:

```toml
schema_version = "0.1"
theme = "minimal"

[root]
kind = "Label"

[root.props]
text = "Hello NeoTUI"
align = "center"
```

If the chosen parser requires a different TOML shape, adjust the fixture, but keep the semantic structure:

- app spec;
- schema version;
- theme;
- root component;
- component kind;
- component props.

---

## 27. Final Rule

Prefer a small, working, tested vertical slice over a large incomplete architecture.

NeoTUI must grow through executable increments.

Every change should move the project closer to:

```bash
neotui run examples/dashboard.toml
neotui run examples/dashboard.toml --gui
```
