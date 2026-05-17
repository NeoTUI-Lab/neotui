// NeoTUI CLI
// Command-line interface for NeoTUI applications

use std::fmt;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::ExitCode;
use std::{collections::BTreeMap, ffi::OsString};

use clap::{Parser, Subcommand};
use crossterm::terminal;
use neotui_core::component::{ComponentTree, EventContext, LayoutContext};
use neotui_core::dsl::{AppSpec, DslFormat};
use neotui_core::event::{Command as AppCommand, Event, EventResult};
use neotui_core::layout::Rect;
use neotui_core::registry::ComponentRegistry;
use neotui_core::render::{AnsiRenderer, ScreenBuffer};
use neotui_core::runtime::{panic, AppRuntime, TerminalSession};

#[derive(Debug, Parser)]
#[command(name = "neotui", version, about = "NeoTUI command-line interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute a NeoTUI DSL file in the terminal runtime
    Run { file: String },
    /// Parse and validate a NeoTUI DSL file
    Check { file: String },
    /// Report basic terminal/runtime readiness information
    Doctor,
}

fn main() -> ExitCode {
    run(std::env::args())
}

fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);

    match cli.command {
        Command::Run { file } => match run_file(Path::new(&file)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(1)
            }
        },
        Command::Check { file } => match check_file(Path::new(&file)) {
            Ok(summary) => {
                println!("{summary}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(1)
            }
        },
        Command::Doctor => {
            println!("{}", doctor_report());
            ExitCode::SUCCESS
        }
    }
}

struct LoadedApp {
    format: DslFormat,
    spec: AppSpec,
    tree: ComponentTree,
}

#[derive(Debug)]
enum AppLoadError {
    UnsupportedFormat {
        path: String,
    },
    Read {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
        format: DslFormat,
        source: neotui_core::dsl::DslError,
    },
    Validation {
        path: String,
        format: DslFormat,
        root_kind: String,
        errors: neotui_core::dsl::ValidationErrors,
    },
    Instantiation {
        path: String,
        format: DslFormat,
        root_kind: String,
        source: neotui_core::registry::RegistryError,
    },
}

impl fmt::Display for AppLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "check failed")?;
        writeln!(f, "file: `{}`", self.path())?;
        writeln!(f, "phase: {}", self.phase())?;

        if let Some(format) = self.format() {
            writeln!(f, "format: {}", display_format(format))?;
        }

        if let Some(root_kind) = self.root_kind() {
            writeln!(f, "root: `{root_kind}`")?;
        }

        match self {
            Self::UnsupportedFormat { .. } => write!(
                f,
                "details: unsupported DSL format; expected .toml or .json\nhint: rename the file to use a supported extension or convert it to TOML/JSON before running `neotui check`"
            ),
            Self::Read { source, .. } => write!(
                f,
                "details: {source}\nhint: confirm the file exists and that the current user can read it"
            ),
            Self::Parse { source, .. } => write!(
                f,
                "details: {source}\nhint: fix the file syntax first, then re-run `neotui check {}`",
                self.path()
            ),
            Self::Validation { errors, .. } => write!(
                f,
                "details:\n{errors}\nhint: fix the invalid fields above and re-run `neotui check {}`",
                self.path()
            ),
            Self::Instantiation { source, .. } => write!(
                f,
                "details: {source}\nhint: use currently executable components such as Panel, Label, Divider, Spacer, VBox and HBox, or implement the missing runtime widget first"
            ),
        }
    }
}

impl AppLoadError {
    fn phase(&self) -> &'static str {
        match self {
            Self::UnsupportedFormat { .. } => "format-detect",
            Self::Read { .. } => "read",
            Self::Parse { .. } => "parse",
            Self::Validation { .. } => "validate",
            Self::Instantiation { .. } => "instantiate",
        }
    }

    fn path(&self) -> &str {
        match self {
            Self::UnsupportedFormat { path }
            | Self::Read { path, .. }
            | Self::Parse { path, .. }
            | Self::Validation { path, .. }
            | Self::Instantiation { path, .. } => path,
        }
    }

    fn format(&self) -> Option<DslFormat> {
        match self {
            Self::Parse { format, .. }
            | Self::Validation { format, .. }
            | Self::Instantiation { format, .. } => Some(*format),
            Self::UnsupportedFormat { .. } | Self::Read { .. } => None,
        }
    }

    fn root_kind(&self) -> Option<&str> {
        match self {
            Self::Validation { root_kind, .. } | Self::Instantiation { root_kind, .. } => {
                Some(root_kind)
            }
            Self::UnsupportedFormat { .. } | Self::Read { .. } | Self::Parse { .. } => None,
        }
    }
}

