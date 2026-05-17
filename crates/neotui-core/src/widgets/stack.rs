// Stack layout widgets
// Provide minimal vertical and horizontal containers for declarative layouts

use crate::component::{Component, ComponentNode, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::{split_horizontal, split_vertical, Axis, Constraint, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    id: ComponentId,
    direction: StackDirection,
    gap: u16,
}

impl Stack {
    pub fn vertical(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Vertical,
            gap: 0,
        }
    }

    pub fn horizontal(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Horizontal,
            gap: 0,
        }
    }

    pub fn direction(&self) -> StackDirection {
        self.direction
    }

    pub fn gap(&self) -> u16 {
        self.gap
    }

    pub fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }
}

impl Component for Stack {
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

        let total_gap = self
            .gap
            .saturating_mul(u16::try_from(children.len().saturating_sub(1)).unwrap_or(u16::MAX));

        let available_area = match self.direction {
            StackDirection::Vertical => Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(total_gap),
            ),
            StackDirection::Horizontal => Rect::new(
                area.x,
                area.y,
                area.width.saturating_sub(total_gap),
                area.height,
            ),
        };

        let axis = match self.direction {
            StackDirection::Vertical => Axis::Vertical,
            StackDirection::Horizontal => Axis::Horizontal,
        };
        let constraints = children
            .iter()
            .map(|child| {
                child
                    .layout_hints()
                    .constraint_for_axis(axis)
                    .unwrap_or(Constraint::Flex(1))
            })
            .collect::<Vec<_>>();

        let base_areas = match self.direction {
            StackDirection::Vertical => split_vertical(available_area, &constraints),
            StackDirection::Horizontal => split_horizontal(available_area, &constraints),
        };

        base_areas
            .into_iter()
            .enumerate()
            .map(|(index, rect)| match self.direction {
                StackDirection::Vertical => Rect::new(
                    rect.x,
                    rect.y.saturating_add(
                        self.gap
                            .saturating_mul(u16::try_from(index).unwrap_or(u16::MAX)),
                    ),
                    rect.width,
                    rect.height,
                ),
                StackDirection::Horizontal => Rect::new(
                    rect.x.saturating_add(
                        self.gap
                            .saturating_mul(u16::try_from(index).unwrap_or(u16::MAX)),
                    ),
                    rect.y,
                    rect.width,
                    rect.height,
                ),
            })
            .collect()
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

        let children = vec![
            ComponentNode::new(Box::new(Stack::vertical("a"))),
            ComponentNode::new(Box::new(Stack::vertical("b"))),
        ];
        let areas = stack.child_layout_areas(&Rect::new(0, 0, 8, 5), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 8, 2), Rect::new(0, 2, 8, 3)]);
    }

    #[test]
    fn hbox_splits_children_horizontally() {
        let stack = Stack::horizontal("layout");

        let children = vec![
            ComponentNode::new(Box::new(Stack::horizontal("a"))),
            ComponentNode::new(Box::new(Stack::horizontal("b"))),
            ComponentNode::new(Box::new(Stack::horizontal("c"))),
        ];
        let areas = stack.child_layout_areas(&Rect::new(0, 0, 7, 3), &children);

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

    #[test]
    fn vbox_applies_gap_between_children() {
        let stack = Stack::vertical("layout").with_gap(1);

        let children = vec![
            ComponentNode::new(Box::new(Stack::vertical("a"))),
            ComponentNode::new(Box::new(Stack::vertical("b"))),
        ];
        let areas = stack.child_layout_areas(&Rect::new(0, 0, 8, 6), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 8, 2), Rect::new(0, 3, 8, 3)]);
    }

    #[test]
    fn hbox_applies_gap_between_children() {
        let stack = Stack::horizontal("layout").with_gap(2);

        let children = vec![
            ComponentNode::new(Box::new(Stack::horizontal("a"))),
            ComponentNode::new(Box::new(Stack::horizontal("b"))),
        ];
        let areas = stack.child_layout_areas(&Rect::new(0, 0, 10, 3), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 4, 3), Rect::new(6, 0, 4, 3)]);
    }

    #[test]
    fn stack_uses_fixed_and_flex_constraints_from_children() {
        let stack = Stack::horizontal("layout").with_gap(1);
        let children = vec![
            ComponentNode::new(Box::new(Stack::horizontal("fixed"))).with_layout_hints(
                crate::component::LayoutHints {
                    width: Some(3),
                    ..crate::component::LayoutHints::default()
                },
            ),
            ComponentNode::new(Box::new(Stack::horizontal("flex"))).with_layout_hints(
                crate::component::LayoutHints {
                    grow: Some(1),
                    ..crate::component::LayoutHints::default()
                },
            ),
        ];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 10, 2), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 3, 2), Rect::new(4, 0, 6, 2)]);
    }

    #[test]
    fn stack_uses_percentage_constraint_from_children() {
        let stack = Stack::vertical("layout");
        let children = vec![
            ComponentNode::new(Box::new(Stack::vertical("top"))).with_layout_hints(
                crate::component::LayoutHints {
                    height_pct: Some(25),
                    ..crate::component::LayoutHints::default()
                },
            ),
            ComponentNode::new(Box::new(Stack::vertical("bottom"))),
        ];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 8, 8), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 8, 2), Rect::new(0, 2, 8, 6)]);
    }
}
