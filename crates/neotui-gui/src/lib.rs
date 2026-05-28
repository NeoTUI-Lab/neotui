// NeoTUI GUI
// Embedded terminal GUI bootstrap surface for the Linux GTK/VTE runtime

use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiLaunchOptions {
    pub app_file: PathBuf,
    pub cli_program: String,
    pub window_title: String,
    pub working_directory: Option<PathBuf>,
    pub extra_cli_args: Vec<String>,
}

impl GuiLaunchOptions {
    pub fn new(app_file: impl Into<PathBuf>) -> Self {
        let app_file = app_file.into();
        Self {
            window_title: default_window_title(&app_file),
            app_file,
            cli_program: "neotui".into(),
            working_directory: None,
            extra_cli_args: Vec::new(),
        }
    }

    pub fn with_cli_program(mut self, cli_program: impl Into<String>) -> Self {
        self.cli_program = cli_program.into();
        self
    }

    pub fn with_window_title(mut self, window_title: impl Into<String>) -> Self {
        self.window_title = window_title.into();
        self
    }

    pub fn with_working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    pub fn with_extra_cli_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extra_cli_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn embedded_command(&self) -> Vec<String> {
        let mut command = vec![self.cli_program.clone(), "run".into()];
        command.push(self.app_file.display().to_string());
        command.extend(self.extra_cli_args.iter().cloned());
        command
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiLaunchPlan {
    application_id: String,
    window_title: String,
    working_directory: Option<String>,
    command: Vec<String>,
    default_width: i32,
    default_height: i32,
}

impl GuiLaunchPlan {
    fn from_options(options: &GuiLaunchOptions) -> Self {
        let app_file = absolutize_app_file(&options.app_file);
        let working_directory = options
            .working_directory
            .as_deref()
            .map(absolutize_app_file)
            .or_else(|| app_file.parent().map(Path::to_path_buf))
            .map(|path| path_to_string(&path));
        let mut command = vec![options.cli_program.clone(), "run".into()];
        command.push(path_to_string(&app_file));
        command.extend(options.extra_cli_args.iter().cloned());

        Self {
            application_id: "dev.neotui.gui".into(),
            window_title: options.window_title.clone(),
            working_directory,
            command,
            default_width: 1200,
            default_height: 800,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiAvailability {
    pub platform_supported: bool,
    pub session_available: bool,
    pub gtk_backend_declared: bool,
    pub vte_backend_declared: bool,
    pub reason: &'static str,
}

impl GuiAvailability {
    pub fn ready(&self) -> bool {
        self.platform_supported
            && self.session_available
            && self.gtk_backend_declared
            && self.vte_backend_declared
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiError {
    UnsupportedPlatform,
    MissingDisplaySession,
    ChildSpawnFailed(String),
    ApplicationRunFailed(u8),
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                write!(
                    f,
                    "NeoTUI GUI is currently supported only on Linux GTK/VTE environments"
                )
            }
            Self::MissingDisplaySession => write!(
                f,
                "no graphical Linux session was detected; set DISPLAY or WAYLAND_DISPLAY first"
            ),
            Self::ChildSpawnFailed(source) => write!(
                f,
                "failed to spawn the embedded NeoTUI terminal session inside VTE: {source}"
            ),
            Self::ApplicationRunFailed(code) => write!(
                f,
                "the GTK application loop exited with a non-success status: {code}"
            ),
        }
    }
}

impl std::error::Error for GuiError {}

pub fn detect_gui_availability() -> GuiAvailability {
    let platform_supported = cfg!(target_os = "linux");
    let session_available = graphical_session_present();
    debug!(
        target: "neotui::gui",
        platform_supported,
        session_available,
        "detected GUI availability"
    );

    availability_from_flags(platform_supported, session_available)
}

pub fn prepare_gui_launch(app_file: impl Into<PathBuf>) -> Result<GuiLaunchOptions, GuiError> {
    let availability = detect_gui_availability();

    if !availability.platform_supported {
        return Err(GuiError::UnsupportedPlatform);
    }

    if !availability.session_available {
        return Err(GuiError::MissingDisplaySession);
    }

    Ok(GuiLaunchOptions::new(app_file))
}

pub fn launch_embedded_terminal(_options: &GuiLaunchOptions) -> Result<(), GuiError> {
    let availability = detect_gui_availability();

    if !availability.platform_supported {
        return Err(GuiError::UnsupportedPlatform);
    }

    if !availability.session_available {
        return Err(GuiError::MissingDisplaySession);
    }

    platform::launch_embedded_terminal(_options)
}

fn availability_from_flags(platform_supported: bool, session_available: bool) -> GuiAvailability {
    let reason = if !platform_supported {
        "linux-gtk-vte-only"
    } else if !session_available {
        "missing-display-session"
    } else {
        "ready-for-embed-loop"
    };

    GuiAvailability {
        platform_supported,
        session_available,
        gtk_backend_declared: true,
        vte_backend_declared: true,
        reason,
    }
}

fn graphical_session_present() -> bool {
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn default_window_title(app_file: &Path) -> String {
    let file_name = app_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("app");
    format!("NeoTUI - {file_name}")
}

fn absolutize_app_file(app_file: &Path) -> PathBuf {
    let path = if app_file.is_absolute() {
        app_file.to_path_buf()
    } else if let Ok(current_dir) = std::env::current_dir() {
        current_dir.join(app_file)
    } else {
        app_file.to_path_buf()
    };

    normalize_path(&path)
}

fn normalize_path(path: &Path) -> PathBuf {
    path.components()
        .filter(|component| !matches!(component, std::path::Component::CurDir))
        .collect()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(target_os = "linux")]
mod platform {
    use super::{GuiError, GuiLaunchOptions, GuiLaunchPlan};
    use std::{cell::RefCell, rc::Rc};

    use gtk::{gio, glib, prelude::*};
    use tracing::debug;
    use vte4::prelude::*;

    pub(super) fn launch_embedded_terminal(options: &GuiLaunchOptions) -> Result<(), GuiError> {
        let plan = GuiLaunchPlan::from_options(options);
        debug!(
            target: "neotui::gui",
            cli_program = options.cli_program.as_str(),
            has_working_directory = options.working_directory.is_some(),
            forwarded_arg_count = options.extra_cli_args.len(),
            "starting GTK/VTE embedded terminal launch"
        );
        let application = gtk::Application::builder()
            .application_id(&plan.application_id)
            .flags(gio::ApplicationFlags::NON_UNIQUE)
            .build();

        let launch_error = Rc::new(RefCell::new(None::<GuiError>));
        let plan_for_activate = plan.clone();
        let error_for_activate = Rc::clone(&launch_error);

        application.connect_activate(move |app| {
            let terminal = vte4::Terminal::new();
            terminal.set_hexpand(true);
            terminal.set_vexpand(true);

            let window = gtk::ApplicationWindow::builder()
                .application(app)
                .title(&plan_for_activate.window_title)
                .default_width(plan_for_activate.default_width)
                .default_height(plan_for_activate.default_height)
                .child(&terminal)
                .build();

            let app_for_exit = app.clone();
            terminal.connect_child_exited(move |_, _| {
                app_for_exit.quit();
            });

            let argv = plan_for_activate
                .command
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let envv: [&str; 0] = [];
            let terminal_for_spawn = terminal.clone();
            let app_for_error = app.clone();
            let error_for_spawn = Rc::clone(&error_for_activate);

            terminal.spawn_async(
                vte4::PtyFlags::DEFAULT,
                plan_for_activate.working_directory.as_deref(),
                &argv,
                &envv,
                glib::SpawnFlags::SEARCH_PATH,
                || {},
                -1,
                None::<&gio::Cancellable>,
                move |result| match result {
                    Ok(pid) => terminal_for_spawn.watch_child(pid),
                    Err(source) => {
                        *error_for_spawn.borrow_mut() =
                            Some(GuiError::ChildSpawnFailed(source.to_string()));
                        app_for_error.quit();
                    }
                },
            );

            window.present();
        });

        let exit_code = application.run_with_args(&["neotui-gui"]);
        if let Some(error) = launch_error.borrow_mut().take() {
            return Err(error);
        }

        if exit_code != glib::ExitCode::SUCCESS {
            return Err(GuiError::ApplicationRunFailed(exit_code.get()));
        }

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::{GuiError, GuiLaunchOptions};

    pub(super) fn launch_embedded_terminal(_options: &GuiLaunchOptions) -> Result<(), GuiError> {
        Err(GuiError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_options_build_default_command() {
        let options = GuiLaunchOptions::new("examples/hello.toml");

        assert_eq!(
            options.embedded_command(),
            vec!["neotui", "run", "examples/hello.toml"]
        );
        assert_eq!(options.window_title, "NeoTUI - hello.toml");
    }

    #[test]
    fn launch_options_support_custom_program_and_args() {
        let options = GuiLaunchOptions::new("examples/dashboard.toml")
            .with_cli_program("cargo")
            .with_extra_cli_args(["--debug"]);

        assert_eq!(
            options.embedded_command(),
            vec!["cargo", "run", "examples/dashboard.toml", "--debug"]
        );
    }

    #[test]
    fn launch_options_support_custom_working_directory() {
        let options = GuiLaunchOptions::new("examples/hello.toml")
            .with_working_directory("crates/neotui-cli")
            .with_extra_cli_args(["--gui"]);
        let plan = GuiLaunchPlan::from_options(&options);
        let expected_workdir = path_to_string(
            &std::env::current_dir()
                .expect("current dir should be available")
                .join("crates/neotui-cli"),
        );

        assert_eq!(
            plan.working_directory.as_deref(),
            Some(expected_workdir.as_str())
        );
        assert_eq!(plan.command.last().map(String::as_str), Some("--gui"));
    }

    #[test]
    fn availability_reports_missing_platform_or_session() {
        let unsupported = availability_from_flags(false, false);
        let missing_session = availability_from_flags(true, false);
        let ready = availability_from_flags(true, true);

        assert_eq!(unsupported.reason, "linux-gtk-vte-only");
        assert!(!unsupported.ready());
        assert_eq!(missing_session.reason, "missing-display-session");
        assert!(!missing_session.ready());
        assert_eq!(ready.reason, "ready-for-embed-loop");
        assert!(ready.ready());
    }

    #[test]
    fn launch_plan_anchors_relative_app_paths_to_current_workspace() {
        let options = GuiLaunchOptions::new("examples/hello.toml");
        let plan = GuiLaunchPlan::from_options(&options);
        let current_dir = std::env::current_dir().expect("current dir should be available");
        let expected_path = path_to_string(&current_dir.join("examples/hello.toml"));
        let expected_workdir = path_to_string(&current_dir.join("examples"));

        assert_eq!(
            plan.command,
            vec!["neotui".to_string(), "run".to_string(), expected_path]
        );
        assert_eq!(
            plan.working_directory.as_deref(),
            Some(expected_workdir.as_str())
        );
        assert_eq!(plan.application_id, "dev.neotui.gui");
        assert_eq!(plan.window_title, "NeoTUI - hello.toml");
    }

    #[test]
    fn launch_plan_preserves_custom_command_and_window_defaults() {
        let options = GuiLaunchOptions::new("examples/dashboard.toml")
            .with_cli_program("cargo")
            .with_window_title("NeoTUI Dashboard")
            .with_working_directory(".")
            .with_extra_cli_args(["--release"]);
        let plan = GuiLaunchPlan::from_options(&options);
        let expected_workdir =
            path_to_string(&std::env::current_dir().expect("current dir should be available"));

        assert_eq!(plan.command[0], "cargo");
        assert_eq!(plan.command[1], "run");
        assert_eq!(plan.command.last().map(String::as_str), Some("--release"));
        assert_eq!(
            plan.working_directory.as_deref(),
            Some(expected_workdir.as_str())
        );
        assert_eq!(plan.window_title, "NeoTUI Dashboard");
        assert_eq!(plan.default_width, 1200);
        assert_eq!(plan.default_height, 800);
    }

    #[test]
    fn prepare_launch_requires_supported_environment() {
        let result = prepare_gui_launch("examples/hello.toml");

        if cfg!(target_os = "linux") && graphical_session_present() {
            assert!(result.is_ok());
        } else if cfg!(target_os = "linux") {
            assert_eq!(
                result.expect_err("missing session should fail"),
                GuiError::MissingDisplaySession
            );
        } else {
            assert_eq!(
                result.expect_err("non-linux should fail"),
                GuiError::UnsupportedPlatform
            );
        }
    }

    #[test]
    fn launch_requires_supported_environment_before_runtime_bootstrap() {
        if cfg!(target_os = "linux") && graphical_session_present() {
            return;
        }

        let options = GuiLaunchOptions::new("examples/hello.toml");
        let result = launch_embedded_terminal(&options);

        if cfg!(target_os = "linux") {
            assert_eq!(
                result.expect_err("missing session should fail"),
                GuiError::MissingDisplaySession
            );
        } else {
            assert_eq!(
                result.expect_err("non-linux should fail"),
                GuiError::UnsupportedPlatform
            );
        }
    }
}
