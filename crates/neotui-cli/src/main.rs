// NeoTUI CLI
// Command-line interface for NeoTUI applications

use std::fmt;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::process::ExitCode;
use std::time::Duration;
use std::{collections::BTreeMap, ffi::OsString};

use clap::{Parser, Subcommand};
use crossterm::terminal;
use neotui_core::component::{ComponentTree, EventContext, LayoutContext, LayoutNode};
use neotui_core::data::{
    apply_runtime_bindings_with_forms, ActionSnapshot, ActionStatus, ActionStore, DataSnapshot,
    DataStore, HttpActionRuntime, HttpDataRuntime,
};
use neotui_core::diagnostics;
use neotui_core::dsl::{AppSpec, DslFormat};
use neotui_core::event::{Command as AppCommand, Event, EventResult, KeyCode};
use neotui_core::forms::FormStore;
use neotui_core::layout::Rect;
use neotui_core::registry::ComponentRegistry;
use neotui_core::render::{AnsiRenderer, ScreenBuffer};
use neotui_core::runtime::{panic, AppRuntime, TerminalSession};
use neotui_core::state::StateStore;
use tracing::debug;

#[derive(Debug, Parser)]
#[command(name = "neotui", version, about = "NeoTUI command-line interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute a NeoTUI DSL file in the terminal runtime or inside the embedded GTK/VTE GUI
    Run {
        file: String,
        /// Launch the app inside the embedded Linux GTK/VTE window instead of the current terminal
        #[arg(long)]
        gui: bool,
        /// Override which CLI program the GUI child process should execute for `run <file>`
        #[arg(long, requires = "gui")]
        gui_cli_program: Option<String>,
        /// Override the working directory used by the GUI child process
        #[arg(long, requires = "gui")]
        gui_working_directory: Option<String>,
        /// Forward one additional argument to the child `run` command inside the GUI; repeat as needed
        #[arg(long = "gui-forward-arg", requires = "gui", allow_hyphen_values = true)]
        gui_forward_args: Vec<String>,
    },
    /// Parse and validate a NeoTUI DSL file
    Check { file: String },
    /// Report basic terminal/runtime readiness information
    Doctor,
}

fn main() -> ExitCode {
    diagnostics::init_tracing();
    run(std::env::args())
}

fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    debug!(target: "neotui::cli", command = cli_command_name(&cli.command), "parsed CLI command");

    match cli.command {
        Command::Run {
            file,
            gui,
            gui_cli_program,
            gui_working_directory,
            gui_forward_args,
        } => match run_dispatch(
            Path::new(&file),
            gui,
            gui_cli_program.as_deref(),
            gui_working_directory.as_deref(),
            &gui_forward_args,
        ) {
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

#[derive(Debug)]
struct LoadedApp {
    format: DslFormat,
    spec: AppSpec,
    tree: ComponentTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiForwardingContract {
    cli_program: String,
    working_directory: Option<String>,
    forwarded_run_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiBinaryInvocation {
    program: PathBuf,
    args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorCategory {
    Input,
    FileSystem,
    DslParse,
    DslValidation,
    ComponentRegistry,
    TerminalSession,
    Render,
    Runtime,
    GuiConfig,
    GuiEnvironment,
    GuiBridge,
    GuiLaunch,
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
        write!(f, "{}", self.render_for("check"))
    }
}

impl AppLoadError {
    fn category(&self) -> ErrorCategory {
        match self {
            Self::UnsupportedFormat { .. } => ErrorCategory::Input,
            Self::Read { .. } => ErrorCategory::FileSystem,
            Self::Parse { .. } => ErrorCategory::DslParse,
            Self::Validation { .. } => ErrorCategory::DslValidation,
            Self::Instantiation { .. } => ErrorCategory::ComponentRegistry,
        }
    }

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

    fn render_for(&self, operation: &str) -> String {
        let mut lines = vec![
            format!("{operation} failed"),
            format!("category: {}", display_error_category(self.category())),
            format!("file: `{}`", self.path()),
            format!("phase: {}", self.phase()),
        ];

        if let Some(format) = self.format() {
            lines.push(format!("format: {}", display_format(format)));
        }

        if let Some(root_kind) = self.root_kind() {
            lines.push(format!("root: `{root_kind}`"));
        }

        match self {
            Self::UnsupportedFormat { .. } => {
                lines.push("details: unsupported DSL format; expected .toml or .json".into());
                lines.push(format!(
                    "hint: rename the file to use a supported extension or convert it to TOML/JSON before running `neotui {operation} {}`",
                    self.path()
                ));
            }
            Self::Read { source, .. } => {
                lines.push(format!("details: {source}"));
                lines.push(
                    "hint: confirm the file exists and that the current user can read it".into(),
                );
            }
            Self::Parse { source, .. } => {
                lines.push(format!("details: {source}"));
                lines.push(format!(
                    "hint: fix the file syntax first, then re-run `neotui {operation} {}`",
                    self.path()
                ));
            }
            Self::Validation { errors, .. } => {
                lines.push("details: schema validation failed".into());
                lines.push(errors.to_string());
                lines.push(format!(
                    "hint: fix the invalid fields above and re-run `neotui {operation} {}`",
                    self.path()
                ));
            }
            Self::Instantiation { source, .. } => {
                lines.push(format!("details: {source}"));
                lines.push(format!(
                    "hint: confirm the component props/runtime support for this widget, then re-run `neotui {operation} {}`",
                    self.path()
                ));
            }
        }

        lines.join("\n")
    }
}

fn display_format(format: DslFormat) -> &'static str {
    match format {
        DslFormat::Toml => "toml",
        DslFormat::Json => "json",
    }
}

fn display_error_category(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::Input => "input",
        ErrorCategory::FileSystem => "filesystem",
        ErrorCategory::DslParse => "dsl-parse",
        ErrorCategory::DslValidation => "dsl-validation",
        ErrorCategory::ComponentRegistry => "component-registry",
        ErrorCategory::TerminalSession => "terminal-session",
        ErrorCategory::Render => "render",
        ErrorCategory::Runtime => "runtime",
        ErrorCategory::GuiConfig => "gui-config",
        ErrorCategory::GuiEnvironment => "gui-environment",
        ErrorCategory::GuiBridge => "gui-bridge",
        ErrorCategory::GuiLaunch => "gui-launch",
    }
}

fn format_operation_error(
    operation: &str,
    category: ErrorCategory,
    phase: &str,
    path: Option<&Path>,
    details: impl Into<String>,
    hint: impl Into<String>,
) -> String {
    let mut lines = vec![
        format!("{operation} failed"),
        format!("category: {}", display_error_category(category)),
        format!("phase: {phase}"),
    ];

    if let Some(path) = path {
        lines.push(format!("file: `{}`", path.display()));
    }

    lines.push(format!("details: {}", details.into()));
    lines.push(format!("hint: {}", hint.into()));
    lines.join("\n")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorReport {
    backend: &'static str,
    stdin_tty: bool,
    stdout_tty: bool,
    terminal_size: Option<(u16, u16)>,
    terminal_size_class: &'static str,
    terminal_family: &'static str,
    color_support: &'static str,
    mouse_support: &'static str,
    raw_mode_support: &'static str,
    alternate_screen_support: &'static str,
    gui_support: &'static str,
    gui_platform_supported: bool,
    gui_session_available: bool,
    gui_gtk_backend_declared: bool,
    gui_vte_backend_declared: bool,
    gui_reason: &'static str,
    debug_mode: &'static str,
    term_env_present: bool,
    colorterm_env_present: bool,
    readiness: &'static str,
    hints: Vec<&'static str>,
}

fn doctor_report() -> String {
    format_doctor_report(collect_doctor_report())
}

fn collect_doctor_report() -> DoctorReport {
    let stdin_tty = io::stdin().is_terminal();
    let stdout_tty = io::stdout().is_terminal();
    let terminal_size = terminal::size().ok();
    let term_env = std::env::var_os("TERM");
    let colorterm_env = std::env::var_os("COLORTERM");
    let terminal_family = detect_terminal_family(term_env.as_ref());
    let color_support = detect_color_support(term_env.as_ref(), colorterm_env.as_ref());
    let mouse_support = detect_mouse_support(stdin_tty, stdout_tty, terminal_family);
    let raw_mode_support = detect_raw_mode_support(stdin_tty, stdout_tty, terminal_family);
    let alternate_screen_support = detect_alternate_screen_support(stdout_tty, terminal_family);
    let gui_availability = neotui_gui::detect_gui_availability();
    let gui_support = detect_gui_support(&gui_availability);
    let debug_mode = detect_debug_mode(std::env::var_os("NEOTUI_DEBUG").as_ref());
    let terminal_size_class = classify_terminal_size(terminal_size);
    let hints = collect_doctor_hints(
        stdin_tty,
        stdout_tty,
        terminal_size_class,
        &gui_availability,
        debug_mode,
    );
    let readiness = if stdin_tty
        && stdout_tty
        && raw_mode_support != "unavailable"
        && alternate_screen_support != "unavailable"
    {
        "ready"
    } else {
        "degraded"
    };

    DoctorReport {
        backend: "crossterm",
        stdin_tty,
        stdout_tty,
        terminal_size,
        terminal_size_class,
        terminal_family,
        color_support,
        mouse_support,
        raw_mode_support,
        alternate_screen_support,
        gui_support,
        gui_platform_supported: gui_availability.platform_supported,
        gui_session_available: gui_availability.session_available,
        gui_gtk_backend_declared: gui_availability.gtk_backend_declared,
        gui_vte_backend_declared: gui_availability.vte_backend_declared,
        gui_reason: gui_availability.reason,
        debug_mode,
        term_env_present: term_env.is_some(),
        colorterm_env_present: colorterm_env.is_some(),
        readiness,
        hints,
    }
}

fn format_doctor_report(report: DoctorReport) -> String {
    let terminal_size = report
        .terminal_size
        .map(|(width, height)| format!("{width}x{height}"))
        .unwrap_or_else(|| "unavailable".into());
    let hints = report.hints.join("; ");

    format!(
        "doctor {}\nbackend: {}\nstdin_tty: {}\nstdout_tty: {}\nterminal_size: {}\nterminal_size_class: {}\nterminal_family: {}\ncolor_support: {}\nmouse_support: {}\nraw_mode_support: {}\nalternate_screen_support: {}\ngui_support: {}\ngui_platform_supported: {}\ngui_session_available: {}\ngui_gtk_backend_declared: {}\ngui_vte_backend_declared: {}\ngui_reason: {}\ndebug_mode: {}\nterm_env_present: {}\ncolorterm_env_present: {}\nhint: {}\nnote: this report avoids printing terminal environment values directly",
        report.readiness,
        report.backend,
        bool_label(report.stdin_tty),
        bool_label(report.stdout_tty),
        terminal_size,
        report.terminal_size_class,
        report.terminal_family,
        report.color_support,
        report.mouse_support,
        report.raw_mode_support,
        report.alternate_screen_support,
        report.gui_support,
        bool_label(report.gui_platform_supported),
        bool_label(report.gui_session_available),
        bool_label(report.gui_gtk_backend_declared),
        bool_label(report.gui_vte_backend_declared),
        report.gui_reason,
        report.debug_mode,
        bool_label(report.term_env_present),
        bool_label(report.colorterm_env_present),
        hints
    )
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn detect_terminal_family(term: Option<&std::ffi::OsString>) -> &'static str {
    let Some(term) = term else {
        return "unavailable";
    };
    let lower = term.to_string_lossy().to_ascii_lowercase();

    if lower.contains("xterm") || lower.contains("kitty") || lower.contains("alacritty") {
        "xterm-compatible"
    } else if lower.contains("screen") || lower.contains("tmux") {
        "multiplexer"
    } else if lower == "linux" {
        "linux-console"
    } else if lower == "dumb" {
        "dumb"
    } else {
        "unknown"
    }
}

fn detect_color_support(
    term: Option<&std::ffi::OsString>,
    colorterm: Option<&std::ffi::OsString>,
) -> &'static str {
    let colorterm_lower = colorterm.map(|value| value.to_string_lossy().to_ascii_lowercase());
    let term_lower = term.map(|value| value.to_string_lossy().to_ascii_lowercase());

    if colorterm_lower
        .as_deref()
        .is_some_and(|value| value.contains("truecolor") || value.contains("24bit"))
    {
        "truecolor"
    } else if term_lower
        .as_deref()
        .is_some_and(|value| value.contains("256color"))
    {
        "ansi256"
    } else if term_lower.as_deref().is_some_and(|value| value == "dumb") {
        "monochrome"
    } else if term_lower.is_some() || colorterm_lower.is_some() {
        "basic"
    } else {
        "unknown"
    }
}

fn detect_mouse_support(stdin_tty: bool, stdout_tty: bool, terminal_family: &str) -> &'static str {
    if !stdin_tty || !stdout_tty {
        "unavailable"
    } else if terminal_family == "dumb" || terminal_family == "unavailable" {
        "unlikely"
    } else if terminal_family == "unknown" {
        "unknown"
    } else {
        "likely"
    }
}

fn detect_raw_mode_support(
    stdin_tty: bool,
    stdout_tty: bool,
    terminal_family: &str,
) -> &'static str {
    if !stdin_tty || !stdout_tty || terminal_family == "dumb" {
        "unavailable"
    } else {
        "likely"
    }
}

