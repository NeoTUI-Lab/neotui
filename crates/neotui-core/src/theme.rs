// Theme tokens
// Typed token registry with safe fallback resolution for styling primitives

use std::collections::{BTreeMap, BTreeSet};

use crate::render::{BorderStyle, Color, Style};

pub const THEME_MINIMAL: &str = "minimal";
pub const THEME_DARK: &str = "dark";
pub const THEME_CYBERPUNK: &str = "cyberpunk";

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
            .with_style("divider.default", Style::default())
            .with_border("border.default", BorderStyle::default())
            .with_fallback("label.default", "text.default")
            .with_fallback("panel.title", "text.default")
            .with_fallback("panel.default", "screen.default")
            .with_fallback("button.default", "text.default")
            .with_fallback("list.default", "text.default")
            .with_fallback("panel.border.color", "color.border.default")
            .with_fallback("text.primary", "text.default")
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

    pub fn preset(name: &str) -> Option<Self> {
        match name {
            THEME_MINIMAL => Some(Self::minimal()),
            THEME_DARK => Some(Self::dark()),
            THEME_CYBERPUNK => Some(Self::cyberpunk()),
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
}
