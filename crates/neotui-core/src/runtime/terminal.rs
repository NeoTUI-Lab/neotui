// Terminal session management
// Handles raw mode, alternate screen, and safe teardown

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tracing::debug;

/// Lifecycle contract for terminal setup and teardown.
pub trait TerminalLifecycle {
    fn enter(&mut self) -> io::Result<()>;
    fn exit(&mut self) -> io::Result<()>;
    fn is_active(&self) -> bool;
}

/// Represents an active terminal session
pub struct TerminalSession {
    is_active: bool,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new() -> Self {
        Self { is_active: false }
    }

    /// Enter raw mode and alternate screen
    pub fn enter(&mut self) -> io::Result<()> {
        if self.is_active {
            debug!(target: "neotui::terminal", "terminal session already active");
            return Ok(());
        }

        debug!(target: "neotui::terminal", "entering terminal session");
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        self.is_active = true;
        debug!(target: "neotui::terminal", "terminal session entered");
        Ok(())
    }

    /// Leave alternate screen and restore terminal
    pub fn exit(&mut self) -> io::Result<()> {
        if !self.is_active {
            debug!(target: "neotui::terminal", "terminal session already inactive");
            return Ok(());
        }

        debug!(target: "neotui::terminal", "restoring terminal session");
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        disable_raw_mode()?;

        self.is_active = false;
        debug!(target: "neotui::terminal", "terminal session restored");
        Ok(())
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl TerminalLifecycle for TerminalSession {
    fn enter(&mut self) -> io::Result<()> {
        TerminalSession::enter(self)
    }

    fn exit(&mut self) -> io::Result<()> {
        TerminalSession::exit(self)
    }

    fn is_active(&self) -> bool {
        TerminalSession::is_active(self)
    }
}

impl Default for TerminalSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        // Ensure terminal is restored on drop
        let _ = self.exit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_session_creation() {
        let session = TerminalSession::new();
        assert!(!session.is_active());
    }

    #[test]
    fn test_terminal_session_default() {
        let session = TerminalSession::default();
        assert!(!session.is_active());
    }
}