fn detect_alternate_screen_support(stdout_tty: bool, terminal_family: &str) -> &'static str {
    if !stdout_tty || terminal_family == "dumb" {
        "unavailable"
    } else if terminal_family == "unknown" || terminal_family == "unavailable" {
        "unknown"
    } else {
        "likely"
    }
}

fn detect_gui_support(gui_availability: &neotui_gui::GuiAvailability) -> &'static str {
    let gui_manifest_present = Path::new("crates/neotui-gui/Cargo.toml").exists()
        || PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("crates/neotui-gui/Cargo.toml")
            .exists();

    if !gui_manifest_present {
        "manifest-missing"
    } else if !gui_availability.platform_supported {
        "linux-only-runtime"
    } else if !gui_availability.session_available {
        "session-missing"
    } else if gui_availability.ready() {
        "gtk-vte-declared"
    } else {
        "degraded"
    }
}

fn detect_debug_mode(flag: Option<&std::ffi::OsString>) -> &'static str {
    if diagnostics::debug_mode_from_flag(flag) {
        "enabled"
    } else {
        "disabled"
    }
}

fn classify_terminal_size(size: Option<(u16, u16)>) -> &'static str {
    match size {
        Some((width, height)) if width >= 80 && height >= 24 => "comfortable",
        Some((width, height)) if width >= 40 && height >= 10 => "compact",
        Some(_) => "constrained",
        None => "unavailable",
    }
}

fn collect_doctor_hints(
    stdin_tty: bool,
    stdout_tty: bool,
    terminal_size_class: &str,
    gui_availability: &neotui_gui::GuiAvailability,
    debug_mode: &str,
) -> Vec<&'static str> {
    let mut hints = Vec::new();

    if !stdin_tty || !stdout_tty {
        hints.push("run `neotui` inside an interactive terminal session");
    }
    if terminal_size_class == "constrained" {
        hints.push("resize the terminal for a more reliable MVP layout preview");
    }
    if terminal_size_class == "unavailable" {
        hints.push("terminal size probing is unavailable in this environment");
    }
    if !gui_availability.platform_supported {
        hints.push("the MVP GUI path currently targets Linux with GTK/VTE");
    }
    if gui_availability.platform_supported && !gui_availability.session_available {
        hints.push(
            "start a Linux graphical session with DISPLAY or WAYLAND_DISPLAY before using `--gui`",
        );
    }
    if gui_availability.platform_supported
        && gui_availability.session_available
        && (!gui_availability.gtk_backend_declared || !gui_availability.vte_backend_declared)
    {
        hints.push("the GUI crate is present but its GTK/VTE backend declaration looks incomplete");
    }
    if debug_mode == "enabled" {
        hints.push("NEOTUI_DEBUG appears enabled, so extra diagnostics may be expected");
    }
    if hints.is_empty() {
        hints.push("core runtime signals look usable for terminal-first NeoTUI flows");
    }

    hints
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

    let mut forms = FormStore::new();
    for form in &spec.forms {
        for field in &form.fields {
            if let Some(initial) = &field.initial {
                let _ = forms.set(form.id.clone(), field.id.clone(), initial.clone());
            }
        }
    }
    let effective_spec =
        apply_runtime_bindings_with_forms(&spec, &DataStore::new(), &ActionStore::new(), &forms);
    let tree = ComponentRegistry::new()
        .build_tree(&effective_spec)
        .map_err(|source| AppLoadError::Instantiation {
            path: path_display,
            format,
            root_kind: spec.root.kind.clone(),
            source,
        })?;

    Ok(LoadedApp { format, spec, tree })
}

