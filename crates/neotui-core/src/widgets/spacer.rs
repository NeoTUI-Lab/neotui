// Spacer widget
// Occupies layout space without producing visible output

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spacer {
    id: ComponentId,
}

impl Spacer {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
        }
    }
}

impl Component for Spacer {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn render(&self, _ctx: &RenderContext, _frame: &mut Frame) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::ScreenBuffer;

    #[test]
    fn spacer_layout_uses_component_id_and_area() {
        let spacer = Spacer::new("gap");
        let area = Rect::new(2, 3, 4, 1);

        let node = spacer.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("gap".into()));
        assert_eq!(node.area, area);
    }

    #[test]
    fn spacer_renders_no_visible_output() {
        let spacer = Spacer::new("gap");
        let ctx = RenderContext::new(Rect::new(0, 0, 4, 2));
        let mut frame = ScreenBuffer::new(4, 2);

        spacer.render(&ctx, &mut frame);

        assert!(frame.cells().iter().all(|cell| cell.symbol == ' '));
    }
}
