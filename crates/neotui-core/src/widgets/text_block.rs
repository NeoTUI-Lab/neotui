use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    id: ComponentId,
    text: String,
    style: Style,
}

impl TextBlock {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            text: text.into(),
            style: Style::default(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for TextBlock {
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

        for (row_offset, line) in self.text.lines().enumerate() {
            let Ok(row_offset) = u16::try_from(row_offset) else {
                break;
            };
            if row_offset >= area.height {
                break;
            }

            let y = area.y.saturating_add(row_offset);
            let clipped = line
                .chars()
                .take(usize::from(area.width))
                .collect::<String>();
            let _ = frame.draw_text(area.x, y, &clipped, self.style.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};

    #[test]
    fn text_block_renders_multiple_lines() {
        let block = TextBlock::new("body", "alpha\nbeta");
        let ctx = RenderContext::new(Rect::new(0, 0, 8, 2));
        let mut frame = ScreenBuffer::new(8, 2);

        block.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('a'));
        assert_eq!(frame.get(0, 1).map(|cell| cell.symbol), Some('b'));
    }

    #[test]
    fn text_block_clips_line_width_and_height() {
        let block = TextBlock::new("body", "abcdef\nsecond\nthird");
        let ctx = RenderContext::new(Rect::new(1, 0, 3, 2));
        let mut frame = ScreenBuffer::new(6, 3);

        block.render(&ctx, &mut frame);

        assert_eq!(frame.get(1, 0).map(|cell| cell.symbol), Some('a'));
        assert_eq!(frame.get(3, 0).map(|cell| cell.symbol), Some('c'));
        assert_eq!(frame.get(1, 1).map(|cell| cell.symbol), Some('s'));
        assert_eq!(frame.get(1, 2).map(|cell| cell.symbol), Some(' '));
    }

    #[test]
    fn text_block_applies_style() {
        let style = Style {
            fg: Color::Magenta,
            ..Style::default()
        };
        let block = TextBlock::new("body", "hi").with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 2, 1));
        let mut frame = ScreenBuffer::new(2, 1);

        block.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }
}