fn check_file(path: &Path) -> Result<String, String> {
    debug!(target: "neotui::cli", path = %path.display(), "checking DSL file");
    let LoadedApp { format, spec, tree } =
        load_app(path).map_err(|error| error.render_for("check"))?;

    Ok(format_check_success(path, format, &spec, &tree))
}

fn run_dispatch(
    path: &Path,
    gui: bool,
    gui_cli_program: Option<&str>,
    gui_working_directory: Option<&str>,
    gui_forward_args: &[String],
) -> Result<(), String> {
    debug!(
        target: "neotui::cli",
        path = %path.display(),
        gui,
        "dispatching run command"
    );
    if gui {
        run_file_gui(
            path,
            gui_cli_program,
            gui_working_directory,
            gui_forward_args,
        )
    } else {
        run_file(path)
    }
}

fn run_file(path: &Path) -> Result<(), String> {
    debug!(target: "neotui::cli", path = %path.display(), "starting terminal run");
    let LoadedApp { spec, .. } = load_app(path).map_err(|error| error.render_for("run"))?;
    let mut state = StateStore::new();
    let _ = state.initialize_forms(&spec.forms);
    let mut data_runtime = spec.data.as_ref().map(HttpDataRuntime::new);
    let mut action_runtime = if spec.actions.is_empty() {
        None
    } else {
        Some(HttpActionRuntime::new(&spec.actions))
    };
    let renderer = AnsiRenderer::new();
    let mut terminal = TerminalSession::new();
    let mut runtime = AppRuntime::new();
    if data_runtime.is_some() || action_runtime.is_some() {
        runtime = runtime.with_tick_rate(Duration::from_millis(250));
    }
    let mut viewport = terminal::size().map_err(|source| {
        format_operation_error(
            "run",
            ErrorCategory::TerminalSession,
            "terminal-size",
            Some(path),
            source.to_string(),
            "run `neotui doctor` to inspect terminal readiness, then retry inside an interactive terminal",
        )
    })?;
    let mut render_error = None;

    panic::install_panic_hook();
    terminal.enter().map_err(|source| {
        format_operation_error(
            "run",
            ErrorCategory::TerminalSession,
            "terminal-enter",
            Some(path),
            source.to_string(),
            "run `neotui doctor` to inspect raw-mode and alternate-screen support before retrying",
        )
    })?;

    if let Some(data) = &spec.data {
        for source in &data.sources {
            let _ = state.set_data_snapshot(source.id().to_string(), DataSnapshot::loading());
        }
    }
    for action in &spec.actions {
        let _ = state.set_action_snapshot(action.id.clone(), ActionSnapshot::idle());
    }
    if let Some(runtime) = &mut data_runtime {
        let _ = runtime.tick();
    }
    let mut tree = build_bound_tree(&spec, state.data(), state.actions(), state.forms()).map_err(
        |source| {
            format_operation_error(
                "run",
                ErrorCategory::Runtime,
                "data-bindings",
                Some(path),
                source.to_string(),
                "inspect data bindings and fallback props, then retry after fixing the DSL",
            )
        },
    )?;
    let _ = focus_next_component(&mut tree, &mut state, true);
    let mut layout = render_tree_with_layout(&tree, viewport, &renderer).map_err(|source| {
        format_operation_error(
            "run",
            ErrorCategory::Render,
            "initial-render",
            Some(path),
            source.to_string(),
            "confirm the component tree fits the current terminal size, then retry the render path",
        )
    })?;

    let runtime_result = runtime.run(|event| {
        let mut event_ctx = EventContext::default();
        let mut data_changed = false;
        let mut action_changed = false;
        let mut form_changed = false;
        if let Some(data_runtime) = &mut data_runtime {
            let updates = if matches!(event, Event::Tick) {
                data_runtime.tick()
            } else {
                data_runtime.poll()
            };
            for update in updates {
                data_changed |= state.set_data_snapshot(update.source_id, update.snapshot);
            }
            if data_changed {
                match build_bound_tree(&spec, state.data(), state.actions(), state.forms()) {
                    Ok(next_tree) => {
                        tree = next_tree;
                        if let Some(focused) = state.focused().cloned() {
                            let _ = tree.dispatch_event_to_target(
                                &mut event_ctx,
                                &focused,
                                &Event::FocusGained(focused.clone()),
                            );
                        }
                    }
                    Err(source) => {
                        render_error = Some(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            source.to_string(),
                        ));
                        return EventResult::Command(AppCommand::Quit);
                    }
                }
            }
        }

        if let Some(action_runtime) = &mut action_runtime {
            for update in action_runtime.poll() {
                action_changed |= state.set_action_snapshot(
                    update.action_id.clone(),
                    ActionSnapshot::from_runtime_update(&update),
                );
                if matches!(update.status, ActionStatus::Ready) {
                    if let Some(data_runtime) = &mut data_runtime {
                        for data_update in data_runtime.refresh_sources(&update.refresh_sources) {
                            data_changed |= state
                                .set_data_snapshot(data_update.source_id, data_update.snapshot);
                        }
                    }
                }
            }
        }

        if data_changed || action_changed {
            match build_bound_tree(&spec, state.data(), state.actions(), state.forms()) {
                Ok(next_tree) => {
                    tree = next_tree;
                    if let Some(focused) = state.focused().cloned() {
                        let _ = tree.dispatch_event_to_target(
                            &mut event_ctx,
                            &focused,
                            &Event::FocusGained(focused.clone()),
                        );
                    }
                }
                Err(source) => {
                    render_error = Some(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        source.to_string(),
                    ));
                    return EventResult::Command(AppCommand::Quit);
                }
            }
        }

        let result =
            dispatch_interactive_event(&mut tree, &mut state, &layout, &mut event_ctx, &event);

        let commands = event_commands(&result, &event_ctx);
        for command in commands {
            if let Some((form_id, field_id, value)) = command.form_value_update() {
                form_changed |= state.set_form_value(
                    form_id.to_string(),
                    field_id.to_string(),
                    neotui_core::dsl::Value::String(value.to_string()),
                );
            }

            if let Some(action_id) = command.action_id() {
                if let Some(action_runtime) = &mut action_runtime {
                    if let Some(update) = action_runtime.trigger_with_form_specs(
                        action_id,
                        state.forms(),
                        &spec.forms,
                    ) {
                        action_changed |= state.set_action_snapshot(
                            update.action_id.clone(),
                            ActionSnapshot::from_runtime_update(&update),
                        );
                    }
                }
            }
        }

        if action_changed || form_changed {
            match build_bound_tree(&spec, state.data(), state.actions(), state.forms()) {
                Ok(next_tree) => {
                    tree = next_tree;
                    if let Some(focused) = state.focused().cloned() {
                        let _ = tree.dispatch_event_to_target(
                            &mut event_ctx,
                            &focused,
                            &Event::FocusGained(focused.clone()),
                        );
                    }
                }
                Err(source) => {
                    render_error = Some(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        source.to_string(),
                    ));
                    return EventResult::Command(AppCommand::Quit);
                }
            }
        }

        if let Event::Resize { width, height } = &event {
            viewport = (*width, *height);
        }

        if action_changed
            || data_changed
            || form_changed
            || result.requests_render()
            || event.requests_render()
        {
            match render_tree_with_layout(&tree, viewport, &renderer) {
                Ok(next_layout) => {
                    layout = next_layout;
                }
                Err(source) => {
                    render_error = Some(source);
                    return EventResult::Command(AppCommand::Quit);
                }
            }
        }

        result
    });

    let exit_result = terminal.exit();

    if let Err(source) = runtime_result {
        return Err(format_operation_error(
            "run",
            ErrorCategory::Runtime,
            "event-loop",
            Some(path),
            source.to_string(),
            "re-run with `NEOTUI_DEBUG=1` if you need subsystem tracing for the runtime path",
        ));
    }

    if let Some(source) = render_error {
        return Err(format_operation_error(
            "run",
            ErrorCategory::Render,
            "frame-update",
            Some(path),
            source.to_string(),
            "inspect the last component/event that requested a redraw and retry after fixing the render path",
        ));
    }

    exit_result.map_err(|source| {
        format_operation_error(
            "run",
            ErrorCategory::TerminalSession,
            "terminal-exit",
            Some(path),
            source.to_string(),
            "restore the terminal state manually if needed, then re-run after checking terminal compatibility",
        )
    })
}

