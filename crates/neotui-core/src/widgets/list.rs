use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::{
    ComponentId, Event, EventContext, EventResult, KeyCode, MouseButton, MouseEventKind,
    ScrollDirection,
};
use crate::layout::Rect;
use crate::render::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    id: ComponentId,
    items: Vec<String>,
    title: Option<String>,
    style: Style,
    selected_style: Style,
    focused: bool,
    selected_index: usize,
    scroll_offset: usize,
}

impl List {
    pub fn new<I, S>(id: impl Into<String>, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: ComponentId(id.into()),
            items: items.into_iter().map(Into::into).collect(),
            title: None,
            style: Style::default(),
            selected_style: Style {
                bold: true,
                ..Style::default()
            },
            focused: false,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    fn visible_rows(&self, area: &Rect) -> usize {
        let reserved = usize::from(self.title.is_some());
        usize::from(area.height).saturating_sub(reserved)
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        } else {
            self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));
            self.scroll_offset = self.scroll_offset.min(self.selected_index);
        }
    }

    fn sync_scroll_with_selection(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else {
            let bottom = self.scroll_offset.saturating_add(visible_rows);
            if self.selected_index >= bottom {
                self.scroll_offset = self
                    .selected_index
                    .saturating_add(1)
                    .saturating_sub(visible_rows);
            }
        }
    }
}

impl Component for List {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        let area = ctx.area();
        if area.is_empty() {
            return;
        }

        let mut row = area.y;
        if let Some(title) = &self.title {
            let title = title
                .chars()
                .take(usize::from(area.width))
                .collect::<String>();
            let _ = frame.draw_text(area.x, row, &title, self.style.clone());
            row = row.saturating_add(1);
        }

        let visible_rows = self.visible_rows(area);
        for (visible_index, item) in self
            .items
            .iter()
            .skip(self.scroll_offset)
            .take(visible_rows)
            .enumerate()
        {
            let absolute_index = self.scroll_offset.saturating_add(visible_index);
            let is_selected = absolute_index == self.selected_index;
            let style = if is_selected {
                self.selected_style.clone()
            } else {
                self.style.clone()
            };
            let prefix = if is_selected && self.focused {
                '>'
            } else {
                ' '
            };
            let line = format!("{prefix} {item}");
            let clipped = line
                .chars()
                .take(usize::from(area.width))
                .collect::<String>();
            let y = row.saturating_add(u16::try_from(visible_index).unwrap_or(0));
            let _ = frame.draw_text(area.x, y, &clipped, style);
        }
    }

    fn on_event(&mut self, _ctx: &mut EventContext, event: &Event) -> EventResult {
        self.clamp_selection();
        match event {
            Event::FocusGained(id) if *id == self.id => {
                self.focused = true;
                EventResult::RequestRender
            }
            Event::FocusLost(id) if *id == self.id => {
                self.focused = false;
                EventResult::RequestRender
            }
            Event::Key(key) if self.focused => {
                let previous = self.selected_index;
                match key.code {
                    KeyCode::Up => {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if !self.items.is_empty() {
                            self.selected_index =
                                (self.selected_index + 1).min(self.items.len().saturating_sub(1));
                        }
                    }
                    KeyCode::Home => self.selected_index = 0,
                    KeyCode::End => {
                        if !self.items.is_empty() {
                            self.selected_index = self.items.len().saturating_sub(1);
                        }
                    }
                    KeyCode::PageUp => {
                        self.selected_index = self.selected_index.saturating_sub(5);
                    }
                    KeyCode::PageDown => {
                        if !self.items.is_empty() {
                            self.selected_index =
                                (self.selected_index + 5).min(self.items.len().saturating_sub(1));
                        }
                    }
                    _ => return EventResult::Ignored,
                }
                if previous != self.selected_index {
                    self.sync_scroll_with_selection(5);
                    EventResult::RequestRender
                } else {
                    EventResult::Consumed
                }
            }
            Event::Scroll(scroll) => {
                let previous = self.selected_index;
                match scroll.direction {
                    ScrollDirection::Up => {
                        self.selected_index = self
                            .selected_index
                            .saturating_sub(usize::from(scroll.amount));
                    }
                    ScrollDirection::Down => {
                        if !self.items.is_empty() {
                            self.selected_index = self
                                .selected_index
                                .saturating_add(usize::from(scroll.amount))
                                .min(self.items.len().saturating_sub(1));
                        }
                    }
                }
                if previous != self.selected_index {
                    self.sync_scroll_with_selection(5);
                    EventResult::RequestRender
                } else {
                    EventResult::Consumed
                }
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                self.focused = true;
                EventResult::RequestRender
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::event::{Event, KeyEvent, KeyModifiers, ScrollEvent};
    use crate::render::{Color, ScreenBuffer};

    #[test]
    fn list_renders_title_and_items() {
        let list = List::new("services", ["api", "jobs"]).with_title("Services");
        let ctx = RenderContext::new(Rect::new(0, 0, 12, 3));
        let mut frame = ScreenBuffer::new(12, 3);

        list.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('S'));
        assert_eq!(frame.get(0, 1).map(|cell| cell.symbol), Some(' '));
        assert_eq!(frame.get(2, 1).map(|cell| cell.symbol), Some('a'));
    }

    #[test]
    fn list_moves_selection_on_keys() {
        let mut list = List::new("services", ["api", "jobs", "cache"]);
        let mut ctx = EventContext::default();
        let _ = list.on_event(
            &mut ctx,
            &Event::FocusGained(ComponentId("services".into())),
        );

        assert_eq!(
            list.on_event(
                &mut ctx,
                &Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::default(),
                })
            ),
            EventResult::RequestRender
        );

        let render_ctx = RenderContext::new(Rect::new(0, 0, 12, 3));
        let mut frame = ScreenBuffer::new(12, 3);
        list.render(&render_ctx, &mut frame);
        assert_eq!(frame.get(0, 1).map(|cell| cell.symbol), Some(' '));
        assert_eq!(frame.get(0, 2).map(|cell| cell.symbol), Some('>'));
    }

    #[test]
    fn list_scroll_event_advances_selection() {
        let mut list = List::new("services", ["api", "jobs", "cache"]);

        assert_eq!(
            list.on_event(
                &mut EventContext::default(),
                &Event::Scroll(ScrollEvent {
                    direction: ScrollDirection::Down,
                    amount: 1,
                })
            ),
            EventResult::RequestRender
        );
    }

    #[test]
    fn list_applies_selected_style() {
        let style = Style {
            fg: Color::Cyan,
            bold: true,
            ..Style::default()
        };
        let list = List::new("services", ["api"]).with_selected_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 12, 1));
        let mut frame = ScreenBuffer::new(12, 1);

        list.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }
}
