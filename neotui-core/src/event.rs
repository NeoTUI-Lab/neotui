#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(char),
    Ctrl(char),
    Special(SpecialKey),
    Mouse(i32, i32),
    Resize(u16, u16),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialKey {
    Enter,
    Escape,
    Tab,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event() {
        let e = Event::Key('a');
        assert_eq!(e, Event::Key('a'));
    }

    #[test]
    fn test_ctrl_event() {
        let e = Event::Ctrl('c');
        assert_eq!(e, Event::Ctrl('c'));
    }

    #[test]
    fn test_special_key_event() {
        let e = Event::Special(SpecialKey::Escape);
        assert_eq!(e, Event::Special(SpecialKey::Escape));
    }

    #[test]
    fn test_mouse_event() {
        let e = Event::Mouse(10, 5);
        assert_eq!(e, Event::Mouse(10, 5));
    }

    #[test]
    fn test_resize_event() {
        let e = Event::Resize(120, 40);
        assert_eq!(e, Event::Resize(120, 40));
    }

    #[test]
    fn test_tick_event() {
        let e = Event::Tick;
        assert_eq!(e, Event::Tick);
    }
}
