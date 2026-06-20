// State model
// Minimal runtime state for focus and dirty tracking

use crate::data::{ActionSnapshot, ActionStore, DataSnapshot, DataStore};
use crate::event::{ComponentId, ScrollDirection, ScrollEvent};
use crate::forms::{FormSpec, FormStore};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateStore {
    focused: Option<ComponentId>,
    dirty_components: HashSet<ComponentId>,
    scroll_offsets: HashMap<ComponentId, u16>,
    data: DataStore,
    actions: ActionStore,
    forms: FormStore,
    layout_dirty: bool,
    render_dirty: bool,
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn focused(&self) -> Option<&ComponentId> {
        self.focused.as_ref()
    }

    pub fn set_focus(&mut self, component_id: Option<ComponentId>) -> bool {
        if self.focused == component_id {
            return false;
        }

        self.focused = component_id;
        self.render_dirty = true;
        true
    }

    pub fn focus_next(&mut self, focus_order: &[ComponentId]) -> Option<ComponentId> {
        self.advance_focus(focus_order, true)
    }

    pub fn focus_previous(&mut self, focus_order: &[ComponentId]) -> Option<ComponentId> {
        self.advance_focus(focus_order, false)
    }

    pub fn mark_dirty(&mut self, component_id: ComponentId) -> bool {
        let inserted = self.dirty_components.insert(component_id);

        if inserted {
            self.render_dirty = true;
        }

        inserted
    }

    pub fn scroll_offset(&self, component_id: &ComponentId) -> u16 {
        self.scroll_offsets.get(component_id).copied().unwrap_or(0)
    }

    pub fn data(&self) -> &DataStore {
        &self.data
    }

    pub fn actions(&self) -> &ActionStore {
        &self.actions
    }

    pub fn forms(&self) -> &FormStore {
        &self.forms
    }

    pub fn initialize_forms(&mut self, forms: &[FormSpec]) -> bool {
        let mut changed = false;
        for form in forms {
            for field in &form.fields {
                if let Some(initial) = &field.initial {
                    changed |= self
                        .forms
                        .set(form.id.clone(), field.id.clone(), initial.clone());
                }
            }
        }
        if changed {
            self.render_dirty = true;
        }
        changed
    }

    pub fn set_form_value(
        &mut self,
        form_id: impl Into<String>,
        field_id: impl Into<String>,
        value: crate::dsl::Value,
    ) -> bool {
        let changed = self.forms.set(form_id, field_id, value);
        if changed {
            self.render_dirty = true;
        }
        changed
    }

    pub fn set_data_snapshot(
        &mut self,
        source_id: impl Into<String>,
        snapshot: DataSnapshot,
    ) -> bool {
        let changed = self.data.set(source_id, snapshot);
        if changed {
            self.render_dirty = true;
        }
        changed
    }

    pub fn set_action_snapshot(
        &mut self,
        action_id: impl Into<String>,
        snapshot: ActionSnapshot,
    ) -> bool {
        let changed = self.actions.set(action_id, snapshot);
        if changed {
            self.render_dirty = true;
        }
        changed
    }

    pub fn set_scroll_offset(&mut self, component_id: ComponentId, offset: u16) -> bool {
        let changed = self.scroll_offset(&component_id) != offset;

        if changed {
            self.scroll_offsets.insert(component_id.clone(), offset);
            let _ = self.mark_dirty(component_id);
        }

        changed
    }

    pub fn apply_scroll(
        &mut self,
        component_id: ComponentId,
        event: &ScrollEvent,
        max_offset: u16,
    ) -> u16 {
        let current = self.scroll_offset(&component_id);
        let next = match event.direction {
            ScrollDirection::Up => current.saturating_sub(event.amount),
            ScrollDirection::Down => current.saturating_add(event.amount).min(max_offset),
        };

        let _ = self.set_scroll_offset(component_id, next);
        next
    }

    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.render_dirty = true;
    }

    pub fn is_component_dirty(&self, component_id: &ComponentId) -> bool {
        self.dirty_components.contains(component_id)
    }

    pub fn dirty_components(&self) -> &HashSet<ComponentId> {
        &self.dirty_components
    }

    pub fn is_layout_dirty(&self) -> bool {
        self.layout_dirty
    }

    pub fn is_render_dirty(&self) -> bool {
        self.render_dirty
    }

    pub fn should_render(&self) -> bool {
        self.render_dirty || self.layout_dirty || !self.dirty_components.is_empty()
    }

    pub fn clear_component_dirty(&mut self, component_id: &ComponentId) -> bool {
        self.dirty_components.remove(component_id)
    }

    pub fn clear_render_dirty(&mut self) {
        self.render_dirty = false;
    }

    pub fn clear_layout_dirty(&mut self) {
        self.layout_dirty = false;
    }

    pub fn clear_all_dirty(&mut self) {
        self.dirty_components.clear();
        self.layout_dirty = false;
        self.render_dirty = false;
    }

    fn advance_focus(&mut self, focus_order: &[ComponentId], forward: bool) -> Option<ComponentId> {
        if focus_order.is_empty() {
            return None;
        }

        let next_index = match self.focused() {
            Some(current) => {
                let current_index = focus_order.iter().position(|id| id == current).unwrap_or(0);
                if forward {
                    (current_index + 1) % focus_order.len()
                } else if current_index == 0 {
                    focus_order.len() - 1
                } else {
                    current_index - 1
                }
            }
            None => {
                if forward {
                    0
                } else {
                    focus_order.len() - 1
                }
            }
        };

        let next = focus_order[next_index].clone();
        let _ = self.set_focus(Some(next.clone()));
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> ComponentId {
        ComponentId(value.to_string())
    }

    #[test]
    fn focus_changes_mark_render_dirty() {
        let mut state = StateStore::new();

        assert!(state.set_focus(Some(id("button"))));
        assert_eq!(state.focused(), Some(&id("button")));
        assert!(state.is_render_dirty());
    }

    #[test]
    fn setting_same_focus_is_noop() {
        let mut state = StateStore::new();
        let _ = state.set_focus(Some(id("button")));
        state.clear_render_dirty();

        assert!(!state.set_focus(Some(id("button"))));
        assert!(!state.is_render_dirty());
    }

    #[test]
    fn dirty_components_are_tracked_once() {
        let mut state = StateStore::new();

        assert!(state.mark_dirty(id("label")));
        assert!(!state.mark_dirty(id("label")));
        assert!(state.is_component_dirty(&id("label")));
        assert_eq!(state.dirty_components().len(), 1);
        assert!(state.is_render_dirty());
    }

    #[test]
    fn layout_dirty_implies_render() {
        let mut state = StateStore::new();

        state.mark_layout_dirty();

        assert!(state.is_layout_dirty());
        assert!(state.is_render_dirty());
        assert!(state.should_render());
    }

    #[test]
    fn clearing_specific_component_keeps_other_dirty_flags() {
        let mut state = StateStore::new();
        let _ = state.mark_dirty(id("left"));
        let _ = state.mark_dirty(id("right"));
        state.mark_layout_dirty();

        assert!(state.clear_component_dirty(&id("left")));
        assert!(!state.is_component_dirty(&id("left")));
        assert!(state.is_component_dirty(&id("right")));
        assert!(state.is_layout_dirty());
        assert!(state.should_render());
    }

    #[test]
    fn clear_all_dirty_resets_store() {
        let mut state = StateStore::new();
        let _ = state.mark_dirty(id("root"));
        let _ = state.set_focus(Some(id("root")));
        state.mark_layout_dirty();

        state.clear_all_dirty();

        assert!(!state.is_render_dirty());
        assert!(!state.is_layout_dirty());
        assert!(state.dirty_components().is_empty());
        assert_eq!(state.focused(), Some(&id("root")));
        assert!(!state.should_render());
    }

    #[test]
    fn focus_next_wraps_across_focus_order() {
        let mut state = StateStore::new();
        let focus_order = vec![id("first"), id("second"), id("third")];

        assert_eq!(state.focus_next(&focus_order), Some(id("first")));
        assert_eq!(state.focus_next(&focus_order), Some(id("second")));
        assert_eq!(state.focus_next(&focus_order), Some(id("third")));
        assert_eq!(state.focus_next(&focus_order), Some(id("first")));
    }

    #[test]
    fn focus_previous_wraps_backwards() {
        let mut state = StateStore::new();
        let focus_order = vec![id("first"), id("second"), id("third")];

        assert_eq!(state.focus_previous(&focus_order), Some(id("third")));
        assert_eq!(state.focus_previous(&focus_order), Some(id("second")));
    }

    #[test]
    fn focus_navigation_ignores_empty_order() {
        let mut state = StateStore::new();

        assert_eq!(state.focus_next(&[]), None);
        assert_eq!(state.focus_previous(&[]), None);
        assert_eq!(state.focused(), None);
    }

    #[test]
    fn scroll_offsets_default_to_zero() {
        let state = StateStore::new();
        assert_eq!(state.scroll_offset(&id("list")), 0);
    }

    #[test]
    fn setting_scroll_offset_marks_component_dirty() {
        let mut state = StateStore::new();

        assert!(state.set_scroll_offset(id("list"), 3));
        assert_eq!(state.scroll_offset(&id("list")), 3);
        assert!(state.is_component_dirty(&id("list")));
        assert!(state.is_render_dirty());
    }

    #[test]
    fn setting_action_snapshot_marks_render_dirty() {
        let mut state = StateStore::new();

        assert!(state.set_action_snapshot("refresh", ActionSnapshot::loading()));
        assert_eq!(
            state
                .actions()
                .get("refresh")
                .map(|snapshot| &snapshot.status),
            Some(&crate::data::ActionStatus::Loading)
        );
        assert!(state.is_render_dirty());
    }

    #[test]
    fn setting_form_value_marks_render_dirty() {
        let mut state = StateStore::new();

        assert!(state.set_form_value(
            "incident",
            "summary",
            crate::dsl::Value::String("Disk full".into())
        ));
        assert_eq!(
            state.forms().get("incident", "summary"),
            Some(&crate::dsl::Value::String("Disk full".into()))
        );
        assert!(state.is_render_dirty());
    }

    #[test]
    fn initializing_forms_applies_initial_values() {
        let mut state = StateStore::new();
        let forms = vec![crate::forms::FormSpec {
            id: "incident".into(),
            fields: vec![crate::forms::FormFieldSpec {
                id: "summary".into(),
                kind: crate::forms::FormFieldKind::Text,
                initial: Some(crate::dsl::Value::String("Disk full".into())),
                required: true,
            }],
        }];

        assert!(state.initialize_forms(&forms));
        assert_eq!(
            state.forms().get("incident", "summary"),
            Some(&crate::dsl::Value::String("Disk full".into()))
        );
    }

    #[test]
    fn apply_scroll_moves_offset_down_and_up_with_clamp() {
        let mut state = StateStore::new();
        let component = id("list");

        let down = state.apply_scroll(
            component.clone(),
            &ScrollEvent {
                direction: ScrollDirection::Down,
                amount: 4,
            },
            10,
        );
        let up = state.apply_scroll(
            component.clone(),
            &ScrollEvent {
                direction: ScrollDirection::Up,
                amount: 2,
            },
            10,
        );
        let clamped = state.apply_scroll(
            component.clone(),
            &ScrollEvent {
                direction: ScrollDirection::Down,
                amount: 99,
            },
            5,
        );

        assert_eq!(down, 4);
        assert_eq!(up, 2);
        assert_eq!(clamped, 5);
        assert_eq!(state.scroll_offset(&component), 5);
    }

    #[test]
    fn scroll_up_never_underflows() {
        let mut state = StateStore::new();
        let component = id("list");

        let offset = state.apply_scroll(
            component.clone(),
            &ScrollEvent {
                direction: ScrollDirection::Up,
                amount: 5,
            },
            10,
        );

        assert_eq!(offset, 0);
        assert_eq!(state.scroll_offset(&component), 0);
    }
}
