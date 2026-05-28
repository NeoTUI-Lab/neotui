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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackAlign {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackJustify {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    id: ComponentId,
    direction: StackDirection,
    gap: u16,
    align: StackAlign,
    justify: StackJustify,
}

impl Stack {
    pub fn vertical(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Vertical,
            gap: 0,
            align: StackAlign::Stretch,
            justify: StackJustify::Start,
        }
    }

    pub fn horizontal(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            direction: StackDirection::Horizontal,
            gap: 0,
            align: StackAlign::Stretch,
            justify: StackJustify::Start,
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

    pub fn align(&self) -> StackAlign {
        self.align
    }

    pub fn with_align(mut self, align: StackAlign) -> Self {
        self.align = align;
        self
    }

    pub fn justify(&self) -> StackJustify {
        self.justify
    }

    pub fn with_justify(mut self, justify: StackJustify) -> Self {
        self.justify = justify;
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

        let main_axis = axis;
        let cross_axis = match axis {
            Axis::Vertical => Axis::Horizontal,
            Axis::Horizontal => Axis::Vertical,
        };
        let total_main_used = base_areas.iter().fold(total_gap, |used, rect| {
            used.saturating_add(match main_axis {
                Axis::Vertical => rect.height,
                Axis::Horizontal => rect.width,
            })
        });
        let available_main = match main_axis {
            Axis::Vertical => area.height,
            Axis::Horizontal => area.width,
        };
        let justify_offset = match self.justify {
            StackJustify::Start => 0,
            StackJustify::Center => available_main.saturating_sub(total_main_used) / 2,
            StackJustify::End => available_main.saturating_sub(total_main_used),
        };

        base_areas
            .into_iter()
            .enumerate()
            .map(|(index, rect)| {
                let rect = match self.direction {
                    StackDirection::Vertical => Rect::new(
                        rect.x,
                        rect.y
                            .saturating_add(
                                self.gap
                                    .saturating_mul(u16::try_from(index).unwrap_or(u16::MAX)),
                            )
                            .saturating_add(justify_offset),
                        rect.width,
                        rect.height,
                    ),
                    StackDirection::Horizontal => Rect::new(
                        rect.x
                            .saturating_add(
                                self.gap
                                    .saturating_mul(u16::try_from(index).unwrap_or(u16::MAX)),
                            )
                            .saturating_add(justify_offset),
                        rect.y,
                        rect.width,
                        rect.height,
                    ),
                };

                align_rect_for_child(
                    rect,
                    area,
                    cross_axis,
                    self.align,
                    children[index]
                        .layout_hints()
                        .constraint_for_axis(cross_axis),
                )
            })
            .collect()
    }

    fn render(&self, _ctx: &RenderContext, _frame: &mut Frame) {}
}

impl Default for StackAlign {
    fn default() -> Self {
        Self::Stretch
    }
}

impl Default for StackJustify {
    fn default() -> Self {
        Self::Start
    }
}

fn align_rect_for_child(
    rect: Rect,
    parent: &Rect,
    cross_axis: Axis,
    align: StackAlign,
    cross_constraint: Option<Constraint>,
) -> Rect {
    let requested_cross = match cross_constraint {
        Some(Constraint::Fixed(value)) => Some(value),
        Some(Constraint::Percentage(percent)) => Some(match cross_axis {
            Axis::Horizontal => {
                ((u32::from(parent.width) * u32::from(percent.min(100))) / 100) as u16
            }
            Axis::Vertical => {
                ((u32::from(parent.height) * u32::from(percent.min(100))) / 100) as u16
            }
        }),
        Some(Constraint::Flex(_)) | None => None,
    };

    match cross_axis {
        Axis::Horizontal => {
            let width = match align {
                StackAlign::Stretch => rect.width,
                _ => requested_cross.unwrap_or(rect.width).min(rect.width),
            };
            let x = match align {
                StackAlign::Start | StackAlign::Stretch => rect.x,
                StackAlign::Center => rect.x.saturating_add(rect.width.saturating_sub(width) / 2),
                StackAlign::End => rect.x.saturating_add(rect.width.saturating_sub(width)),
            };

            Rect::new(x, rect.y, width, rect.height)
        }
        Axis::Vertical => {
            let height = match align {
                StackAlign::Stretch => rect.height,
                _ => requested_cross.unwrap_or(rect.height).min(rect.height),
            };
            let y = match align {
                StackAlign::Start | StackAlign::Stretch => rect.y,
                StackAlign::Center => rect
                    .y
                    .saturating_add(rect.height.saturating_sub(height) / 2),
                StackAlign::End => rect.y.saturating_add(rect.height.saturating_sub(height)),
            };

            Rect::new(rect.x, y, rect.width, height)
        }
    }
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

    #[test]
    fn stack_justifies_children_at_center() {
        let stack = Stack::horizontal("layout").with_justify(StackJustify::Center);
        let children = vec![
            ComponentNode::new(Box::new(Stack::horizontal("left"))).with_layout_hints(
                crate::component::LayoutHints {
                    width: Some(2),
                    ..crate::component::LayoutHints::default()
                },
            ),
            ComponentNode::new(Box::new(Stack::horizontal("right"))).with_layout_hints(
                crate::component::LayoutHints {
                    width: Some(2),
                    ..crate::component::LayoutHints::default()
                },
            ),
        ];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 10, 2), &children);

        assert_eq!(areas, vec![Rect::new(3, 0, 2, 2), Rect::new(5, 0, 2, 2)]);
    }

    #[test]
    fn stack_aligns_children_on_cross_axis() {
        let stack = Stack::vertical("layout").with_align(StackAlign::Center);
        let children = vec![ComponentNode::new(Box::new(Stack::vertical("child")))
            .with_layout_hints(crate::component::LayoutHints {
                width: Some(4),
                ..crate::component::LayoutHints::default()
            })];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 10, 4), &children);

        assert_eq!(areas, vec![Rect::new(3, 0, 4, 4)]);
    }

    #[test]
    fn stack_justifies_children_at_end() {
        let stack = Stack::vertical("layout")
            .with_gap(1)
            .with_justify(StackJustify::End);
        let children = vec![
            ComponentNode::new(Box::new(Stack::vertical("top"))).with_layout_hints(
                crate::component::LayoutHints {
                    height: Some(1),
                    ..crate::component::LayoutHints::default()
                },
            ),
            ComponentNode::new(Box::new(Stack::vertical("bottom"))).with_layout_hints(
                crate::component::LayoutHints {
                    height: Some(2),
                    ..crate::component::LayoutHints::default()
                },
            ),
        ];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 6, 8), &children);

        assert_eq!(areas, vec![Rect::new(0, 4, 6, 1), Rect::new(0, 6, 6, 2)]);
    }

    #[test]
    fn stack_stretches_cross_axis_when_requested() {
        let stack = Stack::horizontal("layout").with_align(StackAlign::Stretch);
        let children = vec![ComponentNode::new(Box::new(Stack::horizontal("child")))
            .with_layout_hints(crate::component::LayoutHints {
                width: Some(3),
                height: Some(1),
                ..crate::component::LayoutHints::default()
            })];

        let areas = stack.child_layout_areas(&Rect::new(0, 0, 8, 4), &children);

        assert_eq!(areas, vec![Rect::new(0, 0, 3, 4)]);
    }

    #[test]
    fn stack_combines_nested_constraints_and_alignment() {
        let root = Stack::vertical("root")
            .with_gap(1)
            .with_align(StackAlign::Center)
            .with_justify(StackJustify::Center);
        let children = vec![
            ComponentNode::new(Box::new(Stack::horizontal("header"))).with_layout_hints(
                crate::component::LayoutHints {
                    width: Some(6),
                    height: Some(1),
                    ..crate::component::LayoutHints::default()
                },
            ),
            ComponentNode::new(Box::new(Stack::horizontal("body"))).with_layout_hints(
                crate::component::LayoutHints {
                    width: Some(10),
                    height: Some(2),
                    ..crate::component::LayoutHints::default()
                },
            ),
        ];

        let areas = root.child_layout_areas(&Rect::new(0, 0, 16, 8), &children);

        assert_eq!(areas, vec![Rect::new(5, 2, 6, 1), Rect::new(3, 4, 10, 2)]);
    }
}
