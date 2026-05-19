use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use neotui_core::diagnostics;
use neotui_gui::{launch_embedded_terminal, GuiLaunchOptions};
use tracing::debug;

fn main() -> ExitCode {
    diagnostics::init_tracing();
    match parse_gui_cli_args(std::env::args_os()) {
        Ok(options) => match launch_embedded_terminal(&options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(source) => {
                eprintln!("{source}");
                ExitCode::from(1)
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn parse_gui_cli_args<I, T>(args: I) -> Result<GuiLaunchOptions, String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into);
    let program_name = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "neotui-gui".into());

    let Some(app_file) = args.next() else {
        return Err(usage_message(&program_name));
    };
    let mut options = GuiLaunchOptions::new(PathBuf::from(app_file));
    debug!(
        target: "neotui::gui",
        app_file = %options.app_file.display(),
        "parsed base GUI app file"
    );

    while let Some(argument) = args.next() {
        let argument = argument.to_string_lossy().into_owned();

        match argument.as_str() {
            "--cli-program" => {
                let value = next_value(&mut args, &program_name, "--cli-program")?;
                options = options.with_cli_program(value);
            }
            "--working-directory" => {
                let value = next_value(&mut args, &program_name, "--working-directory")?;
                options = options.with_working_directory(PathBuf::from(value));
            }
            "--window-title" => {
                let value = next_value(&mut args, &program_name, "--window-title")?;
                options = options.with_window_title(value);
            }
            "--forward-arg" => {
                let value = next_value(&mut args, &program_name, "--forward-arg")?;
                options = options.with_extra_cli_args([value]);
            }
            "--help" | "-h" => return Err(usage_message(&program_name)),
            _ => {
                return Err(format!(
                    "unrecognized argument `{argument}`\n{}",
                    usage_message(&program_name)
                ));
            }
        }
    }

    debug!(
        target: "neotui::gui",
        cli_program = options.cli_program.as_str(),
        has_working_directory = options.working_directory.is_some(),
        forwarded_arg_count = options.extra_cli_args.len(),
        "built GUI launch options"
    );

    Ok(options)
}

fn next_value<I>(args: &mut I, program_name: &str, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    let Some(value) = args.next() else {
        return Err(format!(
            "missing value for `{flag}`\n{}",
            usage_message(program_name)
        ));
    };
    let value = value.to_string_lossy().trim().to_string();

    if value.is_empty() {
        return Err(format!(
            "`{flag}` received an empty value\n{}",
            usage_message(program_name)
        ));
    }

    Ok(value)
}

fn usage_message(program_name: &str) -> String {
    format!(
        "usage: {program_name} <app-file> [--cli-program <program>] [--working-directory <path>] [--window-title <title>] [--forward-arg <arg>]...\nexample: {program_name} examples/dashboard.toml --cli-program cargo --forward-arg --release"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parser_requires_app_file() {
        let error = parse_gui_cli_args(["neotui-gui"]).expect_err("app file should be required");

        assert!(error.contains("usage: neotui-gui <app-file>"));
    }

    #[test]
    fn parser_builds_launch_options_with_overrides() {
        let options = parse_gui_cli_args([
            "neotui-gui",
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
        ])
        .expect("valid args should parse");

        assert_eq!(options.cli_program, "cargo");
        assert_eq!(
            options.working_directory.as_deref(),
            Some(Path::new("crates/neotui-cli"))
        );
        assert_eq!(options.window_title, "NeoTUI Dashboard");
        assert_eq!(options.extra_cli_args, vec!["--release", "--locked"]);
    }

    #[test]
    fn parser_rejects_unknown_flag() {
        let error = parse_gui_cli_args(["neotui-gui", "examples/hello.toml", "--unknown"])
            .expect_err("unknown args should fail");

        assert!(error.contains("unrecognized argument `--unknown`"));
    }

    #[test]
    fn parser_rejects_empty_override_value() {
        let error =
            parse_gui_cli_args(["neotui-gui", "examples/hello.toml", "--cli-program", "   "])
                .expect_err("empty override should fail");

        assert!(error.contains("--cli-program"));
        assert!(error.contains("empty value"));
    }
}
