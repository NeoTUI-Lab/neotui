// Stack layout widgets
// Provide minimal vertical and horizontal containers for declarative layouts

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::{split_horizontal, split_vertical, Constraint, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    id: ComponentId,
    direction: StackDirection,
}

impl Stack {
    pub fn vertical(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Vertical,
        }
    }

    pub fn horizontal(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Horizontal,
        }
    }

    pub fn direction(&self) -> StackDirection {
        self.direction
    }
}

impl Component for Stack {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn child_layout_areas(&self, area: &Rect, child_count: usize) -> Vec<Rect> {
        if child_count == 0 {
            return Vec::new();
        }

        let constraints = vec![Constraint::Flex(1); child_count];

        match self.direction {
            StackDirection::Vertical => split_vertical(area.clone(), &constraints),
            StackDirection::Horizontal => split_horizontal(area.clone(), &constraints),
        }
    }

    fn render(&self, _ctx: &RenderContext, _frame: &mut Frame) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;

    #[test]
    fn vbox_splits_children_vertically() {
        let stack = Stack::vertical("layout");

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 8, 5), 2);

        assert_eq!(areas, vec![Rect::new(0, 0, 8, 2), Rect::new(0, 2, 8, 3)]);
    }

    #[test]
    fn hbox_splits_children_horizontally() {
        let stack = Stack::horizontal("layout");

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 7, 3), 3);

        assert_eq!(
            areas,
            vec![
                Rect::new(0, 0, 2, 3),
                Rect::new(2, 0, 2, 3),
                Rect::new(4, 0, 3, 3),
            ]
        );
    }

    #[test]
    fn stack_layout_uses_component_id_and_area() {
        let stack = Stack::horizontal("row");

        let node = stack.layout(&LayoutContext, Rect::new(1, 2, 6, 4));

        assert_eq!(node.component_id, ComponentId("row".into()));
        assert_eq!(node.area, Rect::new(1, 2, 6, 4));
    }
}
