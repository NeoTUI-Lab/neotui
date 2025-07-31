use crate::component::{Component, EventResult};
use crate::event::Event;

pub struct VBox {
    pub children: Vec<Box<dyn Component>>,
    pub focused_index: usize,
}

impl VBox {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            focused_index: 0,
        }
    }

    pub fn children_len(&self) -> usize {
        self.children.len()
    }

    pub fn with_children(children: Vec<Box<dyn Component>>) -> Self {
        let mut vbox = Self {
            focused_index: 0,
            children,
        };

        vbox.focus_first();
        vbox
    }

    pub fn focus_first(&mut self) {
        for (i, child) in self.children.iter_mut().enumerate() {
            if child.is_focusable() {
                child.focus();
                self.focused_index = i;
                break;
            }
        }
    }

    pub fn next_focus(&mut self) {
        let len = self.children.len();
        for i in 1..=len {
            let idx = (self.focused_index + i) % len;
            if self.children[idx].is_focusable() {
                self.children[self.focused_index].blur();
                self.children[idx].focus();
                self.focused_index = idx;
                break;
            }
        }
    }

    pub fn prev_focus(&mut self) {
        let len = self.children.len();
        for i in 1..=len {
            let idx = (self.focused_index + len - i) % len;
            if self.children[idx].is_focusable() {
                self.children[self.focused_index].blur();
                self.children[idx].focus();
                self.focused_index = idx;
                break;
            }
        }
    }

    pub fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }
}

impl Component for VBox {
    fn render(&self, buffer: &mut crate::renderer::ScreenBuffer, area: crate::layout::Rect) {
        let regions = area.split_vertically(self.children.len());
    
        for (child, region) in self.children.iter().zip(regions.iter()) {
            child.render(buffer, *region);
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        for child in self.children.iter_mut() {
            let result = child.on_event(event.clone());
            if let EventResult::Consumed = result {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Label;
    use crate::event::Event;

    #[test]
    fn test_vbox_render() {
        let label1 = Box::new(Label::new("One"));
        let label2 = Box::new(Label::new("Two"));

        let vbox = VBox::with_children(vec![label1, label2]);
        vbox.render(); // Simplesmente imprime por enquanto
    }

    #[test]
    fn test_vbox_event_propagation() {
        struct Echo;
        impl Component for Echo {
            fn render(&self) {}
            fn on_event(&mut self, event: Event) -> EventResult {
                if let Event::Key('x') = event {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
        }

        let echo = Box::new(Echo);
        let mut vbox = VBox::with_children(vec![echo]);

        let result = vbox.on_event(Event::Key('x'));
        assert!(matches!(result, EventResult::Consumed));

        let result = vbox.on_event(Event::Key('z'));
        assert!(matches!(result, EventResult::Ignored));
    }
}
