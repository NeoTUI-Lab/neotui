// NeoTUI CLI
// Command-line interface for NeoTUI applications

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use neotui_core::dsl::{AppSpec, DslFormat};

#[derive(Debug, Parser)]
#[command(name = "neotui", version, about = "NeoTUI command-line interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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

fn check_file(path: &Path) -> Result<String, String> {
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

    Ok(format!(
        "check ok: `{}` parsed as {:?} with root `{}`",
        path.display(),
        format,
        spec.root.kind
    ))
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
            Command::Check { file } => assert_eq!(file, "examples/hello.toml"),
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
            "check".to_string(),
            path.to_string_lossy().to_string(),
        ]);

        assert_eq!(exit, ExitCode::from(1));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn check_file_accepts_dashboard_examples() {
        let toml_output = check_file(Path::new("examples/dashboard.toml"))
            .expect("dashboard.toml should validate");
        let json_output = check_file(Path::new("examples/dashboard.json"))
            .expect("dashboard.json should validate");
        let theme_output = check_file(Path::new("examples/theme-demo.toml"))
            .expect("theme-demo.toml should validate");

        assert!(toml_output.contains("root `Panel`"));
        assert!(json_output.contains("root `Panel`"));
        assert!(theme_output.contains("root `Panel`"));
    }
}
