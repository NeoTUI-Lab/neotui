use crate::component::{Component, EventContext, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::{ComponentId, Event, EventResult, KeyCode};
use crate::layout::Rect;
use crate::render::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInput {
    id: ComponentId,
    value: String,
    placeholder: Option<String>,
    form_id: String,
    field_id: String,
    style: Style,
    focused_style: Style,
    focused: bool,
    cursor: usize,
}

impl TextInput {
    pub fn new(
        id: impl Into<String>,
        value: impl Into<String>,
        form_id: impl Into<String>,
        field_id: impl Into<String>,
    ) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            id: ComponentId(id.into()),
            value,
            placeholder: None,
            form_id: form_id.into(),
            field_id: field_id.into(),
            style: Style::default(),
            focused_style: Style {
                bold: true,
                ..Style::default()
            },
            focused: false,
            cursor,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    fn emit_update(&self, ctx: &mut EventContext) -> EventResult {
        let command = crate::event::Command::SetFormValue {
            form_id: self.form_id.clone(),
            field_id: self.field_id.clone(),
            value: self.value.clone(),
        };
        ctx.push_command(command.clone());
        EventResult::Command(command)
    }

    fn insert_char(&mut self, ch: char) {
        let byte_index = char_to_byte_index(&self.value, self.cursor);
        self.value.insert(byte_index, ch);
        self.cursor = self.cursor.saturating_add(1);
    }

    fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let start = char_to_byte_index(&self.value, self.cursor.saturating_sub(1));
        let end = char_to_byte_index(&self.value, self.cursor);
        self.value.replace_range(start..end, "");
        self.cursor = self.cursor.saturating_sub(1);
        true
    }

    fn delete(&mut self) -> bool {
        let len = self.value.chars().count();
        if self.cursor >= len {
            return false;
        }
        let start = char_to_byte_index(&self.value, self.cursor);
        let end = char_to_byte_index(&self.value, self.cursor.saturating_add(1));
        self.value.replace_range(start..end, "");
        true
    }
}

impl Component for TextInput {
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

        let style = if self.focused {
            self.focused_style.clone()
        } else {
            self.style.clone()
        };
        let mut text = if self.value.is_empty() {
            self.placeholder.clone().unwrap_or_default()
        } else {
            self.value.clone()
        };
        if self.focused {
            let cursor = self.cursor.min(self.value.chars().count());
            let byte_index = char_to_byte_index(&text, cursor);
            text.insert(byte_index, '|');
        }
        let clipped = text
            .chars()
            .take(usize::from(area.width))
            .collect::<String>();
        let _ = frame.draw_text(area.x, area.y, &clipped, style);
    }

    fn on_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::FocusGained(id) if *id == self.id => {
                self.focused = true;
                self.cursor = self.value.chars().count();
                EventResult::RequestRender
            }
            Event::FocusLost(id) if *id == self.id => {
                self.focused = false;
                EventResult::RequestRender
            }
            Event::Key(key) if self.focused => match key.code {
                KeyCode::Char(ch) if !key.modifiers.ctrl && !key.modifiers.alt => {
                    self.insert_char(ch);
                    self.emit_update(ctx)
                }
                KeyCode::Backspace if self.backspace() => self.emit_update(ctx),
                KeyCode::Delete if self.delete() => self.emit_update(ctx),
                KeyCode::Left => {
                    self.cursor = self.cursor.saturating_sub(1);
                    EventResult::RequestRender
                }
                KeyCode::Right => {
                    self.cursor = (self.cursor + 1).min(self.value.chars().count());
                    EventResult::RequestRender
                }
                KeyCode::Home => {
                    self.cursor = 0;
                    EventResult::RequestRender
                }
                KeyCode::End => {
                    self.cursor = self.value.chars().count();
                    EventResult::RequestRender
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, KeyEvent, KeyModifiers};
    use crate::render::ScreenBuffer;

    #[test]
    fn text_input_renders_value_and_cursor_when_focused() {
        let mut input = TextInput::new("summary", "Disk", "incident", "summary");
        let _ = input.on_event(
            &mut EventContext::default(),
            &Event::FocusGained(ComponentId("summary".into())),
        );
        let ctx = RenderContext::new(Rect::new(0, 0, 8, 1));
        let mut frame = ScreenBuffer::new(8, 1);

        input.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('D'));
        assert_eq!(frame.get(4, 0).map(|cell| cell.symbol), Some('|'));
    }

    #[test]
    fn text_input_emits_form_update_on_text_edit() {
        let mut input = TextInput::new("summary", "Disk", "incident", "summary");
        let mut ctx = EventContext::default();
        let _ = input.on_event(&mut ctx, &Event::FocusGained(ComponentId("summary".into())));

        let result = input.on_event(
            &mut ctx,
            &Event::Key(KeyEvent {
                code: KeyCode::Char('!'),
                modifiers: KeyModifiers::default(),
            }),
        );

        assert_eq!(
            result,
            EventResult::Command(crate::event::Command::SetFormValue {
                form_id: "incident".into(),
                field_id: "summary".into(),
                value: "Disk!".into(),
            })
        );
        assert_eq!(ctx.commands.len(), 1);
    }

    #[test]
    fn text_input_supports_backspace_and_cursor_motion() {
        let mut input = TextInput::new("summary", "Disk", "incident", "summary");
        let mut ctx = EventContext::default();
        let _ = input.on_event(&mut ctx, &Event::FocusGained(ComponentId("summary".into())));
        let _ = input.on_event(
            &mut ctx,
            &Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::default(),
            }),
        );

        let result = input.on_event(
            &mut ctx,
            &Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::default(),
            }),
        );

        assert_eq!(
            result,
            EventResult::Command(crate::event::Command::SetFormValue {
                form_id: "incident".into(),
                field_id: "summary".into(),
                value: "Dik".into(),
            })
        );
    }
}
