// Knob widget
// Displays a rotary dial with 8-direction arrows representing value

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Style, TextAlign};

#[derive(Debug, Clone, PartialEq)]
pub struct Knob {
    id: ComponentId,
    title: Option<String>,
    value: f64,
    min: f64,
    max: f64,
    style: Style,
}

impl Knob {
    pub fn new(id: impl Into<String>, value: f64) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            value,
            min: 0.0,
            max: 100.0,
            style: Style::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_min_max(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    fn fraction(&self) -> f64 {
        if self.max <= self.min {
            0.0
        } else {
            let val = self.value.max(self.min).min(self.max);
            (val - self.min) / (self.max - self.min)
        }
    }

    fn arrow(&self) -> char {
        let arrows = ['↑', '↗', '→', '↘', '↓', '↙', '←', '↖'];
        let frac = self.fraction();
        let index = (frac * 7.0).round() as usize;
        arrows[index.min(7)]
    }
}

impl Component for Knob {
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

        let display_title = self.title.clone().unwrap_or_default();
        let arrow_char = self.arrow();
        let dial_str = format!("( {} )", arrow_char);
        let value_str = format!("{:.1}", self.value);

        if area.height >= 3 {
            let mut offset_y = area.y;
            if self.title.is_some() {
                let _ = frame.draw_text_aligned(
                    area.x,
                    offset_y,
                    area.width,
                    &display_title,
                    self.style.clone(),
                    TextAlign::Center,
                );
                offset_y += 1;
            }

            let dial_y = offset_y;
            let _ = frame.draw_text_aligned(
                area.x,
                dial_y,
                area.width,
                &dial_str,
                self.style.clone(),
                TextAlign::Center,
            );

            let value_y = dial_y + 1;
            if value_y < area.bottom() {
                let _ = frame.draw_text_aligned(
                    area.x,
                    value_y,
                    area.width,
                    &value_str,
                    self.style.clone(),
                    TextAlign::Center,
                );
            }
        } else {
            let line_y = area.y + (area.height - 1) / 2;
            let text = if self.title.is_some() {
                format!("{}: {} {}", display_title, dial_str, value_str)
            } else {
                format!("{} {}", dial_str, value_str)
            };
            let _ = frame.draw_text_aligned(
                area.x,
                line_y,
                area.width,
                &text,
                self.style.clone(),
                TextAlign::Center,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ScreenBuffer;
    use crate::testing::snapshot_buffer;

    #[test]
    fn knob_renders_direction_arrow_and_value() {
        let knob = Knob::new("sector_dial", 50.0).with_title("Warp Core");
        let ctx = RenderContext::new(Rect::new(0, 0, 15, 3));
        let mut frame = ScreenBuffer::new(15, 3);

        knob.render(&ctx, &mut frame);

        let snapshot = snapshot_buffer(&frame).replace('·', " ");
        assert!(snapshot.contains("Warp Core"));
        assert!(snapshot.contains("( ↓ )"));
        assert!(snapshot.contains("50.0"));
    }

    #[test]
    fn knob_renders_single_line_compact() {
        let knob = Knob::new("sector_dial", 100.0).with_title("Core");
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 1));
        let mut frame = ScreenBuffer::new(20, 1);

        knob.render(&ctx, &mut frame);

        let snapshot = snapshot_buffer(&frame).replace('·', " ");
        assert!(snapshot.contains("Core: ( ↖ ) 100.0"));
    }
}
