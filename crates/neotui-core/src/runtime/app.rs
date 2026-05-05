// Minimal runtime event loop
// Owns event polling and shortcut normalization for terminal apps

use crate::event::{Command, Event, EventResult, KeyCode};
use crate::runtime::event_adapter;
use crate::runtime::panic;
use crate::runtime::terminal::{TerminalLifecycle, TerminalSession};

use crossterm::event;
use std::io;
use std::time::Duration;

/// Abstraction over event polling so the runtime can be unit tested.
pub trait EventSource {
    fn next_event(&mut self, tick_rate: Option<Duration>) -> io::Result<Event>;
}

/// Default event source backed by crossterm terminal input.
#[derive(Debug, Default)]
pub struct RuntimeEventSource;

impl EventSource for RuntimeEventSource {
    fn next_event(&mut self, tick_rate: Option<Duration>) -> io::Result<Event> {
        loop {
            if let Some(timeout) = tick_rate {
                if !event::poll(timeout)? {
                    return Ok(Event::Tick);
                }
            }

            if let Some(event) = event_adapter::from_crossterm(event::read()?) {
                return Ok(event);
            }
        }
    }
}

/// Minimal runtime loop for terminal-first applications.
#[derive(Debug)]
pub struct AppRuntime<E = RuntimeEventSource> {
    event_source: E,
    tick_rate: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIteration {
    pub event: Event,
    pub result: EventResult,
    pub should_render: bool,
    pub should_quit: bool,
}

impl Default for AppRuntime<RuntimeEventSource> {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeIteration {
    fn new(event: Event, result: EventResult) -> Self {
        let should_quit_from_event = matches!(event, Event::QuitRequested);
        let should_quit_from_command = result.command().is_some_and(Command::requests_quit);

        Self {
            event,
            should_render: result.requests_render() || event.requests_render(),
            should_quit: should_quit_from_event || should_quit_from_command,
            result,
        }
    }
}

impl AppRuntime<RuntimeEventSource> {
    pub fn new() -> Self {
        Self {
            event_source: RuntimeEventSource,
            tick_rate: None,
        }
    }
}

impl<E> AppRuntime<E>
where
    E: EventSource,
{
    pub fn with_event_source(event_source: E) -> Self {
        Self {
            event_source,
            tick_rate: None,
        }
    }

    pub fn with_tick_rate(mut self, tick_rate: Duration) -> Self {
        self.tick_rate = Some(tick_rate);
        self
    }

    pub fn tick_rate(&self) -> Option<Duration> {
        self.tick_rate
    }

    pub fn run<F>(&mut self, mut on_event: F) -> io::Result<()>
    where
        F: FnMut(Event) -> EventResult,
    {
        loop {
            let iteration = self.run_once(&mut on_event)?;

            if iteration.should_quit {
                return Ok(());
            }
        }
    }

    pub fn run_once<F>(&mut self, mut on_event: F) -> io::Result<RuntimeIteration>
    where
        F: FnMut(Event) -> EventResult,
    {
        let event = normalize_runtime_event(self.event_source.next_event(self.tick_rate)?);
        let result = on_event(event.clone());

        Ok(RuntimeIteration::new(event, result))
    }

    pub fn run_with_terminal<F, T>(&mut self, terminal: &mut T, on_event: F) -> io::Result<()>
    where
        F: FnMut(Event) -> EventResult,
        T: TerminalLifecycle,
    {
        panic::install_panic_hook();
        terminal.enter()?;

        let run_result = self.run(on_event);
        let exit_result = terminal.exit();

        match (run_result, exit_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), _) => Err(err),
            (Ok(()), Err(err)) => Err(err),
        }
    }
}

impl AppRuntime<RuntimeEventSource> {
    pub fn run_terminal<F>(&mut self, on_event: F) -> io::Result<()>
    where
        F: FnMut(Event) -> EventResult,
    {
        let mut terminal = TerminalSession::new();
        self.run_with_terminal(&mut terminal, on_event)
    }
}