fn dispatch_interactive_event(
    tree: &mut ComponentTree,
    state: &mut StateStore,
    layout: &LayoutNode,
    event_ctx: &mut EventContext,
    event: &Event,
) -> EventResult {
    match event {
        Event::Key(key) if matches!(key.code, KeyCode::Tab) => {
            focus_next_component(tree, state, !key.modifiers.shift)
        }
        Event::Key(_) => match state.focused().cloned() {
            Some(focused) => tree.dispatch_event_to_target(event_ctx, &focused, event),
            None => tree.dispatch_event(event_ctx, event),
        },
        Event::Scroll(_) => match tree.resolve_scroll_target(layout, state.focused(), event) {
            Some(target) => tree.dispatch_event_to_target(event_ctx, &target, event),
            None => EventResult::Ignored,
        },
        Event::Mouse(_) => tree.dispatch_mouse_event(event_ctx, layout, event),
        _ => tree.dispatch_event(event_ctx, event),
    }
}

fn focus_next_component(
    tree: &mut ComponentTree,
    state: &mut StateStore,
    forward: bool,
) -> EventResult {
    let focus_order = tree.focusable_ids_depth_first();
    let previous = state.focused().cloned();
    let next = if forward {
        state.focus_next(&focus_order)
    } else {
        state.focus_previous(&focus_order)
    };

    let Some(next) = next else {
        return EventResult::Ignored;
    };

    let mut event_ctx = EventContext::default();
    if let Some(previous) = previous.filter(|previous| previous != &next) {
        let _ = tree.dispatch_event_to_target(
            &mut event_ctx,
            &previous,
            &Event::FocusLost(previous.clone()),
        );
    }
    let _ = tree.dispatch_event_to_target(&mut event_ctx, &next, &Event::FocusGained(next.clone()));

    EventResult::RequestRender
}

fn event_commands(result: &EventResult, ctx: &EventContext) -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if let Some(command) = result.command() {
        commands.push(command.clone());
    }
    if let Some(command) = result.bubbled_command() {
        commands.push(command.clone());
    }
    for command in &ctx.commands {
        if !commands.contains(command) {
            commands.push(command.clone());
        }
    }
    commands
}

fn build_bound_tree(
    spec: &AppSpec,
    data: &DataStore,
    actions: &ActionStore,
    forms: &FormStore,
) -> Result<ComponentTree, neotui_core::registry::RegistryError> {
    let effective_spec = apply_runtime_bindings_with_forms(spec, data, actions, forms);
    ComponentRegistry::new().build_tree(&effective_spec)
}

fn run_file_gui(
    path: &Path,
    gui_cli_program: Option<&str>,
    gui_working_directory: Option<&str>,
    gui_forward_args: &[String],
) -> Result<(), String> {
    load_app(path).map_err(|error| error.render_for("run"))?;
    debug!(
        target: "neotui::cli",
        path = %path.display(),
        "starting GUI bridge flow"
    );
    let forwarding_contract =
        resolve_gui_forwarding_contract(gui_cli_program, gui_working_directory, gui_forward_args)?;
    let gui_binary_program = resolve_gui_binary_program()?;

    let options = neotui_gui::prepare_gui_launch(path).map_err(|source| {
        let retry_examples = gui_launch_retry_examples(path);
        format_operation_error(
            "run --gui",
            ErrorCategory::GuiEnvironment,
            "prepare-launch",
            Some(path),
            source.to_string(),
            format!(
                "run `neotui doctor` to inspect Linux GUI readiness before retrying one of:\n{}\n{}",
                retry_examples.0, retry_examples.1
            ),
        )
    })?;

    let options = apply_gui_forwarding_contract(options, &forwarding_contract);
    let invocation = build_gui_binary_invocation(gui_binary_program, &options);
    debug!(
        target: "neotui::cli",
        gui_binary = %invocation.program.display(),
        forwarded_arg_count = invocation.args.len().saturating_sub(1),
        "spawning dedicated GUI binary"
    );

    let status = ProcessCommand::new(&invocation.program)
        .args(&invocation.args)
        .status()
        .map_err(|source| {
            let retry_examples = gui_launch_retry_examples(path);
            format_operation_error(
                "run --gui",
                ErrorCategory::GuiBridge,
                "spawn-gui-binary",
                Some(path),
                format!(
                    "failed to spawn `{}`: {source}",
                    invocation.program.display()
                ),
                format!(
                    "ensure the `neotui-gui` binary is available next to `neotui` or on PATH, then retry one of:\n{}\n{}",
                    retry_examples.0, retry_examples.1
                ),
            )
        })?;

    if status.success() {
        return Ok(());
    }

    Err({
        let retry_examples = gui_launch_retry_examples(path);
        format_operation_error(
            "run --gui",
            ErrorCategory::GuiLaunch,
            "gui-process-exit",
            Some(path),
            format!(
                "`neotui-gui` exited unsuccessfully with status {}",
                display_exit_status(&status)
            ),
            format!(
                "run `neotui doctor` and confirm GTK/VTE prerequisites plus graphical-session availability before retrying one of:\n{}\n{}",
                retry_examples.0, retry_examples.1
            ),
        )
    })
}

fn resolve_gui_forwarding_contract(
    gui_cli_program: Option<&str>,
    gui_working_directory: Option<&str>,
    gui_forward_args: &[String],
) -> Result<GuiForwardingContract, String> {
    let cli_program = match gui_cli_program {
        Some(program) => normalized_non_empty_gui_value(
            program,
            "--gui-cli-program",
            "pass the child CLI executable explicitly, for example `--gui-cli-program cargo`",
        )?,
        None => current_cli_program_for_gui()?,
    };
    let working_directory = gui_working_directory
        .map(|value| {
            normalized_non_empty_gui_value(
                value,
                "--gui-working-directory",
                "pass a workspace-relative or absolute path, for example `--gui-working-directory crates/neotui-cli`",
            )
        })
        .transpose()?;
    let forwarded_run_args = gui_forward_args
        .iter()
        .map(|value| {
            normalized_non_empty_gui_value(
                value,
                "--gui-forward-arg",
                "repeat the flag per forwarded argument, for example `--gui-forward-arg --release`",
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let contract = GuiForwardingContract {
        cli_program,
        working_directory,
        forwarded_run_args,
    };
    debug!(
        target: "neotui::cli",
        has_working_directory = contract.working_directory.is_some(),
        forwarded_arg_count = contract.forwarded_run_args.len(),
        "resolved GUI forwarding contract"
    );

    Ok(contract)
}

fn apply_gui_forwarding_contract(
    options: neotui_gui::GuiLaunchOptions,
    contract: &GuiForwardingContract,
) -> neotui_gui::GuiLaunchOptions {
    let mut options = options.with_cli_program(contract.cli_program.clone());

    if let Some(working_directory) = &contract.working_directory {
        options = options.with_working_directory(working_directory);
    }

    if !contract.forwarded_run_args.is_empty() {
        options = options.with_extra_cli_args(contract.forwarded_run_args.iter().cloned());
    }

    options
}

fn current_cli_program_for_gui() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|source| {
            format_operation_error(
                "run --gui",
                ErrorCategory::GuiConfig,
                "resolve-cli-program",
                None,
                source.to_string(),
                "pass `--gui-cli-program <program>` explicitly if the current executable path is unavailable",
            )
        })
}

fn resolve_gui_binary_program() -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe().map_err(|source| {
        format_operation_error(
            "run --gui",
            ErrorCategory::GuiBridge,
            "resolve-gui-binary",
            None,
            source.to_string(),
            "ensure the GUI binary is installed next to the CLI or reachable on PATH",
        )
    })?;
    let sibling = sibling_gui_binary_path(&current_exe);

    if sibling.exists() {
        Ok(sibling)
    } else {
        Ok(PathBuf::from(gui_binary_file_name_for(&current_exe)))
    }
}

