// Sparkline widget
// Displays a sequence of numeric data points as a compact bar-height trend line

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Color, Style};

#[derive(Debug, Clone, PartialEq)]
pub struct Sparkline {
    id: ComponentId,
    title: Option<String>,
    values: Vec<f64>,
    style: Style,
}

const BAR_CHARS: [char; 8] = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

impl Sparkline {
    pub fn new(id: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            values,
            style: Style::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for Sparkline {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn layout(&self, _ctx: &LayoutContext, area: Rect) -> LayoutNode {
        LayoutNode::new(self.id(), area)
    }

    fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        let area = ctx.area();
        if area.is_empty() || self.values.is_empty() {
            return;
        }

        let (title_height, spark_y) = if self.title.is_some() && area.height >= 2 {
            (1, area.y + 1)
        } else {
            (0, area.y)
        };

        if title_height > 0 {
            if let Some(ref title) = self.title {
                let muted_style = Style {
                    fg: Color::Indexed(8),
                    ..Style::default()
                };
                let _ = frame.draw_text(area.x, area.y, title, muted_style);
            }
        }

        let mut spark_x = area.x;
        let mut spark_width = area.width;

        if title_height == 0 {
            if let Some(ref title) = self.title {
                let title_len = title
                    .chars()
                    .count()
                    .min(usize::from(area.width.saturating_sub(4)));
                if title_len > 0 {
                    let _ = frame.draw_text(
                        area.x,
                        area.y,
                        &format!("{}:", &title[..title_len]),
                        Style {
                            fg: Color::Indexed(8),
                            ..Style::default()
                        },
                    );
                    let shift = u16::try_from(title_len + 2).unwrap_or(0);
                    spark_x = spark_x.saturating_add(shift);
                    spark_width = spark_width.saturating_sub(shift);
                }
            }
        }

        if spark_width == 0 {
            return;
        }

        // Take only the latest values that fit in spark_width
        let visible_count = usize::from(spark_width).min(self.values.len());
        let start_index = self.values.len().saturating_sub(visible_count);
        let visible_values = &self.values[start_index..];

        // Find min and max of visible values to auto-scale
        let mut min = visible_values[0];
        let mut max = visible_values[0];
        for &val in visible_values {
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }

        let range = max - min;

        // Render each visible value
        for (i, &val) in visible_values.iter().enumerate() {
            let i_u16 = u16::try_from(i).unwrap_or(0);
            if i_u16 >= spark_width {
                break;
            }

            let char_index = if range == 0.0 {
                3 // default to mid-range block if flat
            } else {
                let frac = (val - min) / range;
                let idx = (frac * 7.0).round() as usize;
                idx.min(7)
            };

            let symbol = BAR_CHARS[char_index];
            let _ = frame.set(
                spark_x + i_u16,
                spark_y,
                crate::render::Cell {
                    symbol,
                    style: self.style.clone(),
                },
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
    fn sparkline_renders_blocks() {
        let spark = Sparkline::new("history", vec![10.0, 20.0, 30.0, 40.0]);
        let ctx = RenderContext::new(Rect::new(0, 0, 4, 1));
        let mut frame = ScreenBuffer::new(4, 1);

        spark.render(&ctx, &mut frame);

        // Scaled to min=10, max=40, values map to frac: 0.0, 0.33, 0.67, 1.0
        // Char index: 0.0*7=0 (' '), 0.33*7=2 ('▃'), 0.67*7=5 ('▆'), 1.0*7=7 ('█')
        assert_eq!(snapshot_buffer(&frame), "·▃▆█");
    }
}
