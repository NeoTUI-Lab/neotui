use crate::component::{Component, EventContext, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::{ComponentId, Event, EventResult, KeyCode, MouseButton, MouseEventKind};
use crate::layout::Rect;
use crate::render::{Style, TextAlign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    id: ComponentId,
    text: String,
    variant: Option<String>,
    style: Style,
    focused_style: Style,
    focused: bool,
    pressed: bool,
    on_click: Option<String>,
}

impl Button {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        let style = Style::default();
        let focused_style = Style {
            bold: true,
            ..Style::default()
        };

        Self {
            id: ComponentId(id.into()),
            text: text.into(),
            variant: None,
            style,
            focused_style,
            focused: false,
            pressed: false,
            on_click: None,
        }
    }

    pub fn with_on_click(mut self, action_id: impl Into<String>) -> Self {
        self.on_click = Some(action_id.into());
        self
    }

    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
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

    fn label(&self) -> String {
        let left = if self.pressed { '<' } else { '[' };
        let right = if self.pressed { '>' } else { ']' };
        format!("{left} {} {right}", self.text)
    }

    fn activate(&mut self, ctx: &mut EventContext) -> EventResult {
        self.pressed = !self.pressed;
        if let Some(action_id) = &self.on_click {
            ctx.push_action(action_id.clone());
            EventResult::Command(crate::event::Command::Action(action_id.clone()))
        } else {
            EventResult::RequestRender
        }
    }
}

impl Component for Button {
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

        let y = area.y.saturating_add(area.height.saturating_sub(1) / 2);
        let style = if self.focused {
            self.focused_style.clone()
        } else {
            self.style.clone()
        };

        let label = self.label();
        let _ = frame.draw_text_aligned(area.x, y, area.width, &label, style, TextAlign::Center);
    }

    fn on_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::FocusGained(id) if *id == self.id => {
                self.focused = true;
                EventResult::RequestRender
            }
            Event::FocusLost(id) if *id == self.id => {
                self.focused = false;
                self.pressed = false;
                EventResult::RequestRender
            }
            Event::Key(key) if self.focused && matches!(key.code, KeyCode::Enter) => {
                self.activate(ctx)
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                self.activate(ctx)
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::event::{Event, KeyEvent, KeyModifiers};
    use crate::render::{Color, ScreenBuffer};
    use crate::testing::snapshot_buffer;

    #[test]
    fn button_renders_centered_label() {
        let button = Button::new("deploy", "Deploy");
        let ctx = RenderContext::new(Rect::new(0, 0, 14, 1));
        let mut frame = ScreenBuffer::new(14, 1);

        button.render(&ctx, &mut frame);

        assert_eq!(frame.get(2, 0).map(|cell| cell.symbol), Some('['));
        assert_eq!(frame.get(11, 0).map(|cell| cell.symbol), Some(']'));
    }

    #[test]
    fn button_focus_and_activation_request_render() {
        let mut button = Button::new("deploy", "Deploy");
        let mut ctx = EventContext::default();

        assert_eq!(
            button.on_event(&mut ctx, &Event::FocusGained(ComponentId("deploy".into()))),
            EventResult::RequestRender
        );
        assert_eq!(
            button.on_event(
                &mut ctx,
                &Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::default(),
                })
            ),
            EventResult::RequestRender
        );
    }

    #[test]
    fn button_activation_emits_declared_action() {
        let mut button = Button::new("refresh", "Refresh").with_on_click("refresh_now");
        let mut ctx = EventContext::default();
        let _ = button.on_event(&mut ctx, &Event::FocusGained(ComponentId("refresh".into())));

        let result = button.on_event(
            &mut ctx,
            &Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::default(),
            }),
        );

        assert_eq!(
            result,
            EventResult::Command(crate::event::Command::Action("refresh_now".into()))
        );
        assert_eq!(
            ctx.commands,
            vec![crate::event::Command::Action("refresh_now".into())]
        );
    }

    #[test]
    fn button_applies_focused_style() {
        let style = Style {
            fg: Color::Yellow,
            bold: true,
            ..Style::default()
        };
        let mut button = Button::new("deploy", "Deploy").with_focused_style(style.clone());
        let _ = button.on_event(
            &mut EventContext::default(),
            &Event::FocusGained(ComponentId("deploy".into())),
        );
        let ctx = RenderContext::new(Rect::new(0, 0, 14, 1));
        let mut frame = ScreenBuffer::new(14, 1);

        button.render(&ctx, &mut frame);

        assert_eq!(frame.get(2, 0).map(|cell| cell.style.clone()), Some(style));
    }

    #[test]
    fn button_snapshot_stays_stable() {
        let button = Button::new("deploy", "Deploy");
        let ctx = RenderContext::new(Rect::new(0, 0, 14, 1));
        let mut frame = ScreenBuffer::new(14, 1);

        button.render(&ctx, &mut frame);

        assert_eq!(snapshot_buffer(&frame), "··[·Deploy·]··");
    }
}