fn sibling_gui_binary_path(current_exe: &Path) -> PathBuf {
    let binary_name = gui_binary_file_name_for(current_exe);
    current_exe
        .parent()
        .map(|directory| directory.join(&binary_name))
        .unwrap_or_else(|| PathBuf::from(binary_name))
}

fn gui_binary_file_name_for(current_exe: &Path) -> String {
    match current_exe.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if !ext.is_empty() => format!("neotui-gui.{ext}"),
        _ => "neotui-gui".into(),
    }
}

fn build_gui_binary_invocation(
    gui_binary_program: PathBuf,
    options: &neotui_gui::GuiLaunchOptions,
) -> GuiBinaryInvocation {
    let mut args = vec![options.app_file.display().to_string()];
    args.push("--cli-program".into());
    args.push(options.cli_program.clone());

    if let Some(working_directory) = &options.working_directory {
        args.push("--working-directory".into());
        args.push(working_directory.display().to_string());
    }

    args.push("--window-title".into());
    args.push(options.window_title.clone());

    for forwarded_arg in &options.extra_cli_args {
        args.push("--forward-arg".into());
        args.push(forwarded_arg.clone());
    }

    GuiBinaryInvocation {
        program: gui_binary_program,
        args,
    }
}

fn display_exit_status(status: &std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "terminated by signal".into())
}

fn normalized_non_empty_gui_value(
    value: &str,
    flag: &str,
    usage_hint: &str,
) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format_operation_error(
            "run --gui",
            ErrorCategory::GuiConfig,
            "validate-gui-flag",
            None,
            format!("{flag} received an empty value"),
            usage_hint,
        ));
    }

    Ok(trimmed.to_string())
}

fn gui_launch_retry_examples(path: &Path) -> (String, String) {
    (
        format!("neotui run {} --gui", path.display()),
        format!(
            "neotui run {} --gui --gui-cli-program cargo --gui-forward-arg --release",
            path.display()
        ),
    )
}

fn cli_command_name(command: &Command) -> &'static str {
    match command {
        Command::Run { .. } => "run",
        Command::Check { .. } => "check",
        Command::Doctor => "doctor",
    }
}

#[allow(dead_code)]
fn render_tree(
    tree: &ComponentTree,
    viewport: (u16, u16),
    renderer: &AnsiRenderer,
) -> std::io::Result<()> {
    render_tree_with_layout(tree, viewport, renderer).map(|_| ())
}

