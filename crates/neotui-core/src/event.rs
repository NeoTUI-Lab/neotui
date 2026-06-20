// Event model
// Core abstractions for event handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn matches_shortcut(&self, shortcut: &KeyShortcut) -> bool {
        self.code == shortcut.code && self.modifiers == shortcut.modifiers
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyShortcut {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyShortcut {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn ctrl(code: char) -> Self {
        Self {
            code: KeyCode::Char(code),
            modifiers: KeyModifiers {
                ctrl: true,
                ..Default::default()
            },
        }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::default(),
        }
    }

    pub fn matches(&self, event: &Event) -> bool {
        matches!(event, Event::Key(key) if key.matches_shortcut(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollEvent {
    pub direction: ScrollDirection,
    pub amount: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Scroll(ScrollEvent),
    Resize { width: u16, height: u16 },
    FocusGained(ComponentId),
    FocusLost(ComponentId),
    Tick,
    QuitRequested,
    HelpRequested,
}

impl Event {
    pub fn requests_render(&self) -> bool {
        matches!(
            self,
            Event::Resize { .. }
                | Event::FocusGained(_)
                | Event::FocusLost(_)
                | Event::HelpRequested
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    Ignored,
    Consumed,
    RequestRender,
    Command(Command),
    Bubble(Command),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Quit,
    Help,
    Action(String),
    SetFormValue {
        form_id: String,
        field_id: String,
        value: String,
    },
}

impl EventResult {
    pub fn requests_render(&self) -> bool {
        matches!(
            self,
            EventResult::RequestRender
                | EventResult::Command(Command::Help | Command::SetFormValue { .. })
        )
    }

    pub fn command(&self) -> Option<&Command> {
        match self {
            EventResult::Command(command) => Some(command),
            _ => None,
        }
    }

    pub fn bubbled_command(&self) -> Option<&Command> {
        match self {
            EventResult::Bubble(command) => Some(command),
            _ => None,
        }
    }
}

impl Command {
    pub fn requests_quit(&self) -> bool {
        matches!(self, Command::Quit)
    }

    pub fn requests_render(&self) -> bool {
        matches!(self, Command::Help | Command::SetFormValue { .. })
    }

    pub fn action_id(&self) -> Option<&str> {
        match self {
            Command::Action(id) => Some(id),
            _ => None,
        }
    }

    pub fn form_value_update(&self) -> Option<(&str, &str, &str)> {
        match self {
            Command::SetFormValue {
                form_id,
                field_id,
                value,
            } => Some((form_id, field_id, value)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_creation() {
        let key = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers {
                ctrl: true,
                ..Default::default()
            },
        };
        assert_eq!(key.code, KeyCode::Char('x'));
        assert!(key.modifiers.ctrl);
    }

    #[test]
    fn test_event_variants() {
        let events = vec![
            Event::QuitRequested,
            Event::HelpRequested,
            Event::Tick,
            Event::Resize {
                width: 80,
                height: 24,
            },
        ];
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_event_result_variants() {
        let results = vec![
            EventResult::Ignored,
            EventResult::Consumed,
            EventResult::RequestRender,
            EventResult::Command(Command::Quit),
            EventResult::Bubble(Command::Help),
        ];
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_command_variants() {
        let commands = vec![
            Command::Quit,
            Command::Help,
            Command::Action("deploy".into()),
            Command::SetFormValue {
                form_id: "incident".into(),
                field_id: "summary".into(),
                value: "Disk full".into(),
            },
        ];
        assert_eq!(commands.len(), 4);
    }

    #[test]
    fn test_event_result_helpers() {
        assert!(EventResult::RequestRender.requests_render());
        assert!(EventResult::Command(Command::Help).requests_render());
        assert!(!EventResult::Consumed.requests_render());
        assert_eq!(
            EventResult::Command(Command::Quit).command(),
            Some(&Command::Quit)
        );
        assert_eq!(EventResult::Ignored.command(), None);
        assert_eq!(
            EventResult::Bubble(Command::Help).bubbled_command(),
            Some(&Command::Help)
        );
    }

    #[test]
    fn test_command_helpers() {
        assert!(Command::Quit.requests_quit());
        assert!(!Command::Help.requests_quit());
        assert!(Command::Help.requests_render());
        assert!(!Command::Quit.requests_render());
        assert_eq!(Command::Action("deploy".into()).action_id(), Some("deploy"));
        assert_eq!(
            Command::SetFormValue {
                form_id: "incident".into(),
                field_id: "summary".into(),
                value: "Disk full".into(),
            }
            .form_value_update(),
            Some(("incident", "summary", "Disk full"))
        );
    }

    #[test]
    fn test_event_render_helpers() {
        assert!(Event::Resize {
            width: 120,
            height: 40
        }
        .requests_render());
        assert!(Event::HelpRequested.requests_render());
        assert!(Event::FocusGained(ComponentId("root".into())).requests_render());
        assert!(!Event::Tick.requests_render());
        assert!(!Event::QuitRequested.requests_render());
    }

    #[test]
    fn test_key_shortcut_matches_key_event() {
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers {
                ctrl: true,
                ..Default::default()
            },
        });

        assert!(KeyShortcut::ctrl('q').matches(&event));
        assert!(!KeyShortcut::ctrl('w').matches(&event));
        assert!(!KeyShortcut::plain(KeyCode::Char('q')).matches(&event));
    }
}