fn normalize_runtime_event(event: Event) -> Event {
    if let Some(scroll) = event_adapter::extract_scroll(&event) {
        return Event::Scroll(scroll);
    }

    if is_quit_shortcut(&event) {
        return Event::QuitRequested;
    }

    if is_help_shortcut(&event) {
        return Event::HelpRequested;
    }

    event
}

fn is_quit_shortcut(event: &Event) -> bool {
    match event {
        Event::Key(key) => {
            key.modifiers.ctrl && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
        }
        _ => false,
    }
}

fn is_help_shortcut(event: &Event) -> bool {
    matches!(event, Event::Key(key) if key.code == KeyCode::F(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{
        KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, ScrollDirection, ScrollEvent,
    };
    use std::collections::VecDeque;

    #[derive(Debug)]
    struct MockEventSource {
        events: VecDeque<Event>,
        last_tick_rate: Option<Duration>,
    }

    impl MockEventSource {
        fn new(events: Vec<Event>) -> Self {
            Self {
                events: events.into(),
                last_tick_rate: None,
            }
        }
    }

    impl EventSource for MockEventSource {
        fn next_event(&mut self, tick_rate: Option<Duration>) -> io::Result<Event> {
            self.last_tick_rate = tick_rate;
            self.events
                .pop_front()
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "no more events"))
        }
    }

    #[derive(Debug, Default)]
    struct MockTerminal {
        entered: bool,
        exited: bool,
        fail_on_enter: bool,
        fail_on_exit: bool,
    }

    impl TerminalLifecycle for MockTerminal {
        fn enter(&mut self) -> io::Result<()> {
            self.entered = true;

            if self.fail_on_enter {
                return Err(io::Error::other("enter failed"));
            }

            Ok(())
        }

        fn exit(&mut self) -> io::Result<()> {
            self.exited = true;

            if self.fail_on_exit {
                return Err(io::Error::other("exit failed"));
            }

            Ok(())
        }

        fn is_active(&self) -> bool {
            self.entered && !self.exited
        }
    }

    #[test]
    fn runtime_converts_ctrl_q_to_quit_requested() {
        let mut runtime =
            AppRuntime::with_event_source(MockEventSource::new(vec![Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers {
                    ctrl: true,
                    ..Default::default()
                },
            })]));

        let mut seen = Vec::new();
        runtime
            .run(|event| {
                seen.push(event);
                EventResult::Consumed
            })
            .expect("runtime should stop on ctrl+q");

        assert_eq!(seen, vec![Event::QuitRequested]);
    }

    #[test]
    fn runtime_promotes_mouse_scroll_to_scroll_event() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 3,
                row: 5,
            }),
            Event::QuitRequested,
        ]));

        let mut seen = Vec::new();
        runtime
            .run(|event| {
                seen.push(event);
                EventResult::Consumed
            })
            .expect("runtime should process queued events");

        assert_eq!(
            seen,
            vec![
                Event::Scroll(ScrollEvent {
                    direction: ScrollDirection::Down,
                    amount: 1,
                }),
                Event::QuitRequested,
            ]
        );
    }

    #[test]
    fn runtime_passes_configured_tick_rate_to_event_source() {
        let tick_rate = Duration::from_millis(16);
        let event_source = MockEventSource::new(vec![Event::QuitRequested]);
        let mut runtime = AppRuntime::with_event_source(event_source).with_tick_rate(tick_rate);

        runtime
            .run(|_| EventResult::Consumed)
            .expect("runtime should stop on quit event");

        assert_eq!(runtime.tick_rate(), Some(tick_rate));
        assert_eq!(runtime.event_source.last_tick_rate, Some(tick_rate));
    }

    #[test]
    fn runtime_maps_f1_to_help_requested() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![
            Event::Key(KeyEvent {
                code: KeyCode::F(1),
                modifiers: KeyModifiers::default(),
            }),
            Event::QuitRequested,
        ]));

        let mut seen = Vec::new();
        runtime
            .run(|event| {
                seen.push(event);
                EventResult::Consumed
            })
            .expect("runtime should process help and quit events");

        assert_eq!(seen[0], Event::HelpRequested);
    }

    #[test]
    fn runtime_enters_and_exits_terminal_session() {
        let mut runtime =
            AppRuntime::with_event_source(MockEventSource::new(vec![Event::QuitRequested]));
        let mut terminal = MockTerminal::default();

        runtime
            .run_with_terminal(&mut terminal, |_| EventResult::Consumed)
            .expect("runtime should manage terminal lifecycle");

        assert!(terminal.entered);
        assert!(terminal.exited);
        assert!(!terminal.is_active());
    }

    #[test]
    fn runtime_exits_terminal_even_when_event_loop_fails() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(Vec::new()));
        let mut terminal = MockTerminal::default();

        let result = runtime.run_with_terminal(&mut terminal, |_| EventResult::Consumed);

        assert!(result.is_err());
        assert!(terminal.entered);
        assert!(terminal.exited);
    }

    #[test]
    fn runtime_stops_when_handler_requests_quit_command() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![Event::Tick]));
        let mut handled = 0;

        runtime
            .run(|_| {
                handled += 1;
                EventResult::Command(Command::Quit)
            })
            .expect("runtime should stop on quit command");

        assert_eq!(handled, 1);
    }

    #[test]
    fn runtime_keeps_running_for_non_quit_commands() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![
            Event::Tick,
            Event::QuitRequested,
        ]));
        let mut seen = Vec::new();

        runtime
            .run(|event| {
                seen.push(event);
                if seen.len() == 1 {
                    EventResult::Command(Command::Help)
                } else {
                    EventResult::Consumed
                }
            })
            .expect("runtime should ignore non-quit commands");

        assert_eq!(seen, vec![Event::Tick, Event::QuitRequested]);
    }

    #[test]
    fn run_once_reports_render_request() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![Event::Tick]));

        let iteration = runtime
            .run_once(|_| EventResult::RequestRender)
            .expect("runtime should process a single event");

        assert_eq!(iteration.event, Event::Tick);
        assert_eq!(iteration.result, EventResult::RequestRender);
        assert!(iteration.should_render);
        assert!(!iteration.should_quit);
    }

    #[test]
    fn run_once_reports_quit_requested_event() {
        let mut runtime =
            AppRuntime::with_event_source(MockEventSource::new(vec![Event::QuitRequested]));

        let iteration = runtime
            .run_once(|_| EventResult::Consumed)
            .expect("runtime should process quit event");

        assert_eq!(iteration.event, Event::QuitRequested);
        assert!(iteration.should_quit);
        assert!(!iteration.should_render);
    }

    #[test]
    fn run_once_marks_help_command_for_render() {
        let mut runtime = AppRuntime::with_event_source(MockEventSource::new(vec![Event::Tick]));

        let iteration = runtime
            .run_once(|_| EventResult::Command(Command::Help))
            .expect("runtime should process help command");

        assert_eq!(iteration.result, EventResult::Command(Command::Help));
        assert!(iteration.should_render);
        assert!(!iteration.should_quit);
    }

    #[test]
    fn runtime_iteration_marks_quit_command() {
        let iteration = RuntimeIteration::new(Event::Tick, EventResult::Command(Command::Quit));

        assert!(iteration.should_quit);
        assert!(!iteration.should_render);
    }

    #[test]
    fn runtime_iteration_marks_resize_for_render_even_if_ignored() {
        let iteration = RuntimeIteration::new(
            Event::Resize {
                width: 100,
                height: 30,
            },
            EventResult::Ignored,
        );

        assert!(iteration.should_render);
        assert!(!iteration.should_quit);
    }
}
