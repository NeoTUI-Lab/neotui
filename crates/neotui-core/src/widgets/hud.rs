// HUD layout primitives
// KeyValueRow and StatusStrip for operational dashboards and instrumentations

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Color, Style, TextAlign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValueRow {
    id: ComponentId,
    key: String,
    value: String,
    connector: char,
    style: Style,
}

impl KeyValueRow {
    pub fn new(id: impl Into<String>, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            key: key.into(),
            value: value.into(),
            connector: '.',
            style: Style::default(),
        }
    }

    pub fn with_connector(mut self, connector: char) -> Self {
        self.connector = connector;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for KeyValueRow {
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

        let key_len = self.key.chars().count();
        let val_len = self.value.chars().count();
        let total_width = usize::from(area.width);

        if key_len + val_len >= total_width {
            // No room for connector, just draw key and value next to each other or truncate
            let key_space = total_width.saturating_sub(val_len).max(1);
            let clipped_key: String = self.key.chars().take(key_space).collect();
            let _ = frame.draw_text(area.x, area.y, &clipped_key, self.style.clone());

            let val_x = area.x.saturating_add(u16::try_from(key_space).unwrap_or(0));
            let val_width = area
                .width
                .saturating_sub(u16::try_from(key_space).unwrap_or(0));
            if val_width > 0 {
                let _ = frame.draw_text_aligned(
                    val_x,
                    area.y,
                    val_width,
                    &self.value,
                    self.style.clone(),
                    TextAlign::Right,
                );
            }
        } else {
            // Draw Key
            let _ = frame.draw_text(area.x, area.y, &self.key, self.style.clone());

            // Draw Connector dots
            let conn_start = area.x.saturating_add(u16::try_from(key_len).unwrap_or(0));
            let conn_width = u16::try_from(total_width - key_len - val_len).unwrap_or(0);
            let conn_str: String = std::iter::repeat(self.connector)
                .take(usize::from(conn_width))
                .collect();

            let conn_style = Style {
                fg: Color::Indexed(8), // muted connector
                ..self.style.clone()
            };
            let _ = frame.draw_text(conn_start, area.y, &conn_str, conn_style);

            // Draw Value
            let val_x = conn_start.saturating_add(conn_width);
            let _ = frame.draw_text(val_x, area.y, &self.value, self.style.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusStrip {
    id: ComponentId,
    text: String,
    status: String,
    /// Optional fill character key: "chevron" → ▸, "arrow" → →, "dots" → ·
    fill: Option<String>,
    style: Style,
    status_style: Option<Style>,
}

impl StatusStrip {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            text: text.into(),
            status: "normal".into(),
            fill: None,
            style: Style::default(),
            status_style: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_fill(mut self, fill: impl Into<String>) -> Self {
        self.fill = Some(fill.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_status_style(mut self, style: Style) -> Self {
        self.status_style = Some(style);
        self
    }

    fn status_colors(&self) -> (Color, Color) {
        if let Some(ref style) = self.status_style {
            return (style.fg.clone(), style.bg.clone());
        }
        // Returns (fg, bg) for the status block
        match self.status.to_lowercase().as_str() {
            "critical" | "danger" => (Color::White, Color::Red),
            "warning" => (Color::Black, Color::Yellow),
            "info" => (Color::Black, Color::Cyan),
            "normal" | "success" => (Color::White, Color::Green),
            _ => (Color::White, Color::Indexed(8)),
        }
    }
}

impl Component for StatusStrip {
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

        let (status_fg, status_bg) = self.status_colors();
        let status_tag = format!(" {} ", self.status.to_uppercase());
        let status_len = u16::try_from(status_tag.chars().count()).unwrap_or(0);

        // Draw background track for the entire strip
        let bg_cell = crate::render::Cell {
            symbol: ' ',
            style: self.style.clone(),
        };
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let _ = frame.set(x, y, bg_cell.clone());
            }
        }

        // Draw Status Tag block at the left
        let tag_style = Style {
            fg: status_fg,
            bg: status_bg,
            bold: true,
            ..Style::default()
        };

        if area.width >= status_len {
            for x in 0..status_len {
                let symbol = status_tag.chars().nth(usize::from(x)).unwrap_or(' ');
                let _ = frame.set(
                    area.x + x,
                    area.y,
                    crate::render::Cell {
                        symbol,
                        style: tag_style.clone(),
                    },
                );
            }

            // Draw status text next to the block
            let text_x = area.x.saturating_add(status_len).saturating_add(1);
            let text_width = area.width.saturating_sub(status_len).saturating_sub(1);
            if text_width > 0 {
                let text_chars = self.text.chars().count() as u16;
                let _ = frame.draw_text(text_x, area.y, &self.text, self.style.clone());

                // Draw fill pattern in remaining space after text
                if let Some(ref fill_key) = self.fill {
                    let fill_char = match fill_key.as_str() {
                        "chevron" => '▸',
                        "arrow" => '→',
                        "dots" => '·',
                        _ => '▸',
                    };
                    let fill_start = text_x.saturating_add(text_chars).saturating_add(1);
                    let fill_style = Style {
                        fg: Color::Indexed(8),
                        ..Style::default()
                    };
                    for fx in fill_start..area.right() {
                        let _ = frame.set(
                            fx,
                            area.y,
                            crate::render::Cell {
                                symbol: fill_char,
                                style: fill_style.clone(),
                            },
                        );
                    }
                }
            }
        } else {
            // Width is too small, just draw what fits of the tag
            let _ = frame.draw_text(area.x, area.y, &status_tag, tag_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ScreenBuffer;
    use crate::testing::snapshot_buffer;

    #[test]
    fn key_value_row_renders_with_connector() {
        let row = KeyValueRow::new("kv", "CPU", "84.2%").with_connector('.');
        let ctx = RenderContext::new(Rect::new(0, 0, 15, 1));
        let mut frame = ScreenBuffer::new(15, 1);

        row.render(&ctx, &mut frame);

        assert_eq!(snapshot_buffer(&frame), "CPU.......84.2%");
    }

    #[test]
    fn status_strip_renders_block_tag() {
        let strip = StatusStrip::new("status", "System Online").with_status("normal");
        let ctx = RenderContext::new(Rect::new(0, 0, 24, 1));
        let mut frame = ScreenBuffer::new(24, 1);

        strip.render(&ctx, &mut frame);

        let snapshot = snapshot_buffer(&frame);
        assert!(snapshot.contains("NORMAL"));
        assert!(snapshot.contains("System·Online"));
    }
}
