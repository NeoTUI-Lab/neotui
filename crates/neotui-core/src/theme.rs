// Theme tokens
// Typed token registry with safe fallback resolution for styling primitives

use std::collections::{BTreeMap, BTreeSet};

use crate::render::{BorderStyle, Color, Style};

pub const THEME_MINIMAL: &str = "minimal";
pub const THEME_DARK: &str = "dark";
pub const THEME_CYBERPUNK: &str = "cyberpunk";
pub const THEME_REDLINE: &str = "redline";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeTokenValue {
    Color(Color),
    Style(Style),
    Border(BorderStyle),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Theme {
    name: Option<String>,
    tokens: BTreeMap<String, ThemeTokenValue>,
    fallbacks: BTreeMap<String, String>,
}

impl Theme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::default()
        }
    }

    pub fn baseline() -> Self {
        Self::named("baseline")
            .with_color("color.text.default", Color::White)
            .with_color("color.text.muted", Color::Indexed(8))
            .with_color("color.border.default", Color::White)
            .with_color("color.background.default", Color::Reset)
            .with_style(
                "text.default",
                Style {
                    fg: Color::White,
                    ..Style::default()
                },
            )
            .with_style(
                "text.muted",
                Style {
                    fg: Color::Indexed(8),
                    ..Style::default()
                },
            )
            .with_style(
                "screen.default",
                Style {
                    fg: Color::White,
                    bg: Color::Reset,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border",
                Style {
                    fg: Color::White,
                    ..Style::default()
                },
            )
            .with_style(
                "button.focused",
                Style {
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "list.selected",
                Style {
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style("divider.default", Style::default())
            .with_border("border.default", BorderStyle::default())
            .with_fallback("label.default", "text.default")
            .with_fallback("panel.title", "text.default")
            .with_fallback("panel.default", "screen.default")
            .with_fallback("panel.surface", "panel.default")
            .with_fallback("panel.surface.plain", "screen.default")
            .with_fallback("panel.surface.data", "panel.surface")
            .with_fallback("panel.surface.alert", "panel.surface")
            .with_fallback("panel.surface.warning", "panel.surface")
            .with_fallback("panel.surface.success", "panel.surface")
            .with_fallback("panel.surface.hero", "panel.surface")
            .with_fallback("button.default", "text.default")
            .with_fallback("list.default", "text.default")
            .with_fallback("text_block.default", "text.default")
            .with_fallback("graph.default", "text.default")
            .with_fallback("table.header", "text.primary")
            .with_fallback("table.row", "text.default")
            .with_fallback("table.selected", "list.selected")
            .with_fallback("panel.border.color", "color.border.default")
            .with_fallback("text.primary", "text.default")
            .with_fallback("surface.base", "screen.default")
            .with_fallback("surface.panel", "panel.surface")
            .with_fallback("surface.raised", "surface.panel")
            .with_fallback("surface.recessed", "surface.panel")
            .with_fallback("border.subtle", "panel.border")
            .with_fallback("border.strong", "panel.border")
            .with_fallback("border.alert", "panel.border.danger")
            .with_fallback("accent.primary", "text.primary")
            .with_fallback("accent.warning", "status.warning")
            .with_fallback("accent.danger", "status.critical")
            .with_fallback("accent.success", "status.normal")
            .with_fallback("data.track", "gauge.track")
            .with_fallback("data.fill", "gauge.filled")
            .with_fallback("data.glow", "sparkline.default")
            // New rich widgets from Epic 17
            .with_fallback("metric.default", "text.default")
            .with_fallback("gauge.track", "text.muted")
            .with_fallback("gauge.filled", "text.default")
            .with_fallback("sparkline.default", "text.default")
            .with_fallback("key_value_row.default", "text.default")
            .with_fallback("status_strip.default", "text.default")
            .with_fallback("knob.default", "text.default")
            .with_fallback("grid.dot", "text.muted")
            // Button and Panel variants
            .with_fallback("button.danger", "button.default")
            .with_fallback("button.warning", "button.default")
            .with_fallback("button.success", "button.default")
            .with_fallback("button.info", "button.default")
            .with_fallback("button.danger.focused", "button.focused")
            .with_fallback("button.warning.focused", "button.focused")
            .with_fallback("button.success.focused", "button.focused")
            .with_fallback("button.info.focused", "button.focused")
            .with_fallback("panel.border.danger", "panel.border")
            .with_fallback("panel.border.subtle", "border.subtle")
            .with_fallback("panel.border.data", "panel.border.info")
            .with_fallback("panel.border.alert", "panel.border.danger")
            .with_fallback("panel.border.hero", "border.strong")
            .with_fallback("panel.border.warning", "panel.border")
            .with_fallback("panel.border.success", "panel.border")
            .with_fallback("panel.border.info", "panel.border")
            // Generic status styles
            .with_style(
                "status.normal",
                Style {
                    fg: Color::Green,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.warning",
                Style {
                    fg: Color::Yellow,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.critical",
                Style {
                    fg: Color::Red,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.info",
                Style {
                    fg: Color::Cyan,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.normal.tag",
                Style {
                    fg: Color::White,
                    bg: Color::Green,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.warning.tag",
                Style {
                    fg: Color::Black,
                    bg: Color::Yellow,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.critical.tag",
                Style {
                    fg: Color::White,
                    bg: Color::Red,
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.info.tag",
                Style {
                    fg: Color::Black,
                    bg: Color::Cyan,
                    bold: true,
                    ..Style::default()
                },
            )
    }

    pub fn minimal() -> Self {
        Self::baseline()
            .named_like(THEME_MINIMAL)
            .with_color("color.text.default", Color::White)
            .with_color("color.text.muted", Color::Indexed(8))
            .with_color("color.border.default", Color::White)
            .with_style(
                "screen.default",
                Style {
                    fg: Color::White,
                    bg: Color::Reset,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border",
                Style {
                    fg: Color::White,
                    ..Style::default()
                },
            )
            .with_style(
                "divider.default",
                Style {
                    fg: Color::Indexed(8),
                    ..Style::default()
                },
            )
    }

    pub fn dark() -> Self {
        Self::baseline()
            .named_like(THEME_DARK)
            .with_color(
                "color.text.default",
                Color::Rgb {
                    r: 230,
                    g: 232,
                    b: 235,
                },
            )
            .with_color(
                "color.text.muted",
                Color::Rgb {
                    r: 148,
                    g: 163,
                    b: 184,
                },
            )
            .with_color(
                "color.border.default",
                Color::Rgb {
                    r: 100,
                    g: 116,
                    b: 139,
                },
            )
            .with_color(
                "color.background.default",
                Color::Rgb {
                    r: 15,
                    g: 23,
                    b: 42,
                },
            )
            .with_style(
                "text.default",
                Style {
                    fg: Color::Rgb {
                        r: 230,
                        g: 232,
                        b: 235,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "text.muted",
                Style {
                    fg: Color::Rgb {
                        r: 148,
                        g: 163,
                        b: 184,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "screen.default",
                Style {
                    fg: Color::Rgb {
                        r: 230,
                        g: 232,
                        b: 235,
                    },
                    bg: Color::Rgb {
                        r: 15,
                        g: 23,
                        b: 42,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border",
                Style {
                    fg: Color::Rgb {
                        r: 100,
                        g: 116,
                        b: 139,
                    },
                    bg: Color::Rgb {
                        r: 15,
                        g: 23,
                        b: 42,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "divider.default",
                Style {
                    fg: Color::Rgb {
                        r: 71,
                        g: 85,
                        b: 105,
                    },
                    bg: Color::Rgb {
                        r: 15,
                        g: 23,
                        b: 42,
                    },
                    ..Style::default()
                },
            )
    }

    pub fn cyberpunk() -> Self {
        Self::baseline()
            .named_like(THEME_CYBERPUNK)
            .with_color(
                "color.text.default",
                Color::Rgb {
                    r: 110,
                    g: 255,
                    b: 214,
                },
            )
            .with_color(
                "color.text.muted",
                Color::Rgb {
                    r: 255,
                    g: 102,
                    b: 196,
                },
            )
            .with_color(
                "color.border.default",
                Color::Rgb {
                    r: 255,
                    g: 230,
                    b: 92,
                },
            )
            .with_color(
                "color.background.default",
                Color::Rgb { r: 10, g: 6, b: 26 },
            )
            .with_style(
                "text.default",
                Style {
                    fg: Color::Rgb {
                        r: 110,
                        g: 255,
                        b: 214,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "text.muted",
                Style {
                    fg: Color::Rgb {
                        r: 255,
                        g: 102,
                        b: 196,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "screen.default",
                Style {
                    fg: Color::Rgb {
                        r: 110,
                        g: 255,
                        b: 214,
                    },
                    bg: Color::Rgb { r: 10, g: 6, b: 26 },
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border",
                Style {
                    fg: Color::Rgb {
                        r: 255,
                        g: 230,
                        b: 92,
                    },
                    bg: Color::Rgb { r: 10, g: 6, b: 26 },
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "divider.default",
                Style {
                    fg: Color::Rgb {
                        r: 255,
                        g: 102,
                        b: 196,
                    },
                    bg: Color::Rgb { r: 10, g: 6, b: 26 },
                    ..Style::default()
                },
            )
    }

    pub fn redline() -> Self {
        let background = Color::Rgb { r: 5, g: 9, b: 14 };
        let surface = Color::Rgb {
            r: 11,
            g: 18,
            b: 26,
        };
        let text = Color::Rgb {
            r: 224,
            g: 232,
            b: 238,
        };
        let muted = Color::Rgb {
            r: 111,
            g: 131,
            b: 146,
        };
        let accent = Color::Rgb {
            r: 255,
            g: 66,
            b: 74,
        };
        let danger = Color::Rgb {
            r: 255,
            g: 35,
            b: 48,
        };
        let warning = Color::Rgb {
            r: 255,
            g: 117,
            b: 74,
        };
        let cyan = Color::Rgb {
            r: 83,
            g: 209,
            b: 224,
        };

        let success = Color::Rgb {
            r: 46,
            g: 204,
            b: 113,
        };
        let surface_raised = Color::Rgb {
            r: 18,
            g: 28,
            b: 38,
        };
        let surface_recessed = Color::Rgb { r: 7, g: 12, b: 18 };
        let border_subtle = Color::Rgb {
            r: 58,
            g: 74,
            b: 86,
        };

        Self::baseline()
            .named_like(THEME_REDLINE)
            .with_color("color.text.default", text.clone())
            .with_color("color.text.muted", muted.clone())
            .with_color("color.border.default", accent.clone())
            .with_color("color.background.default", background.clone())
            .with_color("color.surface.default", surface.clone())
            .with_color("color.surface.raised", surface_raised.clone())
            .with_color("color.surface.recessed", surface_recessed.clone())
            .with_color("color.accent.default", accent.clone())
            .with_color("color.danger.default", danger.clone())
            .with_color("color.warning.default", warning.clone())
            .with_color("color.accent.secondary", cyan.clone())
            .with_color("color.success.default", success.clone())
            .with_style(
                "screen.default",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "surface.base",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "surface.panel",
                Style {
                    fg: text.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "surface.raised",
                Style {
                    fg: text.clone(),
                    bg: surface_raised.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "surface.recessed",
                Style {
                    fg: text.clone(),
                    bg: surface_recessed.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "text.default",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "text.primary",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "text.muted",
                Style {
                    fg: muted,
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.default",
                Style {
                    fg: text.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.surface",
                Style {
                    fg: text.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.surface.plain",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.surface.data",
                Style {
                    fg: text.clone(),
                    bg: surface_recessed.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.surface.alert",
                Style {
                    fg: text.clone(),
                    bg: Color::Rgb { r: 28, g: 7, b: 12 },
                    ..Style::default()
                },
            )
            .with_style(
                "panel.surface.hero",
                Style {
                    fg: text.clone(),
                    bg: Color::Rgb { r: 16, g: 6, b: 24 },
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border",
                Style {
                    fg: accent.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "border.subtle",
                Style {
                    fg: border_subtle.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "border.strong",
                Style {
                    fg: accent.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "border.alert",
                Style {
                    fg: danger.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.subtle",
                Style {
                    fg: border_subtle.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.data",
                Style {
                    fg: cyan.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.alert",
                Style {
                    fg: danger.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.hero",
                Style {
                    fg: accent.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.title",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "divider.default",
                Style {
                    fg: accent.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.default",
                Style {
                    fg: text.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.focused",
                Style {
                    fg: Color::White,
                    bg: danger.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "list.default",
                Style {
                    fg: text.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "list.selected",
                Style {
                    fg: Color::White,
                    bg: Color::Rgb {
                        r: 86,
                        g: 16,
                        b: 24,
                    },
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "graph.default",
                Style {
                    fg: warning.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "accent.primary",
                Style {
                    fg: accent.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "accent.warning",
                Style {
                    fg: warning.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "accent.danger",
                Style {
                    fg: danger.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "accent.success",
                Style {
                    fg: success.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_fallback("label.default", "text.primary")
            .with_fallback("text_block.default", "text.default")
            .with_fallback("table.header", "panel.title")
            .with_fallback("table.row", "text.default")
            .with_fallback("table.selected", "list.selected")
            // Rich widgets overrides
            .with_style(
                "gauge.track",
                Style {
                    fg: Color::Rgb {
                        r: 20,
                        g: 30,
                        b: 40,
                    },
                    ..Style::default()
                },
            )
            .with_style(
                "gauge.filled",
                Style {
                    fg: accent.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_fallback("data.track", "gauge.track")
            .with_fallback("data.fill", "gauge.filled")
            .with_fallback("data.glow", "sparkline.default")
            .with_style(
                "sparkline.default",
                Style {
                    fg: cyan.clone(),
                    bg: background.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "status_strip.default",
                Style {
                    fg: text.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            // Button variant overrides
            .with_style(
                "button.danger",
                Style {
                    fg: danger.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.danger.focused",
                Style {
                    fg: Color::White,
                    bg: danger.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "button.warning",
                Style {
                    fg: warning.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.warning.focused",
                Style {
                    fg: Color::Black,
                    bg: warning.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "button.success",
                Style {
                    fg: success.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.success.focused",
                Style {
                    fg: Color::White,
                    bg: success.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "button.info",
                Style {
                    fg: cyan.clone(),
                    bg: surface.clone(),
                    ..Style::default()
                },
            )
            .with_style(
                "button.info.focused",
                Style {
                    fg: Color::White,
                    bg: cyan.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            // Panel variant overrides
            .with_style(
                "panel.border.danger",
                Style {
                    fg: danger.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.warning",
                Style {
                    fg: warning.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.success",
                Style {
                    fg: success.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "panel.border.info",
                Style {
                    fg: cyan.clone(),
                    bg: background.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            // Status overrides
            .with_style(
                "status.normal",
                Style {
                    fg: success.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.warning",
                Style {
                    fg: warning.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.critical",
                Style {
                    fg: danger.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.info",
                Style {
                    fg: cyan.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.normal.tag",
                Style {
                    fg: Color::White,
                    bg: success.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.warning.tag",
                Style {
                    fg: Color::Black,
                    bg: warning.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.critical.tag",
                Style {
                    fg: Color::White,
                    bg: danger.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
            .with_style(
                "status.info.tag",
                Style {
                    fg: Color::Black,
                    bg: cyan.clone(),
                    bold: true,
                    ..Style::default()
                },
            )
    }

    pub fn preset(name: &str) -> Option<Self> {
        match name {
            THEME_MINIMAL => Some(Self::minimal()),
            THEME_DARK => Some(Self::dark()),
            THEME_CYBERPUNK => Some(Self::cyberpunk()),
            THEME_REDLINE => Some(Self::redline()),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn token(&self, token: &str) -> Option<&ThemeTokenValue> {
        self.tokens.get(token)
    }

    pub fn fallback(&self, token: &str) -> Option<&str> {
        self.fallbacks.get(token).map(String::as_str)
    }

    pub fn set_token(&mut self, token: impl Into<String>, value: ThemeTokenValue) {
        self.tokens.insert(token.into(), value);
    }

    pub fn set_fallback(&mut self, token: impl Into<String>, fallback: impl Into<String>) {
        self.fallbacks.insert(token.into(), fallback.into());
    }

    pub fn with_token(mut self, token: impl Into<String>, value: ThemeTokenValue) -> Self {
        self.set_token(token, value);
        self
    }

    pub fn with_color(mut self, token: impl Into<String>, color: Color) -> Self {
        self.set_token(token, ThemeTokenValue::Color(color));
        self
    }

    pub fn with_style(mut self, token: impl Into<String>, style: Style) -> Self {
        self.set_token(token, ThemeTokenValue::Style(style));
        self
    }

    pub fn with_border(mut self, token: impl Into<String>, border: BorderStyle) -> Self {
        self.set_token(token, ThemeTokenValue::Border(border));
        self
    }

    pub fn with_fallback(mut self, token: impl Into<String>, fallback: impl Into<String>) -> Self {
        self.set_fallback(token, fallback);
        self
    }

    fn named_like(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn resolve_color(&self, token: &str) -> Color {
        match self.resolve_token(token) {
            Some(ThemeTokenValue::Color(color)) => color.clone(),
            _ => Color::Reset,
        }
    }

    pub fn resolve_style(&self, token: &str) -> Style {
        match self.resolve_token(token) {
            Some(ThemeTokenValue::Style(style)) => style.clone(),
            _ => Style::default(),
        }
    }

    pub fn resolve_border(&self, token: &str) -> BorderStyle {
        match self.resolve_token(token) {
            Some(ThemeTokenValue::Border(border)) => *border,
            _ => BorderStyle::default(),
        }
    }

    fn resolve_token(&self, token: &str) -> Option<&ThemeTokenValue> {
        let mut current = token;
        let mut visited = BTreeSet::new();

        loop {
            if !visited.insert(current) {
                return None;
            }

            if let Some(value) = self.tokens.get(current) {
                return Some(value);
            }

            current = self.fallbacks.get(current)?.as_str();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_style_token() {
        let style = Style {
            fg: Color::Cyan,
            bold: true,
            ..Style::default()
        };
        let theme = Theme::new().with_style("button.focused", style.clone());

        assert_eq!(theme.resolve_style("button.focused"), style);
    }

    #[test]
    fn resolves_token_through_fallback_chain() {
        let style = Style {
            fg: Color::Yellow,
            ..Style::default()
        };
        let theme = Theme::new()
            .with_style("text.default", style.clone())
            .with_fallback("label.default", "text.default")
            .with_fallback("button.label", "label.default");

        assert_eq!(theme.resolve_style("button.label"), style);
    }

    #[test]
    fn missing_style_token_falls_back_to_safe_default() {
        let theme = Theme::new();

        assert_eq!(theme.resolve_style("missing.token"), Style::default());
    }

    #[test]
    fn missing_color_token_falls_back_to_reset() {
        let theme = Theme::new();

        assert_eq!(theme.resolve_color("missing.token"), Color::Reset);
    }

    #[test]
    fn missing_border_token_falls_back_to_default_border() {
        let theme = Theme::new();

        assert_eq!(
            theme.resolve_border("missing.token"),
            BorderStyle::default()
        );
    }

    #[test]
    fn type_mismatch_still_returns_safe_default() {
        let theme = Theme::new().with_color("panel.border", Color::Green);

        assert_eq!(theme.resolve_style("panel.border"), Style::default());
    }

    #[test]
    fn cyclic_fallbacks_do_not_loop_forever() {
        let theme = Theme::new().with_fallback("a", "b").with_fallback("b", "a");

        assert_eq!(theme.resolve_style("a"), Style::default());
    }

    #[test]
    fn baseline_theme_exposes_expected_defaults() {
        let theme = Theme::baseline();

        assert_eq!(theme.name(), Some("baseline"));
        assert_eq!(
            theme.resolve_style("text.primary"),
            Style {
                fg: Color::White,
                ..Style::default()
            }
        );
        assert_eq!(
            theme.resolve_border("border.default"),
            BorderStyle::default()
        );
        assert_eq!(theme.resolve_color("panel.border.color"), Color::White);
    }

    #[test]
    fn preset_resolves_known_theme_names() {
        assert_eq!(
            Theme::preset(THEME_MINIMAL).and_then(|theme| theme.name().map(str::to_string)),
            Some(THEME_MINIMAL.to_string())
        );
        assert_eq!(
            Theme::preset(THEME_DARK).and_then(|theme| theme.name().map(str::to_string)),
            Some(THEME_DARK.to_string())
        );
        assert_eq!(
            Theme::preset(THEME_CYBERPUNK).and_then(|theme| theme.name().map(str::to_string)),
            Some(THEME_CYBERPUNK.to_string())
        );
        assert_eq!(
            Theme::preset(THEME_REDLINE).and_then(|theme| theme.name().map(str::to_string)),
            Some(THEME_REDLINE.to_string())
        );
        assert!(Theme::preset("unknown").is_none());
    }

    #[test]
    fn minimal_theme_keeps_neutral_defaults() {
        let theme = Theme::minimal();

        assert_eq!(theme.resolve_style("screen.default").bg, Color::Reset);
        assert_eq!(theme.resolve_style("panel.border").fg, Color::White);
    }

    #[test]
    fn dark_theme_uses_dark_background_and_soft_foreground() {
        let theme = Theme::dark();

        assert_eq!(
            theme.resolve_style("screen.default").bg,
            Color::Rgb {
                r: 15,
                g: 23,
                b: 42,
            }
        );
        assert_eq!(
            theme.resolve_style("text.default").fg,
            Color::Rgb {
                r: 230,
                g: 232,
                b: 235,
            }
        );
    }

    #[test]
    fn cyberpunk_theme_has_high_contrast_accented_border() {
        let theme = Theme::cyberpunk();
        let border = theme.resolve_style("panel.border");

        assert_eq!(
            border.fg,
            Color::Rgb {
                r: 255,
                g: 230,
                b: 92,
            }
        );
        assert!(border.bold);
        assert_eq!(
            theme.resolve_style("text.muted").fg,
            Color::Rgb {
                r: 255,
                g: 102,
                b: 196,
            }
        );
    }

    #[test]
    fn redline_theme_exposes_cinematic_control_panel_tokens() {
        let theme = Theme::redline();

        assert_eq!(theme.name(), Some(THEME_REDLINE));
        assert_eq!(
            theme.resolve_style("screen.default").bg,
            Color::Rgb { r: 5, g: 9, b: 14 }
        );
        assert_eq!(
            theme.resolve_style("panel.border").fg,
            Color::Rgb {
                r: 255,
                g: 66,
                b: 74,
            }
        );
        assert!(theme.resolve_style("button.focused").bold);
        assert_eq!(
            theme.resolve_style("table.header"),
            theme.resolve_style("panel.title")
        );
    }

    #[test]
    fn redline_theme_exposes_rich_widgets_and_variants() {
        let theme = Theme::redline();

        assert_eq!(
            theme.resolve_style("gauge.filled").fg,
            Color::Rgb {
                r: 255,
                g: 66,
                b: 74
            }
        );
        assert_eq!(
            theme.resolve_style("sparkline.default").fg,
            Color::Rgb {
                r: 83,
                g: 209,
                b: 224
            }
        );
        assert_eq!(
            theme.resolve_style("button.danger").fg,
            Color::Rgb {
                r: 255,
                g: 35,
                b: 48
            }
        );
        assert_eq!(
            theme.resolve_style("panel.border.warning").fg,
            Color::Rgb {
                r: 255,
                g: 117,
                b: 74
            }
        );
        assert_eq!(
            theme.resolve_style("status.normal").fg,
            Color::Rgb {
                r: 46,
                g: 204,
                b: 113
            }
        );
    }

    #[test]
    fn redline_theme_exposes_visual_system_tokens() {
        let theme = Theme::redline();

        assert_eq!(
            theme.resolve_style("surface.raised").bg,
            Color::Rgb {
                r: 18,
                g: 28,
                b: 38
            }
        );
        assert_eq!(
            theme.resolve_style("panel.border.data").fg,
            Color::Rgb {
                r: 83,
                g: 209,
                b: 224
            }
        );
        assert_eq!(
            theme.resolve_style("panel.surface.hero").bg,
            Color::Rgb { r: 16, g: 6, b: 24 }
        );
        assert_eq!(
            theme.resolve_style("data.fill"),
            theme.resolve_style("gauge.filled")
        );
    }
}
