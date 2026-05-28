// Gauge widget
// Displays horizontal or vertical progress bars indicating status level

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Color, Style};
use crate::widgets::DividerOrientation;

/// Controls the visual fill pattern for the Gauge bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GaugeFillStyle {
    /// Uniform solid fill: `████░░░░` (default)
    #[default]
    Solid,
    /// Gradient fill with degrading block density: `████▓▒░░`
    /// Creates a depth/energy-drain effect common in FUI dashboards.
    Gradient,
    /// Block fill with a single precision marker `▌` at the fill boundary: `████▌░░░`
    Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gauge {
    id: ComponentId,
    title: Option<String>,
    value: f64,
    min: f64,
    max: f64,
    orientation: DividerOrientation,
    fill_style: GaugeFillStyle,
    style: Style,
    filled_style: Style,
}

impl Gauge {
    pub fn new(id: impl Into<String>, value: f64) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            value,
            min: 0.0,
            max: 100.0,
            orientation: DividerOrientation::Horizontal,
            fill_style: GaugeFillStyle::Solid,
            style: Style {
                fg: Color::Indexed(8), // track color
                ..Style::default()
            },
            filled_style: Style {
                fg: Color::Green,
                bold: true,
                ..Style::default()
            },
        }
    }

    pub fn with_fill_style(mut self, fill_style: GaugeFillStyle) -> Self {
        self.fill_style = fill_style;
        self
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

    pub fn with_orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    fn fraction(&self) -> f64 {
        if self.max <= self.min {
            0.0
        } else {
            let val = self.value.max(self.min).min(self.max);
            (val - self.min) / (self.max - self.min)
        }
    }
}

impl Component for Gauge {
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

        let frac = self.fraction();

        match self.orientation {
            DividerOrientation::Horizontal => {
                let (title_height, bar_y) = if self.title.is_some() && area.height >= 2 {
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

                // Calculate horizontal gauge bar width
                let mut bar_x = area.x;
                let mut bar_width = area.width;

                // If height is 1 and title exists, split the width
                if title_height == 0 {
                    if let Some(ref title) = self.title {
                        let title_len = title
                            .chars()
                            .count()
                            .min(usize::from(area.width.saturating_sub(5)));
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
                            bar_x = bar_x.saturating_add(shift);
                            bar_width = bar_width.saturating_sub(shift);
                        }
                    }
                }

                if bar_width > 0 {
                    let filled_chars = (frac * f64::from(bar_width)).round() as u16;
                    let filled_chars = filled_chars.min(bar_width);

                    // Draw filled portion with the selected fill style
                    for i in 0..filled_chars {
                        let symbol = match self.fill_style {
                            GaugeFillStyle::Solid => '█',
                            GaugeFillStyle::Block => {
                                if i + 1 == filled_chars {
                                    '▌'
                                } else {
                                    '█'
                                }
                            }
                            GaugeFillStyle::Gradient => {
                                // Last 15% of filled portion → ▒, next 25% → ▓, rest → █
                                let from_end = filled_chars.saturating_sub(i);
                                let pct_from_end =
                                    f64::from(from_end) / f64::from(filled_chars.max(1));
                                if pct_from_end <= 0.15 {
                                    '▒'
                                } else if pct_from_end <= 0.40 {
                                    '▓'
                                } else {
                                    '█'
                                }
                            }
                        };
                        let _ = frame.set(
                            bar_x + i,
                            bar_y,
                            crate::render::Cell {
                                symbol,
                                style: self.filled_style.clone(),
                            },
                        );
                    }
                    // Draw unfilled portion
                    for i in filled_chars..bar_width {
                        let _ = frame.set(
                            bar_x + i,
                            bar_y,
                            crate::render::Cell {
                                symbol: '░',
                                style: self.style.clone(),
                            },
                        );
                    }
                }
            }
            DividerOrientation::Vertical => {
                let (title_height, bar_y, bar_height) = if self.title.is_some() && area.height >= 3
                {
                    (1, area.y + 1, area.height.saturating_sub(1))
                } else {
                    (0, area.y, area.height)
                };

                if title_height > 0 {
                    if let Some(ref title) = self.title {
                        let _ = frame.draw_text(
                            area.x,
                            area.y,
                            &title[..1.min(title.len())],
                            self.style.clone(),
                        );
                    }
                }

                if bar_height > 0 {
                    let filled_chars = (frac * f64::from(bar_height)).round() as u16;
                    let filled_chars = filled_chars.min(bar_height);

                    // Draw vertical bar from bottom to top
                    for i in 0..bar_height {
                        let cell_y = bar_y + bar_height - 1 - i;
                        let cell = if i < filled_chars {
                            crate::render::Cell {
                                symbol: '█',
                                style: self.filled_style.clone(),
                            }
                        } else {
                            crate::render::Cell {
                                symbol: '░',
                                style: self.style.clone(),
                            }
                        };
                        let _ = frame.set(area.x, cell_y, cell);
                    }
                }
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
    fn gauge_renders_horizontal_bar() {
        let gauge = Gauge::new("mem", 50.0);
        let ctx = RenderContext::new(Rect::new(0, 0, 10, 1));
        let mut frame = ScreenBuffer::new(10, 1);

        gauge.render(&ctx, &mut frame);

        assert_eq!(snapshot_buffer(&frame), "█████░░░░░");
    }

    #[test]
    fn gauge_renders_vertical_bar() {
        let gauge = Gauge::new("cpu", 25.0).with_orientation(DividerOrientation::Vertical);
        let ctx = RenderContext::new(Rect::new(0, 0, 1, 4));
        let mut frame = ScreenBuffer::new(1, 4);

        gauge.render(&ctx, &mut frame);

        assert_eq!(snapshot_buffer(&frame), "░\n░\n░\n█");
    }

    #[test]
    fn gauge_gradient_uses_shaded_blocks() {
        let gauge = Gauge::new("power", 80.0).with_fill_style(GaugeFillStyle::Gradient);
        let ctx = RenderContext::new(Rect::new(0, 0, 10, 1));
        let mut frame = ScreenBuffer::new(10, 1);

        gauge.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame);
        // Should contain full blocks and at least one shaded block (▓ or ▒)
        assert!(snap.contains('█'), "should have full blocks");
        assert!(
            snap.contains('▓') || snap.contains('▒'),
            "gradient should have shaded blocks: {}",
            snap
        );
    }

    #[test]
    fn gauge_block_places_precision_marker() {
        let gauge = Gauge::new("charge", 50.0).with_fill_style(GaugeFillStyle::Block);
        let ctx = RenderContext::new(Rect::new(0, 0, 10, 1));
        let mut frame = ScreenBuffer::new(10, 1);

        gauge.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame);
        assert!(
            snap.contains('▌'),
            "block style should have a ▌ precision marker: {}",
            snap
        );
    }
}
