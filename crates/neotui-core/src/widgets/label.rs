// Label widget
// Renders a single line of aligned text into the framebuffer

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Style, TextAlign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    id: ComponentId,
    text: String,
    align: TextAlign,
    style: Style,
}

impl Label {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            text: text.into(),
            align: TextAlign::Left,
            style: Style::default(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn align(&self) -> TextAlign {
        self.align
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for Label {
    fn id(&self) -> ComponentId {
        self.id.clone()
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
        frame.draw_text_aligned(
            area.x,
            y,
            area.width,
            &self.text,
            self.style.clone(),
            self.align,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};
    use crate::testing::snapshot_buffer;

    #[test]
    fn label_renders_left_aligned_text() {
        let label = Label::new("title", "Hello NeoTUI");
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 1));
        let mut frame = ScreenBuffer::new(20, 1);

        label.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('H'));
        assert_eq!(frame.get(11, 0).map(|cell| cell.symbol), Some('I'));
    }

    #[test]
    fn label_renders_centered_text_inside_area() {
        let label = Label::new("title", "Neo").with_align(TextAlign::Center);
        let ctx = RenderContext::new(Rect::new(2, 0, 7, 1));
        let mut frame = ScreenBuffer::new(12, 1);

        label.render(&ctx, &mut frame);

        assert_eq!(frame.get(4, 0).map(|cell| cell.symbol), Some('N'));
        assert_eq!(frame.get(5, 0).map(|cell| cell.symbol), Some('e'));
        assert_eq!(frame.get(6, 0).map(|cell| cell.symbol), Some('o'));
    }

    #[test]
    fn label_renders_in_vertical_middle_row() {
        let label = Label::new("title", "Hi");
        let ctx = RenderContext::new(Rect::new(0, 1, 4, 3));
        let mut frame = ScreenBuffer::new(4, 5);

        label.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 2).map(|cell| cell.symbol), Some('H'));
        assert_eq!(frame.get(1, 2).map(|cell| cell.symbol), Some('i'));
    }

    #[test]
    fn label_applies_style_to_rendered_cells() {
        let style = Style {
            fg: Color::Green,
            bold: true,
            ..Style::default()
        };
        let label = Label::new("title", "Go").with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 4, 1));
        let mut frame = ScreenBuffer::new(4, 1);

        label.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }

    #[test]
    fn label_layout_uses_its_own_component_id() {
        let label = Label::new("greeting", "Hello");
        let area = Rect::new(1, 2, 10, 1);

        let node = label.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("greeting".into()));
        assert_eq!(node.area, area);
    }

    #[test]
    fn label_snapshot_stays_stable() {
        let label = Label::new("title", "Neo").with_align(TextAlign::Center);
        let ctx = RenderContext::new(Rect::new(0, 0, 9, 1));
        let mut frame = ScreenBuffer::new(9, 1);

        label.render(&ctx, &mut frame);

        assert_eq!(snapshot_buffer(&frame), "···Neo···");
    }
}
