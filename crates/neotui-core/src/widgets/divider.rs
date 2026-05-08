// Divider widget
// Renders a horizontal or vertical separator line into the framebuffer

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Cell, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divider {
    id: ComponentId,
    orientation: DividerOrientation,
    symbol: char,
    style: Style,
}

impl Divider {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            orientation: DividerOrientation::Horizontal,
            symbol: '-',
            style: Style::default(),
        }
    }

    pub fn orientation(&self) -> DividerOrientation {
        self.orientation
    }

    pub fn symbol(&self) -> char {
        self.symbol
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn with_orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_symbol(mut self, symbol: char) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for Divider {
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

        match self.orientation {
            DividerOrientation::Horizontal => {
                let y = area.y.saturating_add(area.height.saturating_sub(1) / 2);

                for offset in 0..area.width {
                    let x = area.x.saturating_add(offset);
                    let _ = frame.set(
                        x,
                        y,
                        Cell {
                            symbol: self.symbol,
                            style: self.style.clone(),
                        },
                    );
                }
            }
            DividerOrientation::Vertical => {
                let x = area.x.saturating_add(area.width.saturating_sub(1) / 2);

                for offset in 0..area.height {
                    let y = area.y.saturating_add(offset);
                    let _ = frame.set(
                        x,
                        y,
                        Cell {
                            symbol: self.symbol,
                            style: self.style.clone(),
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};

    #[test]
    fn divider_renders_horizontal_line() {
        let divider = Divider::new("sep");
        let ctx = RenderContext::new(Rect::new(1, 0, 4, 3));
        let mut frame = ScreenBuffer::new(6, 3);

        divider.render(&ctx, &mut frame);

        assert_eq!(frame.get(1, 1).map(|cell| cell.symbol), Some('-'));
        assert_eq!(frame.get(4, 1).map(|cell| cell.symbol), Some('-'));
    }

    #[test]
    fn divider_renders_vertical_line() {
        let divider = Divider::new("sep")
            .with_orientation(DividerOrientation::Vertical)
            .with_symbol('|');
        let ctx = RenderContext::new(Rect::new(0, 1, 3, 4));
        let mut frame = ScreenBuffer::new(4, 6);

        divider.render(&ctx, &mut frame);

        assert_eq!(frame.get(1, 1).map(|cell| cell.symbol), Some('|'));
        assert_eq!(frame.get(1, 4).map(|cell| cell.symbol), Some('|'));
    }

    #[test]
    fn divider_applies_style_to_rendered_cells() {
        let style = Style {
            fg: Color::Magenta,
            bold: true,
            ..Style::default()
        };
        let divider = Divider::new("sep").with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 3, 1));
        let mut frame = ScreenBuffer::new(3, 1);

        divider.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }

    #[test]
    fn divider_layout_uses_component_id_and_area() {
        let divider = Divider::new("sep");
        let area = Rect::new(0, 2, 8, 1);

        let node = divider.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("sep".into()));
        assert_eq!(node.area, area);
    }
}
