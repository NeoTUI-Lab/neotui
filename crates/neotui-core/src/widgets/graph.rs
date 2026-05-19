use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::Style;

#[derive(Debug, Clone, PartialEq)]
pub struct Graph {
    id: ComponentId,
    values: Vec<f64>,
    title: Option<String>,
    style: Style,
}

impl Graph {
    pub fn new<I>(id: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        Self {
            id: ComponentId(id.into()),
            values: values.into_iter().collect(),
            title: None,
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

impl Component for Graph {
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

        let mut chart_origin_y = area.y;
        let mut chart_height = area.height;
        if let Some(title) = &self.title {
            let title = title
                .chars()
                .take(usize::from(area.width))
                .collect::<String>();
            let _ = frame.draw_text(area.x, area.y, &title, self.style.clone());
            chart_origin_y = chart_origin_y.saturating_add(1);
            chart_height = chart_height.saturating_sub(1);
        }
        if chart_height == 0 || self.values.is_empty() {
            return;
        }

        let max_value = self
            .values
            .iter()
            .copied()
            .fold(f64::MIN, f64::max)
            .max(0.0);
        if max_value <= 0.0 {
            return;
        }

        for (column, value) in self.values.iter().take(usize::from(area.width)).enumerate() {
            let Ok(column) = u16::try_from(column) else {
                break;
            };
            let normalized = (*value / max_value).clamp(0.0, 1.0);
            let bar_height = ((normalized * f64::from(chart_height)).round() as u16).max(1);
            for fill in 0..bar_height.min(chart_height) {
                let x = area.x.saturating_add(column);
                let y = chart_origin_y
                    .saturating_add(chart_height.saturating_sub(1))
                    .saturating_sub(fill);
                let _ = frame.draw_text(x, y, "#", self.style.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};

    #[test]
    fn graph_renders_title_and_bars() {
        let graph = Graph::new("latency", [1.0, 2.0, 3.0]).with_title("Latency");
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        graph.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('L'));
        assert_eq!(frame.get(0, 3).map(|cell| cell.symbol), Some('#'));
        assert_eq!(frame.get(2, 1).map(|cell| cell.symbol), Some('#'));
    }

    #[test]
    fn graph_applies_style() {
        let style = Style {
            fg: Color::Green,
            ..Style::default()
        };
        let graph = Graph::new("latency", [3.0]).with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 2, 2));
        let mut frame = ScreenBuffer::new(2, 2);

        graph.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 1).map(|cell| cell.style.clone()), Some(style));
    }
}
