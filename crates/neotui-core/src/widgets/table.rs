use crate::component::{Component, EventContext, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::{ComponentId, Event, EventResult, KeyCode, ScrollDirection};
use crate::layout::Rect;
use crate::render::{Style, TextAlign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    key: String,
    title: String,
    width: u16,
    align: TextAlign,
}

impl TableColumn {
    pub fn new(key: impl Into<String>, title: impl Into<String>, width: u16) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            width,
            align: TextAlign::Left,
        }
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    id: ComponentId,
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    style: Style,
    header_style: Style,
    selected_style: Style,
    focused: bool,
    selected_index: usize,
    scroll_offset: usize,
}

impl Table {
    pub fn new<I, R>(id: impl Into<String>, columns: Vec<TableColumn>, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = String>,
    {
        Self {
            id: ComponentId(id.into()),
            columns,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().collect())
                .collect(),
            style: Style::default(),
            header_style: Style {
                bold: true,
                ..Style::default()
            },
            selected_style: Style {
                bold: true,
                ..Style::default()
            },
            focused: false,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    fn visible_rows(&self, area: &Rect) -> usize {
        usize::from(area.height.saturating_sub(1))
    }

    fn clamp_selection(&mut self) {
        if self.rows.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        } else {
            self.selected_index = self.selected_index.min(self.rows.len().saturating_sub(1));
            self.scroll_offset = self.scroll_offset.min(self.selected_index);
        }
    }

    fn sync_scroll_with_selection(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else {
            let bottom = self.scroll_offset.saturating_add(visible_rows);
            if self.selected_index >= bottom {
                self.scroll_offset = self
                    .selected_index
                    .saturating_add(1)
                    .saturating_sub(visible_rows);
            }
        }
    }

    fn draw_row(&self, frame: &mut Frame, area: &Rect, y: u16, cells: &[String], style: Style) {
        let mut x = area.x;
        for (index, column) in self.columns.iter().enumerate() {
            if x >= area.x.saturating_add(area.width) {
                break;
            }

            let available = area.x.saturating_add(area.width).saturating_sub(x);
            let width = column.width.min(available);
            if width == 0 {
                break;
            }

            let value = cells.get(index).map(String::as_str).unwrap_or("");
            let _ = frame.draw_text_aligned(x, y, width, value, style.clone(), column.align);
            x = x.saturating_add(width);

            if x < area.x.saturating_add(area.width) {
                let _ = frame.draw_text(x, y, " ", style.clone());
                x = x.saturating_add(1);
            }
        }
    }
}

impl Component for Table {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        let area = ctx.area();
        if area.is_empty() || self.columns.is_empty() {
            return;
        }

        let headers = self
            .columns
            .iter()
            .map(|column| column.title.clone())
            .collect::<Vec<_>>();
        self.draw_row(frame, area, area.y, &headers, self.header_style.clone());

        let visible_rows = self.visible_rows(area);
        for (visible_index, row) in self
            .rows
            .iter()
            .skip(self.scroll_offset)
            .take(visible_rows)
            .enumerate()
        {
            let absolute_index = self.scroll_offset.saturating_add(visible_index);
            let style = if absolute_index == self.selected_index {
                self.selected_style.clone()
            } else {
                self.style.clone()
            };
            let y = area
                .y
                .saturating_add(1)
                .saturating_add(u16::try_from(visible_index).unwrap_or(0));
            self.draw_row(frame, area, y, row, style);

            if self.focused && absolute_index == self.selected_index {
                let _ = frame.draw_text(area.x, y, ">", self.selected_style.clone());
            }
        }
    }

