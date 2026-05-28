// Metric widget
// Displays key numerical operational values, status indicators, and delta variations

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Color, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metric {
    id: ComponentId,
    title: String,
    value: String,
    delta: Option<String>,
    status: String, // "normal", "warning", "critical", "info"
    style: Style,
    status_style: Option<Style>,
}

impl Metric {
    pub fn new(id: impl Into<String>, title: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: title.into(),
            value: value.into(),
            delta: None,
            status: "normal".into(),
            style: Style::default(),
            status_style: None,
        }
    }

    pub fn with_delta(mut self, delta: impl Into<String>) -> Self {
        self.delta = Some(delta.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
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

    pub fn status_style(&self) -> Style {
        if let Some(ref style) = self.status_style {
            return style.clone();
        }
        let fg = match self.status.to_lowercase().as_str() {
            "critical" | "danger" => Color::Red,
            "warning" => Color::Yellow,
            "info" => Color::Cyan,
            "normal" | "success" => Color::Green,
            _ => Color::Reset,
        };
        Style {
            fg,
            bold: true,
            ..Style::default()
        }
    }
}

impl Component for Metric {
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

        let muted_style = Style {
            fg: Color::Indexed(8), // Grey / Muted
            ..Style::default()
        };

        let status_style = self.status_style();

        if area.height == 1 {
            // Compact view: "Title: Value (Delta)"
            let display_text = if let Some(ref delta) = self.delta {
                format!("{}: {} ({})", self.title, self.value, delta)
            } else {
                format!("{}: {}", self.title, self.value)
            };
            let _ = frame.draw_text(area.x, area.y, &display_text, self.style.clone());
        } else if area.height == 2 {
            // Two-row view:
            // Row 0: Title
            // Row 1: Value (Delta)
            let _ = frame.draw_text(area.x, area.y, &self.title, muted_style);
            let val_text = if let Some(ref delta) = self.delta {
                format!("{} ({})", self.value, delta)
            } else {
                self.value.clone()
            };
            let _ = frame.draw_text(area.x, area.y + 1, &val_text, status_style);
        } else {
            // Three or more rows (standard dashboard HUD style):
            // Row 0: Title (muted text)
            // Row 1: Value (highlight/bold status color)
            // Row 2: Delta (secondary text)
            let _ = frame.draw_text(area.x, area.y, &self.title, muted_style);
            let _ = frame.draw_text(area.x, area.y + 1, &self.value, status_style);
            if let Some(ref delta) = self.delta {
                let _ = frame.draw_text(area.x, area.y + 2, delta, self.style.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ScreenBuffer;
    use crate::testing::snapshot_buffer;

    #[test]
    fn metric_renders_compact_one_line() {
        let metric = Metric::new("cpu", "CPU", "42%").with_delta("+2%");
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 1));
        let mut frame = ScreenBuffer::new(20, 1);

        metric.render(&ctx, &mut frame);

        assert_eq!(
            snapshot_buffer(&frame).trim_end_matches('·'),
            "CPU:·42%·(+2%)"
        );
    }

    #[test]
    fn metric_renders_three_rows() {
        let metric = Metric::new("cpu", "CPU", "42%")
            .with_delta("+2%")
            .with_status("critical");
        let ctx = RenderContext::new(Rect::new(0, 0, 15, 3));
        let mut frame = ScreenBuffer::new(15, 3);

        metric.render(&ctx, &mut frame);

        let snapshot = snapshot_buffer(&frame);
        let lines: Vec<&str> = snapshot.lines().collect();
        assert!(lines[0].starts_with("CPU"));
        assert!(lines[1].starts_with("42%"));
        assert!(lines[2].starts_with("+2%"));
    }
}
