// Component model
// Shared contracts for declarative UI elements

use crate::event::{Command, ComponentId, Event, EventResult};
use crate::layout::{split_vertical, Axis, Constraint, Position, Rect};
use crate::render::ScreenBuffer;

/// Frame alias used by components when rendering into the framebuffer.
pub type Frame = ScreenBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutNode {
    pub component_id: ComponentId,
    pub area: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(component_id: ComponentId, area: Rect) -> Self {
        Self {
            component_id,
            area,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<LayoutNode>) -> Self {
        self.children = children;
        self
    }

    pub fn find_deepest_at(&self, position: Position) -> Option<&LayoutNode> {
        if !self.area.contains(position) {
            return None;
        }

        for child in self.children.iter().rev() {
            if let Some(hit) = child.find_deepest_at(position) {
                return Some(hit);
            }
        }

        Some(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayoutContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderContext {
    area: Rect,
}

impl RenderContext {
    pub fn new(area: Rect) -> Self {
        Self { area }
    }

    pub fn area(&self) -> &Rect {
        &self.area
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new(Rect::new(0, 0, 0, 0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventContext {
    pub commands: Vec<Command>,
}

impl EventContext {
    pub fn push_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn push_commands<I>(&mut self, commands: I)
    where
        I: IntoIterator<Item = Command>,
    {
        self.commands.extend(commands);
    }

    pub fn push_action(&mut self, action_id: impl Into<String>) {
        self.push_command(Command::Action(action_id.into()));
    }

    pub fn push_form_value(
        &mut self,
        form_id: impl Into<String>,
        field_id: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.push_command(Command::SetFormValue {
            form_id: form_id.into(),
            field_id: field_id.into(),
            value: value.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayoutHints {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub width_pct: Option<u16>,
    pub height_pct: Option<u16>,
    pub grow: Option<u16>,
}

impl LayoutHints {
    pub fn constraint_for_axis(&self, axis: Axis) -> Option<Constraint> {
        match axis {
            Axis::Vertical => {
                if let Some(height) = self.height {
                    Some(Constraint::Fixed(height))
                } else if let Some(height_pct) = self.height_pct {
                    Some(Constraint::Percentage(height_pct))
                } else {
                    self.grow.map(Constraint::Flex)
                }
            }
            Axis::Horizontal => {
                if let Some(width) = self.width {
                    Some(Constraint::Fixed(width))
                } else if let Some(width_pct) = self.width_pct {
                    Some(Constraint::Percentage(width_pct))
                } else {
                    self.grow.map(Constraint::Flex)
                }
            }
        }
    }
}

pub struct ComponentNode {
    component: Box<dyn Component>,
    layout_hints: LayoutHints,
    children: Vec<ComponentNode>,
}

impl ComponentNode {
    pub fn new(component: Box<dyn Component>) -> Self {
        Self {
            component,
            layout_hints: LayoutHints::default(),
            children: Vec::new(),
        }
    }

    pub fn with_layout_hints(mut self, layout_hints: LayoutHints) -> Self {
        self.layout_hints = layout_hints;
        self
    }

    pub fn with_children(mut self, children: Vec<ComponentNode>) -> Self {
        self.children = children;
        self
    }

    pub fn id(&self) -> ComponentId {
        self.component.id()
    }

    pub fn children(&self) -> &[ComponentNode] {
        &self.children
    }

    pub fn layout_hints(&self) -> &LayoutHints {
        &self.layout_hints
    }

    pub fn children_mut(&mut self) -> &mut [ComponentNode] {
        &mut self.children
    }

    pub fn push_child(&mut self, child: ComponentNode) {
        self.children.push(child);
    }

    pub fn find(&self, id: &ComponentId) -> Option<&ComponentNode> {
        if self.id() == *id {
            return Some(self);
        }

        for child in &self.children {
            if let Some(node) = child.find(id) {
                return Some(node);
            }
        }

        None
    }

    pub fn find_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentNode> {
        if self.id() == *id {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(node) = child.find_mut(id) {
                return Some(node);
            }
        }

        None
    }

    pub fn collect_ids_depth_first(&self, ids: &mut Vec<ComponentId>) {
        ids.push(self.id());

        for child in &self.children {
            child.collect_ids_depth_first(ids);
        }
    }

    pub fn max_depth(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(ComponentNode::max_depth)
            .max()
            .unwrap_or(0)
    }

    pub fn collect_focusable_ids_depth_first(&self, ids: &mut Vec<ComponentId>) {
        if self.component.is_focusable() {
            ids.push(self.id());
        }

        for child in &self.children {
            child.collect_focusable_ids_depth_first(ids);
        }
    }

    pub fn render_subtree(&self, ctx: &RenderContext, frame: &mut Frame) {
        self.component.render(ctx, frame);

        for child in &self.children {
            child.render_subtree(ctx, frame);
        }
    }

    pub fn layout_subtree(&self, ctx: &LayoutContext, area: Rect) -> LayoutNode {
        let child_areas = self.component.child_layout_areas(&area, &self.children);
        let children = self
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let child_area = child_areas
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| area.clone());
                child.layout_subtree(ctx, child_area)
            })
            .collect();

        self.component.layout(ctx, area).with_children(children)
    }

    pub fn render_layout_subtree(&self, layout: &LayoutNode, frame: &mut Frame) {
        self.component
            .render(&RenderContext::new(layout.area.clone()), frame);

        for (child, child_layout) in self.children.iter().zip(layout.children.iter()) {
            child.render_layout_subtree(child_layout, frame);
        }
    }

    pub fn dispatch_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult {
        for child in self.children.iter_mut().rev() {
            let result = child.dispatch_event(ctx, event);

            match result {
                EventResult::Ignored => {}
                EventResult::Bubble(_) => {}
                other => return other,
            }
        }

        self.component.on_event(ctx, event)
    }
}

pub struct ComponentTree {
    root: ComponentNode,
}

impl std::fmt::Debug for ComponentTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentTree")
            .field("component_count", &self.component_count())
            .field("max_depth", &self.max_depth())
            .finish()
    }
}

impl ComponentTree {
    pub fn new(root: ComponentNode) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &ComponentNode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut ComponentNode {
        &mut self.root
    }

    pub fn find(&self, id: &ComponentId) -> Option<&ComponentNode> {
        self.root.find(id)
    }

    pub fn find_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentNode> {
        self.root.find_mut(id)
    }

    pub fn ids_depth_first(&self) -> Vec<ComponentId> {
        let mut ids = Vec::new();
        self.root.collect_ids_depth_first(&mut ids);
        ids
    }

    pub fn focusable_ids_depth_first(&self) -> Vec<ComponentId> {
        let mut ids = Vec::new();
        self.root.collect_focusable_ids_depth_first(&mut ids);
        ids
    }

    pub fn component_count(&self) -> usize {
        self.ids_depth_first().len()
    }

    pub fn max_depth(&self) -> usize {
        self.root.max_depth()
    }

    pub fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        self.root.render_subtree(ctx, frame);
    }

    pub fn layout(&self, ctx: &LayoutContext, area: Rect) -> LayoutNode {
        self.root.layout_subtree(ctx, area)
    }

    pub fn render_with_layout(&self, layout: &LayoutNode, frame: &mut Frame) {
        self.root.render_layout_subtree(layout, frame);
    }

    pub fn dispatch_event(&mut self, ctx: &mut EventContext, event: &Event) -> EventResult {
        self.root.dispatch_event(ctx, event)
    }

    pub fn dispatch_event_to_target(
        &mut self,
        ctx: &mut EventContext,
        target_id: &ComponentId,
        event: &Event,
    ) -> EventResult {
        dispatch_event_to_target_node(&mut self.root, ctx, target_id, event)
            .unwrap_or(EventResult::Ignored)
    }

    pub fn dispatch_mouse_event(
        &mut self,
        ctx: &mut EventContext,
        layout: &LayoutNode,
        event: &Event,
    ) -> EventResult {
        let Event::Mouse(mouse) = event else {
            return EventResult::Ignored;
        };

        let position = Position::new(mouse.column, mouse.row);
        let Some(target) = layout.find_deepest_at(position) else {
            return EventResult::Ignored;
        };

        self.dispatch_event_to_target(ctx, &target.component_id, event)
    }

    pub fn resolve_scroll_target(
        &self,
        layout: &LayoutNode,
        focused: Option<&ComponentId>,
        event: &Event,
    ) -> Option<ComponentId> {
        if let Some(focused) = focused {
            return Some(focused.clone());
        }

        let Event::Mouse(mouse) = event else {
            return None;
        };

        layout
            .find_deepest_at(Position::new(mouse.column, mouse.row))
            .map(|node| node.component_id.clone())
    }
}

fn dispatch_event_to_target_node(
    node: &mut ComponentNode,
    ctx: &mut EventContext,
    target_id: &ComponentId,
    event: &Event,
) -> Option<EventResult> {
    if node.id() == *target_id {
        return Some(node.component.on_event(ctx, event));
    }

    for child in node.children.iter_mut().rev() {
        if let Some(result) = dispatch_event_to_target_node(child, ctx, target_id, event) {
            return Some(match result {
                EventResult::Bubble(_) | EventResult::Ignored => {
                    node.component.on_event(ctx, event)
                }
                other => other,
            });
        }
    }

    None
}

/// Common contract implemented by every NeoTUI component.
pub trait Component {
    fn id(&self) -> ComponentId;

    fn is_focusable(&self) -> bool {
        false
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn child_layout_areas(&self, area: &Rect, children: &[ComponentNode]) -> Vec<Rect> {
        if children.is_empty() {
            return Vec::new();
        }

        split_vertical(area.clone(), &vec![Constraint::Flex(1); children.len()])
    }

    fn render(&self, _ctx: &RenderContext, _frame: &mut Frame);

    fn on_event(&mut self, _ctx: &mut EventContext, _event: &Event) -> EventResult {
        EventResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{MouseButton, MouseEvent, MouseEventKind};
    use crate::render::{Cell, Style};

    #[derive(Debug, Clone)]
    struct StubComponent {
        id: ComponentId,
        event_result: EventResult,
        symbol: char,
        focusable: bool,
    }

    impl StubComponent {
        fn new(id: &str, event_result: EventResult) -> Self {
            Self {
                id: ComponentId(id.to_string()),
                event_result,
                symbol: 'N',
                focusable: false,
            }
        }
    }

    impl Component for StubComponent {
        fn id(&self) -> ComponentId {
            self.id.clone()
        }

        fn is_focusable(&self) -> bool {
            self.focusable
        }

        fn render(&self, _ctx: &RenderContext, frame: &mut Frame) {
            let _ = frame.set(
                0,
                0,
                Cell {
                    symbol: self.symbol,
                    style: Style::default(),
                },
            );
        }

        fn on_event(&mut self, ctx: &mut EventContext, _event: &Event) -> EventResult {
            if let EventResult::Command(command) = &self.event_result {
                ctx.push_command(command.clone());
            }

            self.event_result.clone()
        }
    }

    #[test]
    fn component_default_layout_uses_component_id_and_area() {
        let component = StubComponent::new("root", EventResult::Ignored);
        let area = Rect::new(1, 2, 10, 3);

        let node = component.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("root".into()));
        assert_eq!(node.area, area);
        assert!(node.children.is_empty());
    }

    #[test]
    fn component_can_render_into_framebuffer() {
        let component = StubComponent::new("root", EventResult::Ignored);
        let mut frame = Frame::new(2, 1);

        component.render(&RenderContext::default(), &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('N'));
    }

    #[test]
    fn event_context_collects_emitted_commands() {
        let mut component = StubComponent::new("root", EventResult::Command(Command::Help));
        let mut ctx = EventContext::default();

        let result = component.on_event(&mut ctx, &Event::HelpRequested);

        assert_eq!(result, EventResult::Command(Command::Help));
        assert_eq!(ctx.commands, vec![Command::Help]);
    }

    #[test]
    fn layout_node_builder_keeps_children() {
        let child = LayoutNode::new(ComponentId("child".into()), Rect::new(0, 0, 1, 1));
        let node = LayoutNode::new(ComponentId("root".into()), Rect::new(0, 0, 2, 2))
            .with_children(vec![child.clone()]);

        assert_eq!(node.children, vec![child]);
    }

    #[test]
    fn component_tree_finds_nodes_depth_first() {
        let tree = ComponentTree::new(
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![
                    ComponentNode::new(Box::new(StubComponent::new("left", EventResult::Ignored))),
                    ComponentNode::new(Box::new(StubComponent::new("right", EventResult::Ignored))),
                ]),
        );

        assert_eq!(
            tree.ids_depth_first(),
            vec![
                ComponentId("root".into()),
                ComponentId("left".into()),
                ComponentId("right".into()),
            ]
        );
        assert!(tree.find(&ComponentId("right".into())).is_some());
        assert!(tree.find(&ComponentId("missing".into())).is_none());
    }

    #[test]
    fn component_tree_renders_entire_subtree() {
        let mut root = StubComponent::new("root", EventResult::Ignored);
        root.symbol = 'R';
        let mut child = StubComponent::new("child", EventResult::Ignored);
        child.symbol = 'C';

        let tree = ComponentTree::new(
            ComponentNode::new(Box::new(root))
                .with_children(vec![ComponentNode::new(Box::new(child))]),
        );
        let mut frame = Frame::new(1, 1);

        tree.render(&RenderContext::default(), &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('C'));
    }

    #[test]
    fn component_tree_builds_vertical_layout_for_children() {
        let tree = ComponentTree::new(
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![
                    ComponentNode::new(Box::new(StubComponent::new("top", EventResult::Ignored))),
                    ComponentNode::new(Box::new(StubComponent::new(
                        "bottom",
                        EventResult::Ignored,
                    ))),
                ]),
        );

        let layout = tree.layout(&LayoutContext, Rect::new(0, 0, 8, 4));

        assert_eq!(layout.area, Rect::new(0, 0, 8, 4));
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.children[0].area, Rect::new(0, 0, 8, 2));
        assert_eq!(layout.children[1].area, Rect::new(0, 2, 8, 2));
    }

    #[test]
    fn component_tree_default_layout_splits_three_children_evenly() {
        let tree = ComponentTree::new(
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![
                    ComponentNode::new(Box::new(StubComponent::new("a", EventResult::Ignored))),
                    ComponentNode::new(Box::new(StubComponent::new("b", EventResult::Ignored))),
                    ComponentNode::new(Box::new(StubComponent::new("c", EventResult::Ignored))),
                ]),
        );

        let layout = tree.layout(&LayoutContext, Rect::new(0, 0, 9, 7));

        assert_eq!(layout.children[0].area, Rect::new(0, 0, 9, 2));
        assert_eq!(layout.children[1].area, Rect::new(0, 2, 9, 2));
        assert_eq!(layout.children[2].area, Rect::new(0, 4, 9, 3));
    }

    #[test]
    fn component_tree_reports_count_and_depth() {
        let tree = ComponentTree::new(
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![ComponentNode::new(Box::new(StubComponent::new(
                    "child",
                    EventResult::Ignored,
                )))
                .with_children(vec![ComponentNode::new(Box::new(StubComponent::new(
                    "grandchild",
                    EventResult::Ignored,
                )))])]),
        );

        assert_eq!(tree.component_count(), 3);
        assert_eq!(tree.max_depth(), 3);
    }

    #[test]
    fn layout_hints_choose_axis_specific_constraints() {
        let hints = LayoutHints {
            width: Some(4),
            height_pct: Some(60),
            grow: Some(2),
            ..LayoutHints::default()
        };

        assert_eq!(
            hints.constraint_for_axis(Axis::Horizontal),
            Some(Constraint::Fixed(4))
        );
        assert_eq!(
            hints.constraint_for_axis(Axis::Vertical),
            Some(Constraint::Percentage(60))
        );
    }

    #[test]
    fn component_tree_dispatches_events_from_deepest_child() {
        let tree_root =
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![ComponentNode::new(Box::new(StubComponent::new(
                    "child",
                    EventResult::Command(Command::Help),
                )))]);
        let mut tree = ComponentTree::new(tree_root);
        let mut ctx = EventContext::default();

        let result = tree.dispatch_event(&mut ctx, &Event::HelpRequested);

        assert_eq!(result, EventResult::Command(Command::Help));
        assert_eq!(ctx.commands, vec![Command::Help]);
    }

    #[test]
    fn bubbling_child_allows_parent_to_handle_event() {
        let child = StubComponent::new("child", EventResult::Bubble(Command::Help));
        let parent = StubComponent::new("root", EventResult::Command(Command::Quit));
        let mut tree = ComponentTree::new(
            ComponentNode::new(Box::new(parent))
                .with_children(vec![ComponentNode::new(Box::new(child))]),
        );
        let mut ctx = EventContext::default();

        let result = tree.dispatch_event(&mut ctx, &Event::HelpRequested);

        assert_eq!(result, EventResult::Command(Command::Quit));
        assert_eq!(ctx.commands, vec![Command::Quit]);
    }

    #[test]
    fn component_tree_collects_focusable_ids_only() {
        let mut root = StubComponent::new("root", EventResult::Ignored);
        root.focusable = true;
        let mut left = StubComponent::new("left", EventResult::Ignored);
        left.focusable = false;
        let mut right = StubComponent::new("right", EventResult::Ignored);
        right.focusable = true;

        let tree = ComponentTree::new(ComponentNode::new(Box::new(root)).with_children(vec![
            ComponentNode::new(Box::new(left)),
            ComponentNode::new(Box::new(right)),
        ]));

        assert_eq!(
            tree.focusable_ids_depth_first(),
            vec![ComponentId("root".into()), ComponentId("right".into())]
        );
    }

    #[test]
    fn layout_node_hit_tests_deepest_child() {
        let layout = LayoutNode::new(ComponentId("root".into()), Rect::new(0, 0, 10, 10))
            .with_children(vec![LayoutNode::new(
                ComponentId("child".into()),
                Rect::new(2, 2, 4, 4),
            )]);

        let hit = layout.find_deepest_at(Position::new(3, 3));

        assert_eq!(
            hit.map(|node| node.component_id.clone()),
            Some(ComponentId("child".into()))
        );
        assert!(layout.find_deepest_at(Position::new(20, 20)).is_none());
    }

    #[test]
    fn component_tree_dispatches_mouse_to_hit_component() {
        let tree_root =
            ComponentNode::new(Box::new(StubComponent::new("root", EventResult::Ignored)))
                .with_children(vec![ComponentNode::new(Box::new(StubComponent::new(
                    "button",
                    EventResult::Command(Command::Help),
                )))]);
        let layout = LayoutNode::new(ComponentId("root".into()), Rect::new(0, 0, 10, 10))
            .with_children(vec![LayoutNode::new(
                ComponentId("button".into()),
                Rect::new(1, 1, 3, 3),
            )]);
        let mut tree = ComponentTree::new(tree_root);
        let mut ctx = EventContext::default();
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 2,
            row: 2,
        });

        let result = tree.dispatch_mouse_event(&mut ctx, &layout, &event);

        assert_eq!(result, EventResult::Command(Command::Help));
        assert_eq!(ctx.commands, vec![Command::Help]);
    }

    #[test]
    fn targeted_dispatch_bubbles_to_parent_when_child_declines() {
        let tree_root = ComponentNode::new(Box::new(StubComponent::new(
            "root",
            EventResult::Command(Command::Quit),
        )))
        .with_children(vec![ComponentNode::new(Box::new(StubComponent::new(
            "child",
            EventResult::Bubble(Command::Help),
        )))]);
        let mut tree = ComponentTree::new(tree_root);
        let mut ctx = EventContext::default();

        let result = tree.dispatch_event_to_target(
            &mut ctx,
            &ComponentId("child".into()),
            &Event::HelpRequested,
        );

        assert_eq!(result, EventResult::Command(Command::Quit));
        assert_eq!(ctx.commands, vec![Command::Quit]);
    }

    #[test]
    fn scroll_target_prefers_focused_component() {
        let tree = ComponentTree::new(ComponentNode::new(Box::new(StubComponent::new(
            "root",
            EventResult::Ignored,
        ))));
        let layout = LayoutNode::new(ComponentId("root".into()), Rect::new(0, 0, 5, 5));
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 2,
            row: 2,
        });

        let target =
            tree.resolve_scroll_target(&layout, Some(&ComponentId("focus".into())), &event);

        assert_eq!(target, Some(ComponentId("focus".into())));
    }

    #[test]
    fn scroll_target_falls_back_to_hit_test() {
        let tree = ComponentTree::new(ComponentNode::new(Box::new(StubComponent::new(
            "root",
            EventResult::Ignored,
        ))));
        let layout =
            LayoutNode::new(ComponentId("root".into()), Rect::new(0, 0, 5, 5)).with_children(vec![
                LayoutNode::new(ComponentId("list".into()), Rect::new(1, 1, 2, 2)),
            ]);
        let event = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 1,
        });

        let target = tree.resolve_scroll_target(&layout, None, &event);

        assert_eq!(target, Some(ComponentId("list".into())));
    }
}