fn display_format(format: DslFormat) -> &'static str {
    match format {
        DslFormat::Toml => "toml",
        DslFormat::Json => "json",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorReport {
    backend: &'static str,
    stdin_tty: bool,
    stdout_tty: bool,
    terminal_size: Option<(u16, u16)>,
    term_env_present: bool,
    colorterm_env_present: bool,
}

fn doctor_report() -> String {
    format_doctor_report(collect_doctor_report())
}

fn collect_doctor_report() -> DoctorReport {
    DoctorReport {
        backend: "crossterm",
        stdin_tty: io::stdin().is_terminal(),
        stdout_tty: io::stdout().is_terminal(),
        terminal_size: terminal::size().ok(),
        term_env_present: std::env::var_os("TERM").is_some(),
        colorterm_env_present: std::env::var_os("COLORTERM").is_some(),
    }
}

fn format_doctor_report(report: DoctorReport) -> String {
    let terminal_size = report
        .terminal_size
        .map(|(width, height)| format!("{width}x{height}"))
        .unwrap_or_else(|| "unavailable".into());
    let readiness = if report.stdin_tty && report.stdout_tty {
        "ready"
    } else {
        "degraded"
    };

    format!(
        "doctor {readiness}\nbackend: {}\nstdin_tty: {}\nstdout_tty: {}\nterminal_size: {}\nterm_env_present: {}\ncolorterm_env_present: {}\nhint: this report avoids printing terminal environment values directly",
        report.backend,
        bool_label(report.stdin_tty),
        bool_label(report.stdout_tty),
        terminal_size,
        bool_label(report.term_env_present),
        bool_label(report.colorterm_env_present)
    )
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn format_check_success(
    path: &Path,
    format: DslFormat,
    spec: &AppSpec,
    tree: &ComponentTree,
) -> String {
    let theme = spec.theme.as_deref().unwrap_or("none");
    let inspection = CheckInspection::from_root(&spec.root);
    let component_ids = tree
        .ids_depth_first()
        .into_iter()
        .map(|id| id.0)
        .collect::<Vec<_>>();
    let component_preview = component_ids.join(", ");

    format!(
        "check ok\nfile: `{}`\nformat: {}\nschema_version: `{}`\ntheme: `{}`\nroot: `{}`\ncomponent_count: {}\nmax_depth: {}\ncontainer_components: {}\nleaf_components: {}\nstructure_balance: {}\ndominant_kinds: [{}]\norientation: [{}]\nlayout_props: [{}]\ncomponent_kinds: [{}]\ncomponent_ids: [{}]\nphases: parse, validate, instantiate",
        path.display(),
        display_format(format),
        spec.schema_version,
        theme,
        spec.root.kind,
        tree.component_count(),
        tree.max_depth(),
        inspection.metrics.container_components,
        inspection.metrics.leaf_components,
        inspection.structure_balance(),
        inspection.dominant_kinds_preview(),
        inspection.orientation_preview(),
        inspection.layout_props_preview(),
        inspection.kind_counts_preview(),
        component_preview
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StructureMetrics {
    container_components: usize,
    leaf_components: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckInspection {
    kind_counts: BTreeMap<String, usize>,
    layout_prop_counts: BTreeMap<&'static str, usize>,
    orientation_counts: BTreeMap<&'static str, usize>,
    metrics: StructureMetrics,
}

impl CheckInspection {
    fn from_root(root: &neotui_core::dsl::ComponentSpec) -> Self {
        fn visit(component: &neotui_core::dsl::ComponentSpec, inspection: &mut CheckInspection) {
            *inspection
                .kind_counts
                .entry(component.kind.clone())
                .or_insert(0) += 1;

            if component.children.is_empty() {
                inspection.metrics.leaf_components += 1;
            } else {
                inspection.metrics.container_components += 1;
            }

            let has_fixed =
                component.props.contains_key("width") || component.props.contains_key("height");
            let has_percent = component.props.contains_key("width_pct")
                || component.props.contains_key("height_pct");

            if component.props.contains_key("gap") {
                *inspection.layout_prop_counts.entry("gap").or_insert(0) += 1;
            }
            if component.props.contains_key("grow") {
                *inspection.layout_prop_counts.entry("grow").or_insert(0) += 1;
            }
            if has_fixed {
                *inspection.layout_prop_counts.entry("fixed").or_insert(0) += 1;
            }
            if has_percent {
                *inspection.layout_prop_counts.entry("percent").or_insert(0) += 1;
            }
            if component.props.contains_key("align") {
                *inspection.layout_prop_counts.entry("align").or_insert(0) += 1;
            }
            if component.props.contains_key("justify") {
                *inspection.layout_prop_counts.entry("justify").or_insert(0) += 1;
            }

            match component.kind.as_str() {
                "VBox" => *inspection.orientation_counts.entry("vertical").or_insert(0) += 1,
                "HBox" => {
                    *inspection
                        .orientation_counts
                        .entry("horizontal")
                        .or_insert(0) += 1
                }
                "Panel" => *inspection.orientation_counts.entry("framed").or_insert(0) += 1,
                "Divider" => {
                    *inspection
                        .orientation_counts
                        .entry("separator")
                        .or_insert(0) += 1
                }
                _ => {}
            }

            for child in &component.children {
                visit(child, inspection);
            }
        }

        let mut inspection = Self {
            kind_counts: BTreeMap::new(),
            layout_prop_counts: BTreeMap::from([
                ("align", 0),
                ("fixed", 0),
                ("gap", 0),
                ("grow", 0),
                ("justify", 0),
                ("percent", 0),
            ]),
            orientation_counts: BTreeMap::from([
                ("framed", 0),
                ("horizontal", 0),
                ("separator", 0),
                ("vertical", 0),
            ]),
            metrics: StructureMetrics {
                container_components: 0,
                leaf_components: 0,
            },
        };
        visit(root, &mut inspection);
        inspection
    }

    fn structure_balance(&self) -> &'static str {
        match (
            self.metrics
                .container_components
                .cmp(&self.metrics.leaf_components),
            self.metrics.container_components,
            self.metrics.leaf_components,
        ) {
            (_, 0, leafs) if leafs > 0 => "leaf-only",
            (std::cmp::Ordering::Greater, _, _) => "container-heavy",
            (std::cmp::Ordering::Equal, _, _) => "balanced",
            (std::cmp::Ordering::Less, _, _) => "leaf-heavy",
        }
    }

    fn dominant_kinds_preview(&self) -> String {
        let Some(max_count) = self.kind_counts.values().copied().max() else {
            return String::new();
        };

        let mut dominant = self
            .kind_counts
            .iter()
            .filter(|(_, count)| **count == max_count)
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>();
        dominant.sort();
        dominant.join(", ")
    }

    fn orientation_preview(&self) -> String {
        Self::format_non_zero_counts(&self.orientation_counts)
    }

    fn layout_props_preview(&self) -> String {
        Self::format_non_zero_counts(&self.layout_prop_counts)
    }

    fn kind_counts_preview(&self) -> String {
        self.kind_counts
            .iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_non_zero_counts<K>(counts: &BTreeMap<K, usize>) -> String
    where
        K: AsRef<str> + Ord,
    {
        counts
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(key, count)| format!("{}={count}", key.as_ref()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn load_app(path: &Path) -> Result<LoadedApp, AppLoadError> {
    let path_display = path.display().to_string();
    let format = DslFormat::detect_from_path(&path.to_string_lossy()).ok_or_else(|| {
        AppLoadError::UnsupportedFormat {
            path: path_display.clone(),
        }
    })?;

    let input = fs::read_to_string(path).map_err(|source| AppLoadError::Read {
        path: path_display.clone(),
        source,
    })?;

    let spec = match format {
        DslFormat::Toml => AppSpec::from_toml_str(&input),
        DslFormat::Json => AppSpec::from_json_str(&input),
    }
    .map_err(|source| AppLoadError::Parse {
        path: path_display.clone(),
        format,
        source,
    })?;

    spec.validate().map_err(|errors| AppLoadError::Validation {
        path: path_display.clone(),
        format,
        root_kind: spec.root.kind.clone(),
        errors,
    })?;

    let tree = ComponentRegistry::new()
        .build_tree(&spec)
        .map_err(|source| AppLoadError::Instantiation {
            path: path_display,
            format,
            root_kind: spec.root.kind.clone(),
            source,
        })?;

    Ok(LoadedApp { format, spec, tree })
}

fn check_file(path: &Path) -> Result<String, String> {
    let LoadedApp { format, spec, tree } = load_app(path).map_err(|error| error.to_string())?;

    Ok(format_check_success(path, format, &spec, &tree))
}

fn run_file(path: &Path) -> Result<(), String> {
    let LoadedApp { mut tree, .. } = load_app(path).map_err(|error| error.to_string())?;
    let renderer = AnsiRenderer::new();
    let mut terminal = TerminalSession::new();
    let mut runtime = AppRuntime::new();
    let mut viewport = terminal::size()
        .map_err(|source| format!("failed to read terminal size before startup: {source}"))?;
    let mut render_error = None;

    panic::install_panic_hook();
    terminal
        .enter()
        .map_err(|source| format!("failed to enter terminal session: {source}"))?;

    render_tree(&tree, viewport, &renderer)
        .map_err(|source| format!("failed to render `{}`: {source}", path.display()))?;

    let runtime_result = runtime.run(|event| {
        let mut event_ctx = EventContext::default();
        let result = tree.dispatch_event(&mut event_ctx, &event);

        if let Event::Resize { width, height } = &event {
            viewport = (width, height);
        }

        if result.requests_render() || event.requests_render() {
            if let Err(source) = render_tree(&tree, viewport, &renderer) {
                render_error = Some(source);
                return EventResult::Command(AppCommand::Quit);
            }
        }

        result
    });

    let exit_result = terminal.exit();

    if let Err(source) = runtime_result {
        return Err(format!(
            "runtime failure while running `{}`: {source}",
            path.display()
        ));
    }

    if let Some(source) = render_error {
        return Err(format!(
            "failed to render updated frame for `{}`: {source}",
            path.display()
        ));
    }

    exit_result.map_err(|source| {
        format!(
            "failed to restore terminal after running `{}`: {source}",
            path.display()
        )
    })
}

fn render_tree(
    tree: &ComponentTree,
    viewport: (u16, u16),
    renderer: &AnsiRenderer,
) -> std::io::Result<()> {
    let area = Rect::new(0, 0, viewport.0, viewport.1);
    let mut frame = ScreenBuffer::new(viewport.0, viewport.1);
    let layout = tree.layout(&LayoutContext, area);

    tree.render_with_layout(&layout, &mut frame);
    renderer.render_to_stdout(&frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_file(extension: &str, contents: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("neotui-{unique}.{extension}"));
        fs::write(&path, contents).expect("temp fixture should be writable");
        path
    }

    #[test]
    fn clap_parses_check_command() {
        let cli = Cli::parse_from(["neotui", "check", "examples/hello.toml"]);

        match cli.command {
            Command::Run { .. } => panic!("unexpected run command"),
            Command::Check { file } => assert_eq!(file, "examples/hello.toml"),
            Command::Doctor => panic!("unexpected doctor command"),
        }
    }

    #[test]
    fn clap_parses_run_command() {
        let cli = Cli::parse_from(["neotui", "run", "examples/hello.toml"]);

        match cli.command {
            Command::Run { file } => assert_eq!(file, "examples/hello.toml"),
            Command::Check { .. } => panic!("unexpected check command"),
            Command::Doctor => panic!("unexpected doctor command"),
        }
    }

    #[test]
    fn clap_parses_doctor_command() {
        let cli = Cli::parse_from(["neotui", "doctor"]);

        match cli.command {
            Command::Doctor => {}
            Command::Run { .. } => panic!("unexpected run command"),
            Command::Check { .. } => panic!("unexpected check command"),
        }
    }

    #[test]
    fn doctor_report_formats_without_sensitive_env_values() {
        let output = format_doctor_report(DoctorReport {
            backend: "crossterm",
            stdin_tty: true,
            stdout_tty: false,
            terminal_size: Some((120, 40)),
            term_env_present: true,
            colorterm_env_present: false,
        });

        assert!(output.contains("doctor degraded"));
        assert!(output.contains("backend: crossterm"));
        assert!(output.contains("stdin_tty: yes"));
        assert!(output.contains("stdout_tty: no"));
        assert!(output.contains("terminal_size: 120x40"));
        assert!(output.contains("term_env_present: yes"));
        assert!(output.contains("colorterm_env_present: no"));
        assert!(output.contains("avoids printing terminal environment values directly"));
        assert!(!output.contains("xterm"));
    }

    #[test]
    fn check_file_accepts_valid_toml() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root]
kind = "Label"

[root.props]
text = "Hello"
"#,
        );

        let output = check_file(&path).expect("valid DSL should pass");

        assert!(output.contains("check ok"));
        assert!(output.contains("format: toml"));
        assert!(output.contains("root: `Label`"));
        assert!(output.contains("component_count: 1"));
        assert!(output.contains("max_depth: 1"));
        assert!(output.contains("container_components: 0"));
        assert!(output.contains("leaf_components: 1"));
        assert!(output.contains("structure_balance: leaf-only"));
        assert!(output.contains("dominant_kinds: [Label=1]"));
        assert!(output.contains("orientation: []"));
        assert!(output.contains("layout_props: [align=1]"));
        assert!(output.contains("component_kinds: [Label=1]"));
        assert!(output.contains("component_ids: [root]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_app_builds_component_tree_for_valid_fixture() {
        let app = load_app(Path::new("examples/hello.toml")).expect("hello fixture should load");

        assert_eq!(app.format, DslFormat::Toml);
        assert_eq!(app.spec.root.kind, "Label");
        assert_eq!(
            app.tree
                .ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["root"]
        );
    }

    #[test]
    fn check_inspection_consolidates_structural_helpers() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[root]
kind = "VBox"

[root.props]
gap = 1
align = "center"

[[root.children]]
kind = "Label"

[root.children.props]
text = "A"
align = "center"
width = 4
height = 1

[[root.children]]
kind = "HBox"

[root.children.props]
gap = 2
justify = "center"

[[root.children.children]]
kind = "Label"

[root.children.children.props]
text = "B"
grow = 1
"#,
        )
        .expect("inline app spec should parse");

        let inspection = CheckInspection::from_root(&spec.root);

        assert_eq!(
            inspection.metrics,
            StructureMetrics {
                container_components: 2,
                leaf_components: 2,
            }
        );
        assert_eq!(inspection.structure_balance(), "balanced");
        assert_eq!(inspection.dominant_kinds_preview(), "Label=2");
        assert_eq!(inspection.orientation_preview(), "horizontal=1, vertical=1");
        assert_eq!(
            inspection.layout_props_preview(),
            "align=2, fixed=1, gap=2, grow=1, justify=1"
        );
        assert_eq!(inspection.kind_counts_preview(), "HBox=1, Label=2, VBox=1");
    }

    #[test]
    fn check_file_rejects_unsupported_extension() {
        let path = write_temp_file("yaml", "schema_version: '0.1'");

        let error = check_file(&path).expect_err("yaml should not be accepted yet");

        assert!(error.contains("check failed"));
        assert!(error.contains("phase: format-detect"));
        assert!(error.contains("unsupported DSL format"));
        assert!(error.contains("rename the file"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn check_file_rejects_invalid_component() {
        let path = write_temp_file(
            "json",
            r#"{
  "schema_version": "0.1",
  "root": {
    "kind": "Label",
    "props": {
      "text": 42
    }
  }
}"#,
        );

        let error = check_file(&path).expect_err("invalid props should fail validation");

        assert!(error.contains("phase: validate"));
        assert!(error.contains("format: json"));
        assert!(error.contains("root: `Label`"));
        assert!(error.contains("root.props.text"));
        assert!(error.contains("fix the invalid fields"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn check_file_rejects_invalid_syntax_with_actionable_hint() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root
kind = "Label"
"#,
        );

        let error = check_file(&path).expect_err("invalid syntax should fail parsing");

        assert!(error.contains("phase: parse"));
        assert!(error.contains("format: toml"));
        assert!(error.contains("failed to parse TOML DSL"));
        assert!(error.contains("fix the file syntax first"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn check_file_rejects_unimplemented_component_with_runtime_guidance() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root]
kind = "Button"
"#,
        );

        let error = check_file(&path).expect_err("button should fail instantiation");

        assert!(error.contains("phase: instantiate"));
        assert!(error.contains("root: `Button`"));
        assert!(error.contains("component instantiation failed"));
        assert!(error.contains("known but not implemented yet"));
        assert!(error.contains("Panel, Label, Divider, Spacer, VBox and HBox"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn run_returns_failure_exit_code_for_invalid_file() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root]
kind = "Unknown"
"#,
        );

        let exit = run([
            "neotui".to_string(),
            "run".to_string(),
            path.to_string_lossy().to_string(),
        ]);

        assert_eq!(exit, ExitCode::from(1));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn render_tree_draws_hello_fixture_text() {
        let LoadedApp { tree, .. } =
            load_app(Path::new("examples/hello.toml")).expect("hello fixture should load");
        let area = Rect::new(0, 0, 20, 3);
        let mut frame = ScreenBuffer::new(20, 3);
        let layout = tree.layout(&LayoutContext, area);

        tree.render_with_layout(&layout, &mut frame);

        let rendered_row: String = (0..20)
            .map(|x| frame.get(x, 1).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();

        assert!(rendered_row.contains("Hello NeoTUI"));
    }

    #[test]
    fn render_tree_places_dashboard_children_in_distinct_rows() {
        let LoadedApp { tree, .. } =
            load_app(Path::new("examples/dashboard.toml")).expect("dashboard fixture should load");
        let area = Rect::new(0, 0, 36, 8);
        let mut frame = ScreenBuffer::new(36, 8);
        let layout = tree.layout(&LayoutContext, area);

        tree.render_with_layout(&layout, &mut frame);

        let headline_row: String = (0..36)
            .map(|x| frame.get(x, 1).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let divider_row: String = (0..36)
            .map(|x| frame.get(x, 3).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let summary_row: String = (0..36)
            .map(|x| frame.get(x, 5).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();

        assert!(headline_row.contains("Service Health"));
        assert!(divider_row.contains("="));
        assert!(summary_row.contains("All critical services responding"));
    }

    #[test]
    fn check_file_accepts_dashboard_examples() {
        let toml_output = check_file(Path::new("examples/dashboard.toml"))
            .expect("dashboard.toml should validate");
        let json_output = check_file(Path::new("examples/dashboard.json"))
            .expect("dashboard.json should validate");
        let theme_output = check_file(Path::new("examples/theme-demo.toml"))
            .expect("theme-demo.toml should validate");
        let layout_output = check_file(Path::new("examples/layout-demo.toml"))
            .expect("layout-demo.toml should validate");
        let showcase_output = check_file(Path::new("examples/showcase-layout.toml"))
            .expect("showcase-layout.toml should validate");

        assert!(toml_output.contains("root: `Panel`"));
        assert!(json_output.contains("root: `Panel`"));
        assert!(theme_output.contains("root: `Panel`"));
        assert!(layout_output.contains("root: `VBox`"));
        assert!(layout_output.contains("container_components: 2"));
        assert!(layout_output.contains("leaf_components: 3"));
        assert!(layout_output.contains("structure_balance: leaf-heavy"));
        assert!(layout_output.contains("dominant_kinds: [Label=3]"));
        assert!(layout_output.contains("orientation: [horizontal=1, vertical=1]"));
        assert!(layout_output.contains("layout_props: [align=5, fixed=3, gap=2, justify=1]"));
        assert!(layout_output.contains("component_kinds: [HBox=1, Label=3, VBox=1]"));
        assert!(showcase_output.contains("root: `Panel`"));
        assert!(showcase_output.contains("component_count: 9"));
        assert!(showcase_output.contains("container_components: 3"));
        assert!(showcase_output.contains("leaf_components: 6"));
        assert!(showcase_output.contains("structure_balance: leaf-heavy"));
        assert!(showcase_output.contains("dominant_kinds: [Label=5]"));
        assert!(showcase_output
            .contains("orientation: [framed=1, horizontal=1, separator=1, vertical=1]"));
        assert!(showcase_output.contains("layout_props: [align=7, fixed=5, gap=2, justify=1]"));
        assert!(showcase_output
            .contains("component_kinds: [Divider=1, HBox=1, Label=5, Panel=1, VBox=1]"));
    }

    #[test]
    fn render_tree_supports_nested_vbox_and_hbox_layouts() {
        let LoadedApp { tree, .. } =
            load_app(Path::new("examples/layout-demo.toml")).expect("layout fixture should load");
        let area = Rect::new(0, 0, 20, 4);
        let mut frame = ScreenBuffer::new(20, 4);
        let layout = tree.layout(&LayoutContext, area);

        assert_eq!(layout.children[0].area, Rect::new(5, 0, 10, 1));
        assert_eq!(layout.children[1].area, Rect::new(0, 2, 20, 2));
        assert_eq!(layout.children[1].children[0].area, Rect::new(5, 3, 4, 1));
        assert_eq!(layout.children[1].children[1].area, Rect::new(11, 3, 4, 1));

        tree.render_with_layout(&layout, &mut frame);

        let header_row: String = (0..20)
            .map(|x| frame.get(x, 0).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let gap_row: String = (0..20)
            .map(|x| frame.get(x, 1).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let columns_row: String = (0..20)
            .map(|x| frame.get(x, 3).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();

        assert!(header_row.contains("Layout Demo"));
        assert!(gap_row.trim().is_empty());
        assert!(columns_row.contains("Left"));
        assert!(columns_row.contains("Right"));
    }

    #[test]
    fn render_tree_supports_showcase_layout_example() {
        let LoadedApp { tree, .. } = load_app(Path::new("examples/showcase-layout.toml"))
            .expect("showcase layout fixture should load");
        let area = Rect::new(0, 0, 40, 10);
        let mut frame = ScreenBuffer::new(40, 10);
        let layout = tree.layout(&LayoutContext, area);

        assert_eq!(layout.children[0].area, Rect::new(1, 1, 38, 8));
        assert_eq!(layout.children[0].children[0].area, Rect::new(11, 1, 18, 1));
        assert_eq!(layout.children[0].children[2].area, Rect::new(1, 5, 38, 1));
        assert_eq!(
            layout.children[0].children[2].children[0].area,
            Rect::new(5, 5, 8, 1)
        );
        assert_eq!(
            layout.children[0].children[2].children[1].area,
            Rect::new(15, 5, 8, 1)
        );
        assert_eq!(
            layout.children[0].children[2].children[2].area,
            Rect::new(25, 5, 8, 1)
        );

        tree.render_with_layout(&layout, &mut frame);

        let title_row: String = (0..40)
            .map(|x| frame.get(x, 1).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let stats_row: String = (0..40)
            .map(|x| frame.get(x, 5).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let footer_row: String = (0..40)
            .map(|x| frame.get(x, 8).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();

        assert!(title_row.contains("Cluster Overview"));
        assert!(stats_row.contains("API OK"));
        assert!(stats_row.contains("Jobs OK"));
        assert!(stats_row.contains("Cache OK"));
        assert!(footer_row.contains("All critical services responding"));
    }
}
