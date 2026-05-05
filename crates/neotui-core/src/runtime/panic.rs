// Panic hook for safe terminal restoration
// Ensures terminal is restored even on panic

use std::panic::{self, PanicInfo};
use std::sync::Once;

static INIT_PANIC_HOOK: Once = Once::new();

/// Install panic hook for safe terminal restoration
pub fn install_panic_hook() {
    INIT_PANIC_HOOK.call_once(|| {
        panic::set_hook(Box::new(panic_handler));
    });
}

/// Panic handler that restores terminal before aborting
fn panic_handler(info: &PanicInfo) {
    // Restore terminal to safe state
    let _ = restore_terminal();

    // Print panic information
    let location = info.location().unwrap_or_else(|| {
        // Fallback if location is not available
        panic::Location::caller()
    });

    eprintln!();
    eprintln!("=== NeoTUI Panic ===");
    eprintln!(
        "Location: {}:{}:{}",
        location.file(),
        location.line(),
        location.column()
    );

    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        eprintln!("Message: {}", msg);
    } else if let Some(msg) = info.payload().downcast_ref::<String>() {
        eprintln!("Message: {}", msg);
    } else {
        eprintln!("Message: <unknown>");
    }

    eprintln!("====================");
    eprintln!();
}

/// Restore terminal to safe state
fn restore_terminal() -> std::io::Result<()> {
    use crossterm::{
        event::DisableMouseCapture,
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_panic_hook() {
        // Installing multiple times should be safe due to Once
        install_panic_hook();
        install_panic_hook();
    }
}
