// NeoTUI CLI
// Command-line interface for NeoTUI applications

use std::fs;
use std::path::Path;
use std::process::ExitCode;

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
}

fn main() -> ExitCode {
    run(std::env::args())
}

fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
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
    }
}

struct LoadedApp {
    format: DslFormat,
    spec: AppSpec,
    tree: ComponentTree,
}

fn load_app(path: &Path) -> Result<LoadedApp, String> {
    let format = DslFormat::detect_from_path(&path.to_string_lossy()).ok_or_else(|| {
        format!(
            "unsupported DSL format for `{}`; expected .toml or .json",
            path.display()
        )
    })?;

    let input = fs::read_to_string(path)
        .map_err(|source| format!("failed to read `{}`: {source}", path.display()))?;

    let spec = match format {
        DslFormat::Toml => AppSpec::from_toml_str(&input),
        DslFormat::Json => AppSpec::from_json_str(&input),
    }
    .map_err(|source| format!("invalid DSL in `{}`: {source}", path.display()))?;

    spec.validate()
        .map_err(|errors| format!("validation failed for `{}`:\n{errors}", path.display()))?;

    let tree = ComponentRegistry::new()
        .build_tree(&spec)
        .map_err(|source| format!("failed to instantiate `{}`: {source}", path.display()))?;

    Ok(LoadedApp { format, spec, tree })
}

fn check_file(path: &Path) -> Result<String, String> {
    let LoadedApp { format, spec, .. } = load_app(path)?;

    Ok(format!(
        "check ok: `{}` parsed as {:?} with root `{}`",
        path.display(),
        format,
        spec.root.kind
    ))
}

fn run_file(path: &Path) -> Result<(), String> {
    let LoadedApp { mut tree, .. } = load_app(path)?;
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
        }
    }

    #[test]
    fn clap_parses_run_command() {
        let cli = Cli::parse_from(["neotui", "run", "examples/hello.toml"]);

        match cli.command {
            Command::Run { file } => assert_eq!(file, "examples/hello.toml"),
            Command::Check { .. } => panic!("unexpected check command"),
        }
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

        assert!(output.contains("check ok:"));
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
    fn check_file_rejects_unsupported_extension() {
        let path = write_temp_file("yaml", "schema_version: '0.1'");

        let error = check_file(&path).expect_err("yaml should not be accepted yet");

        assert!(error.contains("unsupported DSL format"));
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

        assert!(error.contains("validation failed"));
        assert!(error.contains("root.props.text"));
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

        assert!(toml_output.contains("root `Panel`"));
        assert!(json_output.contains("root `Panel`"));
        assert!(theme_output.contains("root `Panel`"));
        assert!(layout_output.contains("root `VBox`"));
    }

    #[test]
    fn render_tree_supports_nested_vbox_and_hbox_layouts() {
        let LoadedApp { tree, .. } =
            load_app(Path::new("examples/layout-demo.toml")).expect("layout fixture should load");
        let area = Rect::new(0, 0, 20, 4);
        let mut frame = ScreenBuffer::new(20, 4);
        let layout = tree.layout(&LayoutContext, area);

        assert_eq!(layout.children[0].area, Rect::new(0, 0, 20, 1));
        assert_eq!(layout.children[1].area, Rect::new(0, 2, 20, 2));
        assert_eq!(layout.children[1].children[0].area, Rect::new(0, 2, 6, 2));
        assert_eq!(layout.children[1].children[1].area, Rect::new(8, 2, 12, 2));

        tree.render_with_layout(&layout, &mut frame);

        let header_row: String = (0..20)
            .map(|x| frame.get(x, 0).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let gap_row: String = (0..20)
            .map(|x| frame.get(x, 1).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();
        let columns_row: String = (0..20)
            .map(|x| frame.get(x, 2).map(|cell| cell.symbol).unwrap_or(' '))
            .collect();

        assert!(header_row.contains("Layout Demo"));
        assert!(gap_row.trim().is_empty());
        assert!(columns_row.contains("Left"));
        assert!(columns_row.contains("Right"));
    }
}
