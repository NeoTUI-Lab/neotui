// Panel widget
// Renders a bordered container and exposes its inner content area

use crate::component::{Component, ComponentNode, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::layout::{split_vertical, Constraint};
use crate::render::{panel_content_rect, BorderStyle, Padding, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelVariant {
    Plain,
    #[default]
    Framed,
    Data,
    Alert,
    Hero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelDensity {
    Compact,
    #[default]
    Normal,
    Spacious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelChrome {
    Minimal,
    #[default]
    Framed,
    Technical,
    Cinematic,
}

/// Controls how the panel title is rendered on the top border.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TitleStyle {
    /// Plain text in border (default): `+─ TITLE ──+`
    #[default]
    Plain,
    /// Chevron decorators: `◂ TITLE ▸`
    Chevron,
    /// Square bracket decorators: `[ TITLE ]`
    Bracket,
    /// Arrow prefix: `▶ TITLE`
    Arrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    id: ComponentId,
    title: Option<String>,
    title_style: TitleStyle,
    variant: PanelVariant,
    density: PanelDensity,
    chrome: PanelChrome,
    style: Style,
    surface_style: Option<Style>,
    padding: Padding,
    border: BorderStyle,
    border_style_name: Option<String>,
    grid: bool,
    controls: bool,
    grid_style: Option<Style>,
    footer_left: Option<String>,
    footer_right: Option<String>,
}

impl Panel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            title_style: TitleStyle::Plain,
            variant: PanelVariant::Framed,
            density: PanelDensity::Normal,
            chrome: PanelChrome::Framed,
            style: Style::default(),
            surface_style: None,
            padding: Padding::default(),
            border: BorderStyle::default(),
            border_style_name: None,
            grid: false,
            controls: false,
            grid_style: None,
            footer_left: None,
            footer_right: None,
        }
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn padding(&self) -> Padding {
        self.padding
    }

    pub fn border(&self) -> BorderStyle {
        self.border
    }

    pub fn border_style_name(&self) -> Option<&str> {
        self.border_style_name.as_deref()
    }

    pub fn grid(&self) -> bool {
        self.grid
    }

    pub fn controls(&self) -> bool {
        self.controls
    }

    pub fn grid_style(&self) -> Option<&Style> {
        self.grid_style.as_ref()
    }

    pub fn content_area(&self, area: Rect) -> Rect {
        let padding = self.effective_padding();
        if self.has_border() {
            panel_content_rect(area, padding)
        } else {
            apply_padding(area, padding)
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_title_style(mut self, style: TitleStyle) -> Self {
        self.title_style = style;
        self
    }

    pub fn with_variant(mut self, variant: PanelVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn with_density(mut self, density: PanelDensity) -> Self {
        self.density = density;
        self
    }

    pub fn with_chrome(mut self, chrome: PanelChrome) -> Self {
        self.chrome = chrome;
        self
    }

    pub fn with_footer_left(mut self, label: impl Into<String>) -> Self {
        self.footer_left = Some(label.into());
        self
    }

    pub fn with_footer_right(mut self, label: impl Into<String>) -> Self {
        self.footer_right = Some(label.into());
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_surface_style(mut self, style: Style) -> Self {
        self.surface_style = Some(style);
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn with_border_style_name(mut self, name: impl Into<String>) -> Self {
        self.border_style_name = Some(name.into());
        self
    }

    pub fn with_grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    pub fn with_controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }

    pub fn with_grid_style(mut self, style: Style) -> Self {
        self.grid_style = Some(style);
        self
    }

    fn has_border(&self) -> bool {
        !matches!(self.variant, PanelVariant::Plain) && !matches!(self.chrome, PanelChrome::Minimal)
    }

    fn effective_padding(&self) -> Padding {
        let density_padding = match self.density {
            PanelDensity::Compact | PanelDensity::Normal => Padding::zero(),
            PanelDensity::Spacious => Padding::symmetric(1, 0),
        };

        Padding {
            top: self.padding.top.max(density_padding.top),
            right: self.padding.right.max(density_padding.right),
            bottom: self.padding.bottom.max(density_padding.bottom),
            left: self.padding.left.max(density_padding.left),
        }
    }

    fn fill_surface(&self, area: Rect, frame: &mut Frame) {
        let Some(style) = self.surface_style.clone() else {
            return;
        };

        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                if let Some(cell) = frame.get(x, y) {
                    if cell.symbol == ' ' {
                        frame.set(
                            x,
                            y,
                            crate::render::Cell {
                                symbol: ' ',
                                style: style.clone(),
                            },
                        );
                    }
                }
            }
        }
    }
}

fn apply_padding(area: Rect, padding: Padding) -> Rect {
    let horizontal = padding.left.saturating_add(padding.right);
    let vertical = padding.top.saturating_add(padding.bottom);

    Rect::new(
        area.x.saturating_add(padding.left),
        area.y.saturating_add(padding.top),
        area.width.saturating_sub(horizontal),
        area.height.saturating_sub(vertical),
    )
}

impl Component for Panel {
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

        split_vertical(
            self.content_area(area.clone()),
            &vec![Constraint::Flex(1); children.len()],
        )
    }

    fn render(&self, ctx: &RenderContext, frame: &mut Frame) {
        let area = ctx.area().clone();

        // Build decorated title string based on title_style
        let decorated_title: Option<String> = self.title.as_ref().map(|t| match self.title_style {
            TitleStyle::Plain => t.clone(),
            TitleStyle::Chevron => format!("◂ {} ▸", t),
            TitleStyle::Bracket => format!("[ {} ]", t),
            TitleStyle::Arrow => format!("▶ {}", t),
        });

        if self.has_border() {
            frame.draw_panel(
                area.clone(),
                decorated_title.as_deref(),
                self.style.clone(),
                self.effective_padding(),
                self.border,
            );
        } else if let Some(title) = decorated_title.as_deref() {
            let _ = frame.draw_text(area.x, area.y, title, self.style.clone());
        }

        self.fill_surface(self.content_area(area.clone()), frame);

        if self.controls && self.has_border() && area.width >= 12 {
            let right = area.x.saturating_add(area.width).saturating_sub(1);
            let start_x = right.saturating_sub(10);
            frame.draw_text(start_x, area.y, "[ - ▢ X ]", self.style.clone());
        }

        // Corner metadata labels on the bottom border
        let bottom_y = area.y.saturating_add(area.height).saturating_sub(1);
        if self.has_border() && area.height >= 2 {
            if let Some(ref fl) = self.footer_left {
                let fl_x = area.x.saturating_add(1);
                let max_left = area.width.saturating_sub(2) as usize;
                let clipped: String = fl.chars().take(max_left).collect();
                let muted = Style {
                    fg: crate::render::Color::Indexed(8),
                    ..Style::default()
                };
                let _ = frame.draw_text(fl_x, bottom_y, &clipped, muted);
            }
            if let Some(ref fr) = self.footer_right {
                let fr_len = fr.chars().count() as u16;
                let fr_x = area
                    .x
                    .saturating_add(area.width)
                    .saturating_sub(1)
                    .saturating_sub(fr_len);
                if fr_x > area.x {
                    let muted = Style {
                        fg: crate::render::Color::Indexed(8),
                        ..Style::default()
                    };
                    let _ = frame.draw_text(fr_x, bottom_y, fr, muted);
                }
            }
        }

        if self.grid {
            let content_rect = self.content_area(area);
            let style = self.grid_style.clone().unwrap_or_else(|| Style::default());
            for y in content_rect.y..content_rect.y.saturating_add(content_rect.height) {
                for x in content_rect.x..content_rect.x.saturating_add(content_rect.width) {
                    if let Some(cell) = frame.get(x, y) {
                        if cell.symbol == ' ' {
                            frame.set(
                                x,
                                y,
                                crate::render::Cell {
                                    symbol: '·',
                                    style: style.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::render::{Color, ScreenBuffer};
    use crate::testing::snapshot_buffer;

    #[test]
    fn panel_renders_border_and_title() {
        let panel = Panel::new("stats").with_title("Stats");
        let ctx = RenderContext::new(Rect::new(0, 0, 12, 5));
        let mut frame = ScreenBuffer::new(12, 5);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('+'));
        assert_eq!(frame.get(11, 4).map(|cell| cell.symbol), Some('+'));
        assert_eq!(frame.get(2, 0).map(|cell| cell.symbol), Some('S'));
        assert_eq!(frame.get(6, 0).map(|cell| cell.symbol), Some('s'));
    }

    #[test]
    fn panel_content_area_respects_padding() {
        let panel = Panel::new("stats").with_padding(Padding::uniform(1));

        let content = panel.content_area(Rect::new(0, 0, 10, 6));

        assert_eq!(content, Rect::new(2, 2, 6, 2));
    }

    #[test]
    fn panel_applies_style_to_border_cells() {
        let style = Style {
            fg: Color::Cyan,
            bold: true,
            ..Style::default()
        };
        let panel = Panel::new("stats").with_style(style.clone());
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.style.clone()), Some(style));
    }

    #[test]
    fn panel_supports_custom_border_style() {
        let border = BorderStyle {
            top_left: '#',
            top_right: '#',
            bottom_left: '#',
            bottom_right: '#',
            horizontal: '=',
            vertical: '!',
        };
        let panel = Panel::new("stats").with_border(border);
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        panel.render(&ctx, &mut frame);

        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('#'));
        assert_eq!(frame.get(1, 0).map(|cell| cell.symbol), Some('='));
        assert_eq!(frame.get(0, 1).map(|cell| cell.symbol), Some('!'));
    }

    #[test]
    fn plain_panel_uses_full_area_without_border() {
        let panel = Panel::new("plain").with_variant(PanelVariant::Plain);
        let ctx = RenderContext::new(Rect::new(0, 0, 8, 3));
        let mut frame = ScreenBuffer::new(8, 3);

        panel.render(&ctx, &mut frame);

        assert_eq!(
            panel.content_area(Rect::new(0, 0, 8, 3)),
            Rect::new(0, 0, 8, 3)
        );
        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some(' '));
    }

    #[test]
    fn spacious_panel_adds_horizontal_breathing_room() {
        let panel = Panel::new("space").with_density(PanelDensity::Spacious);

        assert_eq!(
            panel.content_area(Rect::new(0, 0, 12, 5)),
            Rect::new(2, 1, 8, 3)
        );
    }

    #[test]
    fn panel_layout_uses_component_id_and_area() {
        let panel = Panel::new("container");
        let area = Rect::new(1, 2, 10, 4);

        let node = panel.layout(&LayoutContext, area.clone());

        assert_eq!(node.component_id, ComponentId("container".into()));
        assert_eq!(node.area, area);
    }

    #[test]
    fn panel_distributes_children_inside_content_area() {
        let panel = Panel::new("container");

        let children = vec![
            ComponentNode::new(Box::new(Panel::new("a"))),
            ComponentNode::new(Box::new(Panel::new("b"))),
            ComponentNode::new(Box::new(Panel::new("c"))),
        ];
        let areas = panel.child_layout_areas(&Rect::new(0, 0, 12, 6), &children);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(1, 1, 10, 1));
        assert_eq!(areas[1], Rect::new(1, 2, 10, 1));
        assert_eq!(areas[2], Rect::new(1, 3, 10, 2));
    }

    #[test]
    fn panel_distributes_children_inside_padded_content_area() {
        let panel = Panel::new("container").with_padding(Padding::uniform(1));

        let children = vec![
            ComponentNode::new(Box::new(Panel::new("top"))),
            ComponentNode::new(Box::new(Panel::new("bottom"))),
        ];
        let areas = panel.child_layout_areas(&Rect::new(0, 0, 14, 8), &children);

        assert_eq!(areas, vec![Rect::new(2, 2, 10, 2), Rect::new(2, 4, 10, 2)]);
    }

    #[test]
    fn panel_renders_grid_dots() {
        let panel = Panel::new("p").with_grid(true);
        let ctx = RenderContext::new(Rect::new(0, 0, 6, 4));
        let mut frame = ScreenBuffer::new(6, 4);

        panel.render(&ctx, &mut frame);

        // Content area is x: 1..5, y: 1..3
        assert_eq!(frame.get(1, 1).map(|cell| cell.symbol), Some('·'));
        assert_eq!(frame.get(4, 2).map(|cell| cell.symbol), Some('·'));
        // Border should not be grid dots
        assert_eq!(frame.get(0, 0).map(|cell| cell.symbol), Some('+'));
    }

    #[test]
    fn panel_renders_window_controls() {
        let panel = Panel::new("p").with_controls(true);
        let ctx = RenderContext::new(Rect::new(0, 0, 15, 4));
        let mut frame = ScreenBuffer::new(15, 4);

        panel.render(&ctx, &mut frame);

        let mut controls_str = String::new();
        for x in 4..=12 {
            if let Some(cell) = frame.get(x, 0) {
                controls_str.push(cell.symbol);
            }
        }
        assert_eq!(controls_str, "[ - ▢ X ]");
    }

    #[test]
    fn panel_snapshot_stays_stable() {
        let panel = Panel::new("stats").with_title("Stats");
        let ctx = RenderContext::new(Rect::new(0, 0, 12, 4));
        let mut frame = ScreenBuffer::new(12, 4);

        panel.render(&ctx, &mut frame);

        assert_eq!(
            snapshot_buffer(&frame),
            concat!(
                "+·Stats·---+\n",
                "|··········|\n",
                "|··········|\n",
                "+----------+"
            )
        );
    }
}
