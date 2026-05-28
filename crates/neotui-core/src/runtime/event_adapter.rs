// Event adapter: converts crossterm events to NeoTUI events
// Bridges crossterm types with normalized NeoTUI event model

use crate::event::{
    ComponentId, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    ScrollDirection, ScrollEvent,
};

use crossterm::event as crossterm_event;

/// Convert crossterm event to NeoTUI event
pub fn from_crossterm(event: crossterm_event::Event) -> Option<Event> {
    match event {
        crossterm_event::Event::Key(key) => Some(Event::Key(convert_key(key))),
        crossterm_event::Event::Mouse(mouse) => Some(Event::Mouse(convert_mouse(mouse))),
        crossterm_event::Event::Resize(w, h) => Some(Event::Resize {
            width: w,
            height: h,
        }),
        crossterm_event::Event::FocusGained => {
            Some(Event::FocusGained(ComponentId("root".to_string())))
        }
        crossterm_event::Event::FocusLost => {
            Some(Event::FocusLost(ComponentId("root".to_string())))
        }
        crossterm_event::Event::Paste(_) => None,
    }
}

fn convert_key(key: crossterm_event::KeyEvent) -> KeyEvent {
    KeyEvent {
        code: convert_key_code(key.code),
        modifiers: convert_modifiers(key.modifiers),
    }
}

fn convert_key_code(code: crossterm_event::KeyCode) -> KeyCode {
    match code {
        crossterm_event::KeyCode::Char(c) => KeyCode::Char(c),
        crossterm_event::KeyCode::Enter => KeyCode::Enter,
        crossterm_event::KeyCode::Esc => KeyCode::Escape,
        crossterm_event::KeyCode::Tab => KeyCode::Tab,
        crossterm_event::KeyCode::Backspace => KeyCode::Backspace,
        crossterm_event::KeyCode::Delete => KeyCode::Delete,
        crossterm_event::KeyCode::Left => KeyCode::Left,
        crossterm_event::KeyCode::Right => KeyCode::Right,
        crossterm_event::KeyCode::Up => KeyCode::Up,
        crossterm_event::KeyCode::Down => KeyCode::Down,
        crossterm_event::KeyCode::Home => KeyCode::Home,
        crossterm_event::KeyCode::End => KeyCode::End,
        crossterm_event::KeyCode::PageUp => KeyCode::PageUp,
        crossterm_event::KeyCode::PageDown => KeyCode::PageDown,
        crossterm_event::KeyCode::F(n) => KeyCode::F(n),
        _ => KeyCode::Char('?'),
    }
}

fn convert_modifiers(modifiers: crossterm_event::KeyModifiers) -> KeyModifiers {
    KeyModifiers {
        shift: modifiers.contains(crossterm_event::KeyModifiers::SHIFT),
        ctrl: modifiers.contains(crossterm_event::KeyModifiers::CONTROL),
        alt: modifiers.contains(crossterm_event::KeyModifiers::ALT),
    }
}

fn convert_mouse(mouse: crossterm_event::MouseEvent) -> MouseEvent {
    MouseEvent {
        kind: convert_mouse_kind(mouse.kind),
        column: mouse.column,
        row: mouse.row,
    }
}

fn convert_mouse_kind(kind: crossterm_event::MouseEventKind) -> MouseEventKind {
    match kind {
        crossterm_event::MouseEventKind::Down(button) => {
            MouseEventKind::Down(convert_mouse_button(button))
        }
        crossterm_event::MouseEventKind::Up(button) => {
            MouseEventKind::Up(convert_mouse_button(button))
        }
        crossterm_event::MouseEventKind::Drag(button) => {
            MouseEventKind::Drag(convert_mouse_button(button))
        }
        crossterm_event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
        crossterm_event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
        crossterm_event::MouseEventKind::Moved
        | crossterm_event::MouseEventKind::ScrollLeft
        | crossterm_event::MouseEventKind::ScrollRight => MouseEventKind::Moved,
    }
}

fn convert_mouse_button(button: crossterm_event::MouseButton) -> MouseButton {
    match button {
        crossterm_event::MouseButton::Left => MouseButton::Left,
        crossterm_event::MouseButton::Right => MouseButton::Right,
        crossterm_event::MouseButton::Middle => MouseButton::Middle,
    }
}

/// Convert scroll events from mouse events
pub fn extract_scroll(event: &Event) -> Option<ScrollEvent> {
    if let Event::Mouse(mouse) = event {
        let direction = match mouse.kind {
            MouseEventKind::ScrollUp => Some(ScrollDirection::Up),
            MouseEventKind::ScrollDown => Some(ScrollDirection::Down),
            _ => None,
        };
        direction.map(|dir| ScrollEvent {
            direction: dir,
            amount: 1,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_crossterm_resize() {
        let crossterm_event = crossterm_event::Event::Resize(80, 24);
        let neo_event = from_crossterm(crossterm_event);
        assert!(matches!(
            neo_event,
            Some(Event::Resize {
                width: 80,
                height: 24
            })
        ));
    }

    #[test]
    fn test_convert_key_modifiers() {
        let crossterm_key = crossterm_event::KeyEvent {
            code: crossterm_event::KeyCode::Char('c'),
            modifiers: crossterm_event::KeyModifiers::CONTROL,
            kind: crossterm_event::KeyEventKind::Press,
            state: crossterm_event::KeyEventState::empty(),
        };
        let neo_key = convert_key(crossterm_key);
        assert_eq!(neo_key.code, KeyCode::Char('c'));
        assert!(neo_key.modifiers.ctrl);
        assert!(!neo_key.modifiers.shift);
    }
}
