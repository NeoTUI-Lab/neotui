use std::ffi::OsString;
use std::sync::Once;

use tracing_subscriber::EnvFilter;

pub const DEFAULT_TRACING_FILTER: &str = concat!(
    "neotui::cli=debug,",
    "neotui::gui=debug,",
    "neotui::dsl=debug,",
    "neotui::registry=debug,",
    "neotui::runtime=debug,",
    "neotui::terminal=debug"
);

static INIT_TRACING: Once = Once::new();

pub fn debug_mode_enabled() -> bool {
    debug_mode_from_flag(std::env::var_os("NEOTUI_DEBUG").as_ref())
}

pub fn debug_mode_from_flag(flag: Option<&OsString>) -> bool {
    let Some(flag) = flag else {
        return false;
    };
    let lower = flag.to_string_lossy().to_ascii_lowercase();

    matches!(lower.as_str(), "1" | "true" | "yes" | "on" | "debug")
}

pub fn init_tracing() {
    if !debug_mode_enabled() {
        return;
    }

    INIT_TRACING.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(DEFAULT_TRACING_FILTER))
            .with_target(true)
            .with_ansi(false)
            .without_time()
            .try_init();

        tracing::debug!(
            target: "neotui::runtime",
            tracing_filter = DEFAULT_TRACING_FILTER,
            "initialized NeoTUI tracing"
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_mode_flag_recognizes_truthy_values() {
        assert!(debug_mode_from_flag(Some(&OsString::from("1"))));
        assert!(debug_mode_from_flag(Some(&OsString::from("true"))));
        assert!(debug_mode_from_flag(Some(&OsString::from("DEBUG"))));
        assert!(!debug_mode_from_flag(Some(&OsString::from("0"))));
        assert!(!debug_mode_from_flag(Some(&OsString::from("false"))));
        assert!(!debug_mode_from_flag(None));
    }
}
