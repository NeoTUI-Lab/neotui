// Event model
// Core abstractions for event handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    Ignored,
    Consumed,
    RequestRender,
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
        ];
        assert_eq!(results.len(), 3);
    }
}