    fn on_event(&mut self, _ctx: &mut EventContext, event: &Event) -> EventResult {
        self.clamp_selection();
        match event {
            Event::FocusGained(id) if *id == self.id => {
                self.focused = true;
                EventResult::RequestRender
            }
            Event::FocusLost(id) if *id == self.id => {
                self.focused = false;
                EventResult::RequestRender
            }
            Event::Key(key) if self.focused => {
                let previous = self.selected_index;
                match key.code {
                    KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                    KeyCode::Down => {
                        if !self.rows.is_empty() {
                            self.selected_index =
                                (self.selected_index + 1).min(self.rows.len().saturating_sub(1));
                        }
                    }
                    KeyCode::Home => self.selected_index = 0,
                    KeyCode::End => {
                        if !self.rows.is_empty() {
                            self.selected_index = self.rows.len().saturating_sub(1);
                        }
                    }
                    KeyCode::PageUp => {
                        self.selected_index = self.selected_index.saturating_sub(5);
                    }
                    KeyCode::PageDown => {
                        if !self.rows.is_empty() {
                            self.selected_index =
                                (self.selected_index + 5).min(self.rows.len().saturating_sub(1));
                        }
                    }
                    _ => return EventResult::Ignored,
                }

                if previous != self.selected_index {
                    self.sync_scroll_with_selection(5);
                    EventResult::RequestRender
                } else {
                    EventResult::Consumed
                }
            }
            Event::Scroll(scroll) => {
                let previous = self.selected_index;
                match scroll.direction {
                    ScrollDirection::Up => {
                        self.selected_index = self
                            .selected_index
                            .saturating_sub(usize::from(scroll.amount));
                    }
                    ScrollDirection::Down => {
                        if !self.rows.is_empty() {
                            self.selected_index = self
                                .selected_index
                                .saturating_add(usize::from(scroll.amount))
                                .min(self.rows.len().saturating_sub(1));
                        }
                    }
                }

                if previous != self.selected_index {
                    self.sync_scroll_with_selection(5);
                    EventResult::RequestRender
                } else {
                    EventResult::Consumed
                }
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::event::{Event, KeyEvent, KeyModifiers};
    use crate::render::{Color, ScreenBuffer};
    use crate::testing::snapshot_buffer;

    fn columns() -> Vec<TableColumn> {
        vec![
            TableColumn::new("service", "Service", 8),
            TableColumn::new("state", "State", 6).with_align(TextAlign::Center),
            TableColumn::new("latency", "P95", 4).with_align(TextAlign::Right),
        ]
    }

    #[test]
    fn table_renders_header_and_rows() {
        let table = Table::new(
            "services",
            columns(),
            [
                vec!["api".into(), "ok".into(), "12".into()],
                vec!["jobs".into(), "warn".into(), "85".into()],
            ],
        );
        let ctx = RenderContext::new(Rect::new(0, 0, 22, 3));
        let mut frame = ScreenBuffer::new(22, 3);

        table.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('S'));
        assert_eq!(frame.get(9, 1).map(|cell| cell.symbol), Some(' '));
        assert_eq!(frame.get(18, 2).map(|cell| cell.symbol), Some('8'));
        assert_eq!(frame.get(19, 2).map(|cell| cell.symbol), Some('5'));
    }

    #[test]
    fn table_moves_selection_on_keys() {
        let mut table = Table::new(
            "services",
            columns(),
            [
                vec!["api".into(), "ok".into(), "12".into()],
                vec!["jobs".into(), "warn".into(), "85".into()],
            ],
        );
        let mut ctx = EventContext::default();

        let _ = table.on_event(
            &mut ctx,
            &Event::FocusGained(ComponentId("services".into())),
        );
        let result = table.on_event(
            &mut ctx,
            &Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::default(),
            }),
        );

        assert_eq!(result, EventResult::RequestRender);
        assert_eq!(table.selected_index, 1);
    }

    #[test]
    fn table_applies_selected_style() {
        let selected_style = Style {
            fg: Color::Red,
            bold: true,
            ..Style::default()
        };
        let table = Table::new(
            "services",
            columns(),
            [vec!["api".into(), "ok".into(), "12".into()]],
        )
        .with_selected_style(selected_style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 22, 2));
        let mut frame = ScreenBuffer::new(22, 2);

        table.render(&ctx, &mut frame);

        assert_eq!(
            frame.get(0, 1).map(|cell| cell.style.clone()),
            Some(selected_style)
        );
    }

    #[test]
    fn table_snapshot_stays_stable() {
        let table = Table::new(
            "services",
            columns(),
            [
                vec!["api".into(), "ok".into(), "12".into()],
                vec!["jobs".into(), "warn".into(), "85".into()],
            ],
        );
        let ctx = RenderContext::new(Rect::new(0, 0, 22, 3));
        let mut frame = ScreenBuffer::new(22, 3);

        table.render(&ctx, &mut frame);

        let snapshot = snapshot_buffer(&frame);

        assert!(snapshot.contains("Service"));
        assert!(snapshot.contains("api"));
        assert!(snapshot.contains("jobs"));
        assert!(snapshot.contains("85"));
    }
}
