// Terminal runtime module
// Manages terminal session lifecycle

pub mod app;
pub mod event_adapter;
pub mod panic;
pub mod terminal;

pub use app::{AppRuntime, EventSource, GlobalShortcuts, RuntimeEventSource, RuntimeIteration};
pub use terminal::{TerminalLifecycle, TerminalSession};
