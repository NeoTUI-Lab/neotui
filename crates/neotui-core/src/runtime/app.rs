// Minimal runtime event loop
// Owns event polling and shortcut normalization for terminal apps

use crate::event::{Event, EventResult, KeyCode};
use crate::runtime::event_adapter;

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

impl Default for AppRuntime<RuntimeEventSource> {
    fn default() -> Self {
        Self::new()
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
            let event = normalize_runtime_event(self.event_source.next_event(self.tick_rate)?);
            let should_quit = matches!(event, Event::QuitRequested);

            let _ = on_event(event);

            if should_quit {
                return Ok(());
            }
        }
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
}
