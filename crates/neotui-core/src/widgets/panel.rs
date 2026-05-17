// Panel widget
// Renders a bordered container and exposes its inner content area

use crate::component::{Component, ComponentNode, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::layout::{split_vertical, Constraint};
use crate::render::{panel_content_rect, BorderStyle, Padding, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    id: ComponentId,
    title: Option<String>,
    style: Style,
    padding: Padding,
    border: BorderStyle,
}

impl Panel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            style: Style::default(),
            padding: Padding::default(),
            border: BorderStyle::default(),
        }
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn padding(&self) -> Padding {
        self.padding
    }

    pub fn border(&self) -> BorderStyle {
        self.border
    }

    pub fn content_area(&self, area: Rect) -> Rect {
        panel_content_rect(area, self.padding)
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }
}

impl Component for Panel {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn child_layout_areas(&self, area: &Rect, children: &[ComponentNode]) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }

        split_vertical(
            self.content_area(area.clone()),
            &vec![Constraint::Flex(1); children.len()],
        )
    }

    fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        frame.draw_panel(
            ctx.area().clone(),
            self.title(),
            self.style.clone(),
            self.padding,
            self.border,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};

    #[test]
    fn panel_renders_border_and_title() {
        let panel = Panel::new("stats").with_title("Stats");
        let ctx = RenderContext::new(Rect::new(0, 0, 12, 5));
        let mut frame = ScreenBuffer::new(12, 5);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('+'));
        assert_eq!(frame.get(11, 4).map(|cell| cell.symbol), Some('+'));
        assert_eq!(frame.get(2, 0).map(|cell| cell.symbol), Some('S'));
        assert_eq!(frame.get(6, 0).map(|cell| cell.symbol), Some('s'));
    }

    #[test]
    fn panel_content_area_respects_padding() {
        let panel = Panel::new("stats").with_padding(Padding::uniform(1));

        let content = panel.content_area(Rect::new(0, 0, 10, 6));

        assert_eq!(content, Rect::new(2, 2, 6, 2));
    }

    #[test]
    fn panel_applies_style_to_border_cells() {
        let style = Style {
            fg: Color::Cyan,
            bold: true,
            ..Style::default()
        };
        let panel = Panel::new("stats").with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }

    #[test]
    fn panel_supports_custom_border_style() {
        let border = BorderStyle {
            top_left: '#',
            top_right: '#',
            bottom_left: '#',
            bottom_right: '#',
            horizontal: '=',
            vertical: '!',
        };
        let panel = Panel::new("stats").with_border(border);
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('#'));
        assert_eq!(frame.get(1, 0).map(|cell| cell.symbol), Some('='));
        assert_eq!(frame.get(0, 1).map(|cell| cell.symbol), Some('!'));
    }

    #[test]
    fn panel_layout_uses_component_id_and_area() {
        let panel = Panel::new("container");
        let area = Rect::new(1, 2, 10, 4);

        let node = panel.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("container".into()));
        assert_eq!(node.area, area);
    }

    #[test]
    fn panel_distributes_children_inside_content_area() {
        let panel = Panel::new("container");

        let children = vec![
            ComponentNode::new(Box::new(Panel::new("a"))),
            ComponentNode::new(Box::new(Panel::new("b"))),
            ComponentNode::new(Box::new(Panel::new("c"))),
        ];
        let areas = panel.child_layout_areas(&Rect::new(0, 0, 12, 6), &children);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(1, 1, 10, 1));
        assert_eq!(areas[1], Rect::new(1, 2, 10, 1));
        assert_eq!(areas[2], Rect::new(1, 3, 10, 2));
    }
}