fn render_tree_with_layout(
    tree: &ComponentTree,
    viewport: (u16, u16),
    renderer: &AnsiRenderer,
) -> std::io::Result<LayoutNode> {
    let area = Rect::new(0, 0, viewport.0, viewport.1);
    let mut frame = ScreenBuffer::new(viewport.0, viewport.1);
    let layout = tree.layout(&LayoutContext, area);

    tree.render_with_layout(&layout, &mut frame);
    renderer.render_to_stdout(&frame)?;
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use neotui_core::event::{ComponentId, KeyEvent, KeyModifiers, ScrollDirection, ScrollEvent};
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
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

    fn fixture_path(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
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
            Command::Run {
                file,
                gui,
                gui_cli_program,
                gui_working_directory,
                gui_forward_args,
            } => {
                assert_eq!(file, "examples/hello.toml");
                assert!(!gui);
                assert!(gui_cli_program.is_none());
                assert!(gui_working_directory.is_none());
                assert!(gui_forward_args.is_empty());
            }
            Command::Check { .. } => panic!("unexpected check command"),
            Command::Doctor => panic!("unexpected doctor command"),
        }
    }

    #[test]
    fn clap_parses_run_gui_command() {
        let cli = Cli::parse_from([
            "neotui",
            "run",
            "examples/hello.toml",
            "--gui",
            "--gui-cli-program",
            "cargo",
            "--gui-working-directory",
            "crates/neotui-cli",
            "--gui-forward-arg",
            "--release",
            "--gui-forward-arg",
            "--locked",
        ]);

        match cli.command {
            Command::Run {
                file,
                gui,
                gui_cli_program,
                gui_working_directory,
                gui_forward_args,
            } => {
                assert_eq!(file, "examples/hello.toml");
                assert!(gui);
                assert_eq!(gui_cli_program.as_deref(), Some("cargo"));
                assert_eq!(gui_working_directory.as_deref(), Some("crates/neotui-cli"));
                assert_eq!(gui_forward_args, vec!["--release", "--locked"]);
            }
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
    fn clap_help_mentions_gui_launch_controls() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("run")
            .expect("run subcommand should be registered")
            .render_long_help()
            .to_string();

        assert!(help.contains("--gui"));
        assert!(help.contains("--gui-cli-program"));
        assert!(help.contains("--gui-working-directory"));
        assert!(help.contains("--gui-forward-arg"));
        assert!(help.contains("embedded GTK/VTE GUI"));
    }

    #[test]
    fn clap_rejects_gui_forwarding_flags_without_gui_mode() {
        let error = Cli::try_parse_from([
            "neotui",
            "run",
            "examples/hello.toml",
            "--gui-forward-arg",
            "--release",
        ])
        .expect_err("gui forwarding flags should require --gui");

        let message = error.to_string();
        assert!(message.contains("--gui"));
        assert!(message.contains("--gui-forward-arg"));
    }

    #[test]
    fn doctor_report_formats_without_sensitive_env_values() {
        let output = format_doctor_report(DoctorReport {
            backend: "crossterm",
            stdin_tty: true,
            stdout_tty: false,
            terminal_size: Some((120, 40)),
            terminal_size_class: "comfortable",
            terminal_family: "unknown",
            color_support: "truecolor",
            mouse_support: "unavailable",
            raw_mode_support: "unavailable",
            alternate_screen_support: "likely",
            gui_support: "linux-only-runtime",
            gui_platform_supported: false,
            gui_session_available: false,
            gui_gtk_backend_declared: true,
            gui_vte_backend_declared: true,
            gui_reason: "linux-gtk-vte-only",
            debug_mode: "enabled",
            term_env_present: true,
            colorterm_env_present: false,
            readiness: "degraded",
            hints: vec![
                "run `neotui` inside an interactive terminal session",
                "the MVP GUI path currently targets Linux with GTK/VTE",
            ],
        });

        assert!(output.contains("doctor degraded"));
        assert!(output.contains("backend: crossterm"));
        assert!(output.contains("stdin_tty: yes"));
        assert!(output.contains("stdout_tty: no"));
        assert!(output.contains("terminal_size: 120x40"));
        assert!(output.contains("terminal_size_class: comfortable"));
        assert!(output.contains("terminal_family: unknown"));
        assert!(output.contains("color_support: truecolor"));
        assert!(output.contains("mouse_support: unavailable"));
        assert!(output.contains("raw_mode_support: unavailable"));
        assert!(output.contains("alternate_screen_support: likely"));
        assert!(output.contains("gui_support: linux-only-runtime"));
        assert!(output.contains("gui_platform_supported: no"));
        assert!(output.contains("gui_session_available: no"));
        assert!(output.contains("gui_gtk_backend_declared: yes"));
        assert!(output.contains("gui_vte_backend_declared: yes"));
        assert!(output.contains("gui_reason: linux-gtk-vte-only"));
        assert!(output.contains("debug_mode: enabled"));
        assert!(output.contains("term_env_present: yes"));
        assert!(output.contains("colorterm_env_present: no"));
        assert!(output.contains("interactive terminal session"));
        assert!(output.contains("avoids printing terminal environment values directly"));
        assert!(!output.contains("xterm"));
    }

    #[test]
    fn doctor_helpers_classify_terminal_signals() {
        let ready_gui = neotui_gui::GuiAvailability {
            platform_supported: true,
            session_available: true,
            gtk_backend_declared: true,
            vte_backend_declared: true,
            reason: "ready-for-embed-loop",
        };
        let missing_session_gui = neotui_gui::GuiAvailability {
            platform_supported: true,
            session_available: false,
            gtk_backend_declared: true,
            vte_backend_declared: true,
            reason: "missing-display-session",
        };

        assert_eq!(
            detect_terminal_family(Some(&std::ffi::OsString::from("xterm-256color"))),
            "xterm-compatible"
        );
        assert_eq!(
            detect_color_support(
                Some(&std::ffi::OsString::from("screen-256color")),
                Some(&std::ffi::OsString::from("truecolor"))
            ),
            "truecolor"
        );
        assert_eq!(
            detect_mouse_support(true, true, "xterm-compatible"),
            "likely"
        );
        assert_eq!(
            detect_raw_mode_support(false, true, "xterm-compatible"),
            "unavailable"
        );
        assert_eq!(detect_alternate_screen_support(true, "unknown"), "unknown");
        assert_eq!(
            detect_debug_mode(Some(&std::ffi::OsString::from("1"))),
            "enabled"
        );
        assert_eq!(classify_terminal_size(Some((30, 8))), "constrained");
        assert_eq!(detect_gui_support(&ready_gui), "gtk-vte-declared");
        assert_eq!(detect_gui_support(&missing_session_gui), "session-missing");
    }

    #[test]
    fn doctor_hints_stay_actionable_and_compact() {
        let hints = collect_doctor_hints(
            false,
            true,
            "constrained",
            &neotui_gui::GuiAvailability {
                platform_supported: false,
                session_available: false,
                gtk_backend_declared: true,
                vte_backend_declared: true,
                reason: "linux-gtk-vte-only",
            },
            "enabled",
        );

        assert_eq!(
            hints,
            vec![
                "run `neotui` inside an interactive terminal session",
                "resize the terminal for a more reliable MVP layout preview",
                "the MVP GUI path currently targets Linux with GTK/VTE",
                "NEOTUI_DEBUG appears enabled, so extra diagnostics may be expected",
            ]
        );
    }

    #[test]
    fn doctor_hints_explain_missing_linux_display_session() {
        let hints = collect_doctor_hints(
            true,
            true,
            "comfortable",
            &neotui_gui::GuiAvailability {
                platform_supported: true,
                session_available: false,
                gtk_backend_declared: true,
                vte_backend_declared: true,
                reason: "missing-display-session",
            },
            "disabled",
        );

        assert_eq!(
            hints,
            vec!["start a Linux graphical session with DISPLAY or WAYLAND_DISPLAY before using `--gui`"]
        );
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
        assert!(output.contains("layout_props: []"));
        assert!(output.contains("component_kinds: [Label=1]"));
        assert!(output.contains("component_ids: [root]"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_app_builds_component_tree_for_valid_fixture() {
        let app =
            load_app(&fixture_path("examples/hello.toml")).expect("hello fixture should load");

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
        assert!(error.contains("category: input"));
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
        assert!(error.contains("category: dsl-validation"));
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
        assert!(error.contains("category: dsl-parse"));
        assert!(error.contains("format: toml"));
        assert!(error.contains("failed to parse TOML DSL"));
        assert!(error.contains("fix the file syntax first"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn check_file_rejects_invalid_button_props_with_actionable_validation() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root]
kind = "Button"
"#,
        );

        let error = check_file(&path).expect_err("button without text should fail validation");

        assert!(error.contains("phase: validate"));
        assert!(error.contains("category: dsl-validation"));
        assert!(error.contains("root: `Button`"));
        assert!(error.contains("schema validation failed"));
        assert!(error.contains("missing required property `text`"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn run_path_renders_load_errors_with_run_operation_label() {
        let path = write_temp_file(
            "toml",
            r#"
schema_version = "0.1"

[root]
kind = "Button"
"#,
        );

        let error = load_app(&path)
            .expect_err("invalid button should fail")
            .render_for("run");

        assert!(error.contains("run failed"));
        assert!(error.contains("category: dsl-validation"));
        assert!(error.contains("hint: fix the invalid fields above and re-run `neotui run"));
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
    fn run_gui_returns_failure_exit_code_when_gui_runtime_is_unavailable() {
        let exit = run([
            "neotui".to_string(),
            "run".to_string(),
            "examples/hello.toml".to_string(),
            "--gui".to_string(),
            "--gui-cli-program".to_string(),
            "cargo".to_string(),
        ]);

        if cfg!(target_os = "linux")
            && (std::env::var_os("DISPLAY").is_some()
                || std::env::var_os("WAYLAND_DISPLAY").is_some())
        {
            return;
        }

        assert_eq!(exit, ExitCode::from(1));
    }

    #[test]
    fn run_file_gui_surfaces_doctor_hint_when_prepare_fails() {
        if cfg!(target_os = "linux")
            && (std::env::var_os("DISPLAY").is_some()
                || std::env::var_os("WAYLAND_DISPLAY").is_some())
        {
            return;
        }

        let error = run_file_gui(Path::new("examples/hello.toml"), Some("cargo"), None, &[])
            .expect_err("headless or non-linux environment should not launch GUI");

        assert!(error.contains("run --gui failed"));
        assert!(error.contains("category: gui-environment"));
        assert!(error.contains("phase: prepare-launch"));
        assert!(error.contains("run `neotui doctor`"));
        assert!(error.contains("`--gui`"));
    }

    #[test]
    fn current_cli_program_for_gui_returns_non_empty_path() {
        let program = current_cli_program_for_gui().expect("current exe should resolve in tests");

        assert!(!program.trim().is_empty());
    }

    #[test]
    fn resolve_gui_forwarding_contract_defaults_to_current_executable() {
        let contract = resolve_gui_forwarding_contract(None, None, &[])
            .expect("default contract should resolve");

        assert!(!contract.cli_program.trim().is_empty());
        assert!(contract.working_directory.is_none());
        assert!(contract.forwarded_run_args.is_empty());
    }

    #[test]
    fn resolve_gui_forwarding_contract_preserves_explicit_overrides() {
        let forwarded_args = vec!["--release".to_string(), "--locked".to_string()];
        let contract = resolve_gui_forwarding_contract(
            Some("cargo"),
            Some("crates/neotui-cli"),
            &forwarded_args,
        )
        .expect("explicit contract should resolve");

        assert_eq!(contract.cli_program, "cargo");
        assert_eq!(
            contract.working_directory.as_deref(),
            Some("crates/neotui-cli")
        );
        assert_eq!(contract.forwarded_run_args, forwarded_args);
    }

    #[test]
    fn resolve_gui_forwarding_contract_rejects_empty_explicit_values() {
        let empty_program = resolve_gui_forwarding_contract(Some("   "), None, &[])
            .expect_err("empty gui cli program should fail");
        let empty_workdir = resolve_gui_forwarding_contract(Some("cargo"), Some("  "), &[])
            .expect_err("empty gui working directory should fail");
        let empty_forward_arg =
            resolve_gui_forwarding_contract(Some("cargo"), None, &[String::from("   ")])
                .expect_err("empty forwarded arg should fail");

        assert!(empty_program.contains("--gui-cli-program"));
        assert!(empty_program.contains("category: gui-config"));
        assert!(empty_workdir.contains("--gui-working-directory"));
        assert!(empty_workdir.contains("phase: validate-gui-flag"));
        assert!(empty_forward_arg.contains("--gui-forward-arg"));
    }

    #[test]
    fn apply_gui_forwarding_contract_shapes_launch_options_without_duplication() {
        let options = neotui_gui::GuiLaunchOptions::new("examples/hello.toml");
        let contract = GuiForwardingContract {
            cli_program: "cargo".into(),
            working_directory: Some("crates/neotui-cli".into()),
            forwarded_run_args: vec!["--release".into(), "--locked".into()],
        };
        let applied = apply_gui_forwarding_contract(options, &contract);

        assert_eq!(applied.cli_program, "cargo");
        assert_eq!(
            applied.working_directory.as_deref(),
            Some(Path::new("crates/neotui-cli"))
        );
        assert_eq!(applied.extra_cli_args, vec!["--release", "--locked"]);
    }

    #[test]
    fn gui_binary_file_name_matches_current_extension() {
        assert_eq!(
            gui_binary_file_name_for(Path::new("C:/tools/neotui.exe")),
            "neotui-gui.exe"
        );
        assert_eq!(
            gui_binary_file_name_for(Path::new("/usr/local/bin/neotui")),
            "neotui-gui"
        );
    }

    #[test]
    fn sibling_gui_binary_path_reuses_current_executable_directory() {
        let sibling = sibling_gui_binary_path(Path::new("C:/tools/neotui.exe"));

        assert_eq!(sibling, PathBuf::from("C:/tools/neotui-gui.exe"));
    }

    #[test]
    fn build_gui_binary_invocation_shapes_expected_process_args() {
        let options = neotui_gui::GuiLaunchOptions::new("examples/dashboard.toml")
            .with_cli_program("cargo")
            .with_window_title("NeoTUI Dashboard")
            .with_working_directory("crates/neotui-cli")
            .with_extra_cli_args(["--release", "--locked"]);
        let invocation = build_gui_binary_invocation(PathBuf::from("neotui-gui"), &options);

        assert_eq!(invocation.program, PathBuf::from("neotui-gui"));
        assert_eq!(
            invocation.args,
            vec![
                "examples/dashboard.toml",
                "--cli-program",
                "cargo",
                "--working-directory",
                "crates/neotui-cli",
                "--window-title",
                "NeoTUI Dashboard",
                "--forward-arg",
                "--release",
                "--forward-arg",
                "--locked",
            ]
        );
    }

    #[test]
    fn gui_launch_retry_examples_cover_runtime_and_dev_paths() {
        let examples = gui_launch_retry_examples(Path::new("examples/dashboard.toml"));

        assert_eq!(examples.0, "neotui run examples/dashboard.toml --gui");
        assert!(examples.1.contains("--gui-cli-program cargo"));
        assert!(examples.1.contains("--gui-forward-arg --release"));
    }

    #[test]
    fn display_exit_status_prefers_numeric_codes() {
        let success = std::process::ExitStatus::from_raw(0);

        assert_eq!(display_exit_status(&success), "0");
    }

    #[test]
    fn render_tree_draws_hello_fixture_text() {
        let LoadedApp { tree, .. } =
            load_app(&fixture_path("examples/hello.toml")).expect("hello fixture should load");
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
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/dashboard.toml"))
            .expect("dashboard fixture should load");
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
        let toml_output = check_file(&fixture_path("examples/dashboard.toml"))
            .expect("dashboard.toml should validate");
        let json_output = check_file(&fixture_path("examples/dashboard.json"))
            .expect("dashboard.json should validate");
        let theme_output = check_file(&fixture_path("examples/theme-demo.toml"))
            .expect("theme-demo.toml should validate");
        let layout_output = check_file(&fixture_path("examples/layout-demo.toml"))
            .expect("layout-demo.toml should validate");
        let dense_layout_output = check_file(&fixture_path("examples/layout-dense.toml"))
            .expect("layout-dense.toml should validate");
        let sidebar_layout_output = check_file(&fixture_path("examples/layout-sidebar.toml"))
            .expect("layout-sidebar.toml should validate");
        let responsive_layout_output = check_file(&fixture_path("examples/layout-responsive.toml"))
            .expect("layout-responsive.toml should validate");
        let interactive_output = check_file(&fixture_path("examples/interactive-flow.toml"))
            .expect("interactive-flow.toml should validate");
        let list_output = check_file(&fixture_path("examples/list-demo.toml"))
            .expect("list-demo.toml should validate");
        let rich_output = check_file(&fixture_path("examples/rich-dashboard.toml"))
            .expect("rich-dashboard.toml should validate");
        let redline_output = check_file(&fixture_path("examples/redline-dashboard.toml"))
            .expect("redline-dashboard.toml should validate");
        let table_output = check_file(&fixture_path("examples/table-demo.toml"))
            .expect("table-demo.toml should validate");
        let http_output = check_file(&fixture_path("examples/http-dashboard.toml"))
            .expect("http-dashboard.toml should validate");
        let form_output = check_file(&fixture_path("examples/form-intent.toml"))
            .expect("form-intent.toml should validate");
        let device_output = check_file(&fixture_path("examples/embedded-device-control.toml"))
            .expect("embedded-device-control.toml should validate");
        let python_form_json_output = check_file(&fixture_path("examples/python/form-intent.json"))
            .expect("Python form-intent.json should validate");
        let showcase_output = check_file(&fixture_path("examples/showcase-layout.toml"))
            .expect("showcase-layout.toml should validate");
        let operational_template_output =
            check_file(&fixture_path("templates/operational-dashboard.toml"))
                .expect("operational dashboard template should validate");
        let task_template_output = check_file(&fixture_path("templates/task-list.toml"))
            .expect("task template should validate");
        let metrics_template_output = check_file(&fixture_path("templates/metrics-monitor.toml"))
            .expect("metrics monitor template should validate");

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
        assert!(dense_layout_output.contains("root: `Panel`"));
        assert!(dense_layout_output.contains("component_kinds: [Button=2"));
        assert!(dense_layout_output.contains("TextBlock=1"));
        assert!(sidebar_layout_output.contains("root: `Panel`"));
        assert!(sidebar_layout_output.contains("layout_props: [align=3, fixed=3, gap=2, grow=1]"));
        assert!(responsive_layout_output.contains("root: `VBox`"));
        assert!(responsive_layout_output
            .contains("layout_props: [align=2, fixed=4, gap=2, grow=1, justify=1]"));
        assert!(interactive_output.contains("root: `Panel`"));
        assert!(interactive_output.contains("component_kinds: [Button=2"));
        assert!(interactive_output.contains("List=1"));
        assert!(interactive_output.contains("TextBlock=1"));
        assert!(list_output.contains("root: `Panel`"));
        assert!(list_output.contains("component_count: 4"));
        assert!(list_output.contains("component_kinds: [Divider=1, Label=1, List=1, Panel=1]"));
        assert!(rich_output.contains("root: `Panel`"));
        assert!(rich_output.contains("component_count: 22"));
        assert!(rich_output.contains("container_components: 11"));
        assert!(rich_output.contains("leaf_components: 11"));
        assert!(rich_output.contains("Button=3"));
        assert!(rich_output.contains("Graph=1"));
        assert!(rich_output.contains("List=1"));
        assert!(rich_output.contains("TextBlock=1"));
        assert!(redline_output.contains("theme: `redline`"));
        assert!(redline_output.contains("root: `Panel`"));
        assert!(redline_output.contains("Button=3"));
        assert!(redline_output.contains("Graph=1"));
        assert!(redline_output.contains("List=1"));
        assert!(table_output.contains("theme: `redline`"));
        assert!(table_output.contains("Table=1"));
        assert!(table_output.contains("component_ids: [table-demo"));
        assert!(http_output.contains("theme: `redline`"));
        assert!(http_output.contains("Button=1"));
        assert!(http_output.contains("StatusStrip=2"));
        assert!(form_output.contains("root: `Panel`"));
        assert!(form_output.contains("TextInput=1"));
        assert!(form_output.contains("TextBlock=1"));
        assert!(form_output.contains("StatusStrip=1"));
        assert!(form_output.contains("Button=1"));
        assert!(device_output.contains("theme: `redline`"));
        assert!(device_output.contains("TextInput=2"));
        assert!(device_output.contains("Button=2"));
        assert!(device_output.contains("Table=1"));
        assert!(device_output.contains("StatusStrip=3"));
        assert!(python_form_json_output.contains("format: json"));
        assert!(python_form_json_output.contains("root: `Panel`"));
        assert!(python_form_json_output.contains("TextInput=1"));
        assert!(python_form_json_output.contains("Button=1"));
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
        assert!(operational_template_output.contains("root: `Panel`"));
        assert!(operational_template_output.contains("Graph=1"));
        assert!(operational_template_output.contains("List=1"));
        assert!(task_template_output.contains("root: `Panel`"));
        assert!(task_template_output.contains("TextBlock=1"));
        assert!(metrics_template_output.contains("root: `Panel`"));
        assert!(metrics_template_output.contains("Graph=1"));
    }

    #[test]
    fn render_tree_supports_nested_vbox_and_hbox_layouts() {
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/layout-demo.toml"))
            .expect("layout fixture should load");
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
        let rendered = (0..4)
            .map(|y| {
                (0..20)
                    .map(|x| frame.get(x, y).map(|cell| cell.symbol).unwrap_or(' '))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(header_row.contains("Layout Dem"));
        assert!(gap_row.trim().is_empty());
        assert!(rendered.contains("Left"));
        assert!(rendered.contains("Righ"));
    }

    #[test]
    fn render_tree_supports_showcase_layout_example() {
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/showcase-layout.toml"))
            .expect("showcase layout fixture should load");
        let area = Rect::new(0, 0, 40, 10);
        let mut frame = ScreenBuffer::new(40, 10);
        let layout = tree.layout(&LayoutContext, area);

        assert_eq!(layout.children[0].area, Rect::new(1, 1, 38, 8));
        assert_eq!(layout.children[0].children[0].area, Rect::new(11, 1, 18, 1));
        assert_eq!(layout.children[0].children[2].area, Rect::new(1, 5, 38, 2));
        assert_eq!(
            layout.children[0].children[2].children[0].area,
            Rect::new(6, 5, 8, 1)
        );
        assert_eq!(
            layout.children[0].children[2].children[1].area,
            Rect::new(16, 5, 8, 1)
        );
        assert_eq!(
            layout.children[0].children[2].children[2].area,
            Rect::new(26, 5, 8, 1)
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

    #[test]
    fn render_tree_supports_rich_dashboard_example() {
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/rich-dashboard.toml"))
            .expect("rich dashboard fixture should load");
        let area = Rect::new(0, 0, 90, 22);
        let mut frame = ScreenBuffer::new(90, 22);
        let layout = tree.layout(&LayoutContext, area);

        tree.render_with_layout(&layout, &mut frame);

        let rendered = (0..22)
            .map(|y| {
                (0..90)
                    .map(|x| frame.get(x, y).map(|cell| cell.symbol).unwrap_or(' '))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Production Overview"));
        assert!(rendered.contains("API"));
        assert!(rendered.contains("Service Queue"));
        assert!(rendered.contains("Throughput"));
        assert!(rendered.contains("Operator Notes"));
        assert!(rendered.contains("[ Deploy ]"));
    }

    #[test]
    fn render_tree_supports_layout_pattern_examples() {
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/layout-sidebar.toml"))
            .expect("sidebar layout fixture should load");
        let area = Rect::new(0, 0, 70, 10);
        let mut frame = ScreenBuffer::new(70, 10);
        let layout = tree.layout(&LayoutContext, area);

        tree.render_with_layout(&layout, &mut frame);

        let rendered = (0..10)
            .map(|y| {
                (0..70)
                    .map(|x| frame.get(x, y).map(|cell| cell.symbol).unwrap_or(' '))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Sections"));
        assert!(rendered.contains("Detail"));
        assert!(rendered.contains("Overview"));
        assert!(rendered.contains("Selected: Overview"));
    }

    #[test]
    fn render_tree_supports_interactive_flow_example() {
        let LoadedApp { tree, .. } = load_app(&fixture_path("examples/interactive-flow.toml"))
            .expect("interactive flow fixture should load");
        let area = Rect::new(0, 0, 92, 16);
        let mut frame = ScreenBuffer::new(92, 16);
        let layout = tree.layout(&LayoutContext, area);

        tree.render_with_layout(&layout, &mut frame);

        let rendered = (0..16)
            .map(|y| {
                (0..92)
                    .map(|x| frame.get(x, y).map(|cell| cell.symbol).unwrap_or(' '))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Interactive Flow"));
        assert!(rendered.contains("Queue"));
        assert!(rendered.contains("Incident triage"));
        assert!(rendered.contains("[ Approve ]"));
        assert!(rendered.contains("[ Defer ]"));
    }

    #[test]
    fn interactive_dispatch_cycles_focus_and_routes_events() {
        let LoadedApp { mut tree, .. } = load_app(&fixture_path("examples/interactive-flow.toml"))
            .expect("interactive flow fixture should load");
        let mut state = StateStore::new();
        let area = Rect::new(0, 0, 92, 16);
        let layout = tree.layout(&LayoutContext, area);
        let mut ctx = EventContext::default();

        assert_eq!(
            focus_next_component(&mut tree, &mut state, true),
            EventResult::RequestRender
        );
        assert_eq!(state.focused(), Some(&ComponentId("queue-list".into())));
        assert_eq!(
            dispatch_interactive_event(
                &mut tree,
                &mut state,
                &layout,
                &mut ctx,
                &Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::default(),
                }),
            ),
            EventResult::RequestRender
        );
        assert_eq!(
            dispatch_interactive_event(
                &mut tree,
                &mut state,
                &layout,
                &mut ctx,
                &Event::Scroll(ScrollEvent {
                    direction: ScrollDirection::Down,
                    amount: 1,
                }),
            ),
            EventResult::RequestRender
        );
        assert_eq!(
            dispatch_interactive_event(
                &mut tree,
                &mut state,
                &layout,
                &mut ctx,
                &Event::Key(KeyEvent {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers::default(),
                }),
            ),
            EventResult::RequestRender
        );
        assert_eq!(state.focused(), Some(&ComponentId("approve-action".into())));
        assert_eq!(
            dispatch_interactive_event(
                &mut tree,
                &mut state,
                &layout,
                &mut ctx,
                &Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::default(),
                }),
            ),
            EventResult::RequestRender
        );
    }

    #[test]
    fn form_input_updates_action_payload_for_embedded_device_example() {
        let LoadedApp { spec, .. } =
            load_app(&fixture_path("examples/embedded-device-control.toml"))
                .expect("embedded device fixture should load");
        let mut state = StateStore::new();
        let _ = state.initialize_forms(&spec.forms);
        let mut tree = build_bound_tree(&spec, state.data(), state.actions(), state.forms())
            .expect("bound fixture should instantiate");
        let area = Rect::new(0, 0, 144, 36);
        let mut layout = tree.layout(&LayoutContext, area);
        let target = ComponentId("mode-input".into());

        for _ in 0..tree.focusable_ids_depth_first().len() {
            if state.focused() == Some(&target) {
                break;
            }
            assert_eq!(
                focus_next_component(&mut tree, &mut state, true),
                EventResult::RequestRender
            );
        }
        assert_eq!(state.focused(), Some(&target));

        for _ in 0.."maintenance-window".chars().count() {
            dispatch_form_key_and_rebuild(
                &spec,
                &mut tree,
                &mut state,
                &mut layout,
                KeyCode::Backspace,
            );
        }
        for ch in "field-test".chars() {
            dispatch_form_key_and_rebuild(
                &spec,
                &mut tree,
                &mut state,
                &mut layout,
                KeyCode::Char(ch),
            );
        }

        assert_eq!(
            state.forms().get("device", "mode"),
            Some(&neotui_core::dsl::Value::String("field-test".into()))
        );

        let action = spec
            .actions
            .iter()
            .find(|action| action.id == "apply_device_config")
            .expect("apply action should exist");
        let rendered = neotui_core::data::render_action_payload(action, state.forms());

        assert_eq!(
            rendered.http.body,
            Some(neotui_core::data::HttpBody::Json(
                neotui_core::dsl::Value::Object(BTreeMap::from([
                    (
                        "hostname".into(),
                        neotui_core::dsl::Value::String("edge-gateway-07".into())
                    ),
                    (
                        "intent".into(),
                        neotui_core::dsl::Value::String("apply_config".into())
                    ),
                    (
                        "mode".into(),
                        neotui_core::dsl::Value::String("field-test".into())
                    ),
                ]))
            ))
        );
    }

    fn dispatch_form_key_and_rebuild(
        spec: &AppSpec,
        tree: &mut ComponentTree,
        state: &mut StateStore,
        layout: &mut LayoutNode,
        code: KeyCode,
    ) {
        let mut ctx = EventContext::default();
        let result = dispatch_interactive_event(
            tree,
            state,
            layout,
            &mut ctx,
            &Event::Key(KeyEvent {
                code,
                modifiers: KeyModifiers::default(),
            }),
        );
        for command in event_commands(&result, &ctx) {
            if let Some((form_id, field_id, value)) = command.form_value_update() {
                let _ = state.set_form_value(
                    form_id.to_string(),
                    field_id.to_string(),
                    neotui_core::dsl::Value::String(value.to_string()),
                );
            }
        }
        *tree = build_bound_tree(spec, state.data(), state.actions(), state.forms())
            .expect("bound fixture should rebuild after form update");
        if let Some(focused) = state.focused().cloned() {
            let mut focus_ctx = EventContext::default();
            let _ = tree.dispatch_event_to_target(
                &mut focus_ctx,
                &focused,
                &Event::FocusGained(focused.clone()),
            );
        }
        *layout = tree.layout(&LayoutContext, Rect::new(0, 0, 144, 36));
    }
}
