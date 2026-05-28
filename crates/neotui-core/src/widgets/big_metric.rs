// BigMetric widget
// Renders a numeric/alphanumeric value in large block-character text using Unicode half-blocks.
//
// Three native font sizes are provided, each with hand-designed bitmaps at that resolution:
//
//   compact : 3 cols × 2 rows per glyph (6 half-pixels)  — secondary metrics
//   large   : 5 cols × 3 rows per glyph (15 half-pixels) — primary display / queue numbers
//   hero    : 7 cols × 4 rows per glyph (28 half-pixels) — full-screen counters
//
// Each font has its own glyph table; larger fonts are NOT scaled-up versions of smaller ones.
// This ensures legibility at every size.

use crate::component::{Component, Frame, LayoutContext, LayoutNode, RenderContext};
use crate::event::ComponentId;
use crate::layout::Rect;
use crate::render::{Color, Style, TextAlign};

// ═══════════════════════════════════════════════════════════════════════════════
//  Font: compact  — 3 columns × 2 rows  (original NeoTUI block font)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns a 3-wide × 2-row glyph.  Each row is a 3-char string.
fn compact_glyph(ch: char) -> Option<[&'static str; 2]> {
    match ch {
        '0' => Some(["▄█▄", "▀█▀"]),
        '1' => Some([" ▐█", " ▐█"]),
        '2' => Some(["▀▀█", "█▄▄"]),
        '3' => Some(["▄▀█", "▀▀█"]),
        '4' => Some(["█▄█", "▀▀█"]),
        '5' => Some(["█▄▄", "▀▀█"]),
        '6' => Some(["█▄▄", "█▀█"]),
        '7' => Some(["▀▀█", " ▐█"]),
        '8' => Some(["▄█▄", "▐█▌"]),
        '9' => Some(["▄█▄", "▀▀█"]),
        ':' => Some([" ▪ ", " ▪ "]),
        '-' => Some(["   ", "▀▀▀"]),
        '.' => Some(["   ", " ▄ "]),
        ' ' => Some(["   ", "   "]),
        'A' | 'a' => Some(["▄█▄", "█▀█"]),
        'B' | 'b' => Some(["▐█▄", "▐█▀"]),
        'C' | 'c' => Some(["▄█ ", "▀█ "]),
        'D' | 'd' => Some(["▐▄▄", "▐▀▀"]),
        'E' | 'e' => Some(["▀██", "█▄▄"]),
        'F' | 'f' => Some(["▀██", "▐  "]),
        'G' | 'g' => Some(["▄█ ", "▀▄█"]),
        'H' | 'h' => Some(["█ █", "█▀█"]),
        'I' | 'i' => Some(["▀█▀", "▄█▄"]),
        'J' | 'j' => Some(["  █", "▀▄█"]),
        'K' | 'k' => Some(["█▄▐", "█▀▐"]),
        'L' | 'l' => Some(["▐  ", "▐▄▄"]),
        'M' | 'm' => Some(["███", "█ █"]),
        'N' | 'n' => Some(["█▄█", "█▀█"]),
        'O' | 'o' => Some(["▄█▄", "▀█▀"]),
        'P' | 'p' => Some(["▐█▄", "▐  "]),
        'Q' | 'q' => Some(["▄█▄", "▀█▐"]),
        'R' | 'r' => Some(["▐█▄", "▐▌▐"]),
        'S' | 's' => Some(["█▄▄", "▀▀█"]),
        'T' | 't' => Some(["▀█▀", " █ "]),
        'U' | 'u' => Some(["█ █", "▀█▀"]),
        'V' | 'v' => Some(["▌ ▐", " ▀ "]),
        'W' | 'w' => Some(["█ █", "█▄█"]),
        'X' | 'x' => Some(["▌▄▐", "▌▀▐"]),
        'Y' | 'y' => Some(["▌ ▐", " █ "]),
        'Z' | 'z' => Some(["▀▀█", "█▄▄"]),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Font: large  — 5 columns × 3 rows   (hand-designed for legibility)
//
//  Each glyph cell is one terminal character.  Because half-blocks give two
//  vertical sub-pixels per cell, a 5×3 grid yields 5 × 6 = 30 half-pixels —
//  enough for clearly-readable uppercase letters and digits.
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns a 5-wide × 3-row glyph.  Each row is a 5-char string.
fn large_glyph(ch: char) -> Option<[&'static str; 3]> {
    match ch {
        //       row0       row1       row2
        '0' => Some(["▄███▄", "█   █", "▀███▀"]),
        '1' => Some(["▄▄█  ", "  █  ", "▄▄█▄▄"]),
        '2' => Some(["▀███▄", " ▄▀▀ ", "█▄▄▄▄"]),
        '3' => Some(["▀███▄", " ▀▀█ ", "▄███▀"]),
        '4' => Some(["█  █ ", "█▄▄█▄", "   █ "]),
        '5' => Some(["█▄▄▄ ", "▀▀▀█ ", "▄▄▄█▀"]),
        '6' => Some(["▄███ ", "█▄▄▄ ", "▀███▀"]),
        '7' => Some(["▀▀▀▀█", "  ▄▀ ", " █   "]),
        '8' => Some(["▄███▄", "▐███▌", "▀███▀"]),
        '9' => Some(["▄███▄", "▀▀▀█ ", " ███▀"]),
        ':' => Some(["     ", "  ●  ", "  ●  "]),
        '-' => Some(["     ", " ▀▀▀ ", "     "]),
        '.' => Some(["     ", "     ", "  ●  "]),
        ' ' => Some(["     ", "     ", "     "]),
        'A' | 'a' => Some([" ▄█▄ ", "█▀▀▀█", "█   █"]),
        'B' | 'b' => Some(["█▀▀▀█", "█▄▄▄█", "█▄▄▄█"]),
        'C' | 'c' => Some(["▄████", "█    ", "▀████"]),
        'D' | 'd' => Some(["█▀▀▀█", "█   █", "█▄▄▄█"]),
        'E' | 'e' => Some(["█▀▀▀▀", "█▄▄  ", "█▄▄▄▄"]),
        'F' | 'f' => Some(["█▀▀▀▀", "█▄▄  ", "█    "]),
        'G' | 'g' => Some(["▄████", "█  ▄█", "▀████"]),
        'H' | 'h' => Some(["█   █", "█▄▄▄█", "█   █"]),
        'I' | 'i' => Some(["▄▄█▄▄", "  █  ", "▄▄█▄▄"]),
        'J' | 'j' => Some(["    █", "    █", "▀██▀ "]),
        'K' | 'k' => Some(["█  ▄▀", "█▀▀  ", "█  ▀▄"]),
        'L' | 'l' => Some(["█    ", "█    ", "█▄▄▄▄"]),
        'M' | 'm' => Some(["█▄▄▄█", "█ █ █", "█   █"]),
        'N' | 'n' => Some(["█▄  █", "█ █ █", "█  ▀█"]),
        'O' | 'o' => Some(["▄███▄", "█   █", "▀███▀"]),
        'P' | 'p' => Some(["█▀▀▀█", "█▄▄▄█", "█    "]),
        'Q' | 'q' => Some(["▄███▄", "█   █", "▀██▀▄"]),
        'R' | 'r' => Some(["█▀▀▀█", "█▄▄▄█", "█  ▀▄"]),
        'S' | 's' => Some(["▄████", "▀▀▀▀▄", "████▀"]),
        'T' | 't' => Some(["▀▀█▀▀", "  █  ", "  █  "]),
        'U' | 'u' => Some(["█   █", "█   █", "▀███▀"]),
        'V' | 'v' => Some(["█   █", " █ █ ", "  █  "]),
        'W' | 'w' => Some(["█   █", "█ █ █", "█▀▀▀█"]),
        'X' | 'x' => Some([" █ █ ", "  █  ", " █ █ "]),
        'Y' | 'y' => Some([" █ █ ", "  █  ", "  █  "]),
        'Z' | 'z' => Some(["▀▀▀▀█", " ▄█▀ ", "█▄▄▄▄"]),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Font: hero  — 7 columns × 4 rows   (maximum terminal legibility)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns a 7-wide × 4-row glyph.
fn hero_glyph(ch: char) -> Option<[&'static str; 4]> {
    match ch {
        '0' => Some([" ▄███▄ ", "█▌   ▐█", "█▌   ▐█", " ▀███▀ "]),
        '1' => Some(["  ▄█   ", "   █   ", "   █   ", " ▄▄█▄▄ "]),
        '2' => Some([" ▄███▄ ", "    ▄█▀", " ▄█▀   ", "█▄▄▄▄▄▄"]),
        '3' => Some([" ▄███▄ ", "   ██▌ ", "   ██▌ ", " ▀███▀ "]),
        '4' => Some(["█    █ ", "█▄▄▄▄█▄", "     █ ", "     █ "]),
        '5' => Some(["█▄▄▄▄▄ ", "▀▀▀▀▀▄ ", "     ▐█", " ▄▄▄▄█▀"]),
        '6' => Some([" ▄████ ", "█▄▄▄▄  ", "█▌  ▐█ ", " ▀███▀ "]),
        '7' => Some(["▀▀▀▀▀▀█", "    ▄█▀", "   █▀  ", "  █    "]),
        '8' => Some([" ▄███▄ ", " ▐███▌ ", "█▌   ▐█", " ▀███▀ "]),
        '9' => Some([" ▄███▄ ", " ▀▀▀▀█ ", "  ████▀", " ▀▀▀▀  "]),
        ':' => Some(["       ", "   ●   ", "   ●   ", "       "]),
        '-' => Some(["       ", " ▀▀▀▀▀ ", "       ", "       "]),
        '.' => Some(["       ", "       ", "       ", "   ●   "]),
        ' ' => Some(["       ", "       ", "       ", "       "]),
        'A' | 'a' => Some(["  ▄█▄  ", " █▀ ▀█ ", "█▄▄▄▄▄█", "█     █"]),
        'B' | 'b' => Some(["█▀▀▀▀█ ", "█▄▄▄▄█ ", "█    █ ", "█▄▄▄▄█ "]),
        'C' | 'c' => Some([" ▄█████", "█▌     ", "█▌     ", " ▀█████"]),
        'D' | 'd' => Some(["█▀▀▀▀█ ", "█    █ ", "█    █ ", "█▄▄▄▄█ "]),
        'E' | 'e' => Some(["█▀▀▀▀▀ ", "█▄▄▄   ", "█      ", "█▄▄▄▄▄ "]),
        'F' | 'f' => Some(["█▀▀▀▀▀ ", "█▄▄▄   ", "█      ", "█      "]),
        'G' | 'g' => Some([" ▄█████", "█▌     ", "█▌ ▄▄██", " ▀█████"]),
        'H' | 'h' => Some(["█     █", "█▄▄▄▄▄█", "█▀▀▀▀▀█", "█     █"]),
        'I' | 'i' => Some([" ▄▄█▄▄ ", "   █   ", "   █   ", " ▀▀█▀▀ "]),
        'J' | 'j' => Some(["      █", "      █", "█    ▐█", " ▀███▀ "]),
        'K' | 'k' => Some(["█   ▄█▀", "█▄▄█▀  ", "█▀▀█▄  ", "█   ▀█▄"]),
        'L' | 'l' => Some(["█      ", "█      ", "█      ", "█▄▄▄▄▄▄"]),
        'M' | 'm' => Some(["█▄▄▄▄▄█", "█ ▐█▌ █", "█  ▐  █", "█     █"]),
        'N' | 'n' => Some(["█▄    █", "█ █   █", "█   █ █", "█    ▀█"]),
        'O' | 'o' => Some([" ▄███▄ ", "█▌   ▐█", "█▌   ▐█", " ▀███▀ "]),
        'P' | 'p' => Some(["█▀▀▀▀█ ", "█    █ ", "█▄▄▄▄█ ", "█      "]),
        'Q' | 'q' => Some([" ▄███▄ ", "█▌   ▐█", "█▌  █▐█", " ▀███▄▄"]),
        'R' | 'r' => Some(["█▀▀▀▀█ ", "█    █ ", "█▄▄▄▄█ ", "█   ▀█▄"]),
        'S' | 's' => Some([" ▄████ ", " ▀▀▀▀▄ ", "     ▐█", " ████▀ "]),
        'T' | 't' => Some(["▀▀▀█▀▀▀", "   █   ", "   █   ", "   █   "]),
        'U' | 'u' => Some(["█     █", "█     █", "█▌   ▐█", " ▀███▀ "]),
        'V' | 'v' => Some(["█     █", " █   █ ", "  █ █  ", "   █   "]),
        'W' | 'w' => Some(["█     █", "█  ▐  █", "█ ▐█▌ █", "█▀▀▀▀▀█"]),
        'X' | 'x' => Some([" █   █ ", "  █▄█  ", "  █▀█  ", " █   █ "]),
        'Y' | 'y' => Some([" █   █ ", "  █ █  ", "   █   ", "   █   "]),
        'Z' | 'z' => Some(["▀▀▀▀▀▀█", "   ▄█▀ ", "  █▀   ", "█▄▄▄▄▄▄"]),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Font style enum
// ═══════════════════════════════════════════════════════════════════════════════

/// Selects the native font resolution for BigMetric rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BigFont {
    /// 3 columns × 2 rows per glyph.  For secondary/small metrics.
    #[default]
    Compact,
    /// 5 columns × 3 rows per glyph.  For primary queue/dashboard numbers.
    Large,
    /// 7 columns × 4 rows per glyph.  For full-screen hero counters.
    Hero,
}

impl BigFont {
    /// Width (in terminal columns) of a single glyph in this font.
    fn glyph_width(self) -> u16 {
        match self {
            Self::Compact => 3,
            Self::Large => 5,
            Self::Hero => 7,
        }
    }

    /// Height (in terminal rows) of a single glyph in this font.
    fn glyph_height(self) -> u16 {
        match self {
            Self::Compact => 2,
            Self::Large => 3,
            Self::Hero => 4,
        }
    }

    /// Gap (in terminal columns) between adjacent glyphs.
    fn gap(self) -> u16 {
        match self {
            Self::Compact => 1,
            Self::Large => 1,
            Self::Hero => 1,
        }
    }

    /// Look up the glyph rows for `ch` in this font.
    /// Returns a slice of &str rows (length = glyph_height).
    fn lookup(self, ch: char) -> Option<GlyphRows> {
        match self {
            Self::Compact => compact_glyph(ch).map(GlyphRows::Two),
            Self::Large => large_glyph(ch).map(GlyphRows::Three),
            Self::Hero => hero_glyph(ch).map(GlyphRows::Four),
        }
    }
}

/// Variable-length glyph row storage.
enum GlyphRows {
    Two([&'static str; 2]),
    Three([&'static str; 3]),
    Four([&'static str; 4]),
}

impl GlyphRows {
    fn row(&self, idx: usize) -> &'static str {
        match self {
            GlyphRows::Two(r) => r[idx],
            GlyphRows::Three(r) => r[idx],
            GlyphRows::Four(r) => r[idx],
        }
    }

    fn len(&self) -> usize {
        match self {
            GlyphRows::Two(_) => 2,
            GlyphRows::Three(_) => 3,
            GlyphRows::Four(_) => 4,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Utility functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Total width needed to render `text` in the given font.
/// Returns 0 if any character is unsupported.
fn font_text_width(text: &str, font: BigFont) -> u16 {
    let count = text.chars().count();
    if count == 0 {
        return 0;
    }
    if !text.chars().all(|c| font.lookup(c).is_some()) {
        return 0;
    }
    let gw = font.glyph_width();
    let gap = font.gap();
    gw * (count as u16) + gap * (count as u16).saturating_sub(1)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Widget
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigMetric {
    id: ComponentId,
    title: Option<String>,
    value: String,
    unit: Option<String>,
    font: BigFont,
    /// Legacy scale field — mapped to BigFont in constructor.  Kept for DSL
    /// backwards-compatibility: if a TOML still says `scale = 2`, the registry
    /// converts it to `font = "large"`.
    style: Style,
    title_style: Style,
}

impl BigMetric {
    pub fn new(id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: ComponentId(id.into()),
            title: None,
            value: value.into(),
            unit: None,
            font: BigFont::Compact,
            style: Style {
                fg: Color::Reset,
                bold: true,
                ..Style::default()
            },
            title_style: Style {
                fg: Color::Indexed(8),
                ..Style::default()
            },
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the font directly.
    pub fn with_font(mut self, font: BigFont) -> Self {
        self.font = font;
        self
    }

    /// Legacy scale adapter — converts scale 1/2/3 to Compact/Large/Hero.
    pub fn with_scale(mut self, scale: u8) -> Self {
        self.font = match scale {
            2 => BigFont::Large,
            3 => BigFont::Hero,
            _ => BigFont::Compact,
        };
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Height of the glyph block (without title row).
    fn glyph_rows(&self) -> u16 {
        self.font.glyph_height()
    }

    /// Render the value in the selected font at (start_x, top_y).
    fn render_font(
        &self,
        frame: &mut Frame,
        start_x: u16,
        top_y: u16,
        available_width: u16,
    ) -> bool {
        let needed = font_text_width(&self.value, self.font);
        if needed == 0 || needed > available_width {
            return false;
        }

        let chars: Vec<char> = self.value.chars().collect();
        let gw = self.font.glyph_width();
        let gap = self.font.gap();
        let gh = self.font.glyph_height() as usize;
        let mut cur_x = start_x;

        for (i, &ch) in chars.iter().enumerate() {
            let Some(glyph) = self.font.lookup(ch) else {
                return false;
            };

            for row_idx in 0..gh.min(glyph.len()) {
                let row_str = glyph.row(row_idx);
                let dst_y = top_y + row_idx as u16;
                for (col, glyph_ch) in row_str.chars().enumerate() {
                    let dst_x = cur_x + col as u16;
                    let _ = frame.set(
                        dst_x,
                        dst_y,
                        crate::render::Cell {
                            symbol: glyph_ch,
                            style: self.style.clone(),
                        },
                    );
                }
            }

            cur_x += gw;
            if i + 1 < chars.len() {
                cur_x += gap;
            }
        }

        true
    }
}

impl Component for BigMetric {
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

        let title_rows = if self.title.is_some() { 1u16 } else { 0 };
        let glyph_rows = self.glyph_rows();
        let needed_rows = title_rows + glyph_rows;

        if area.height >= needed_rows {
            // ── Full big-display mode ─────────────────────────────────────────
            let mut row = area.y;

            if let Some(ref title) = self.title {
                let _ = frame.draw_text(area.x, row, title, self.title_style.clone());
                row += 1;
            }

            let success = self.render_font(frame, area.x, row, area.width);

            if !success {
                let _ = frame.draw_text_aligned(
                    area.x,
                    row,
                    area.width,
                    &self.value,
                    self.style.clone(),
                    TextAlign::Left,
                );
            }

            // Unit label — bottom-right of glyph block
            if let Some(ref unit) = self.unit {
                let val_width = font_text_width(&self.value, self.font);
                if val_width > 0 {
                    let unit_x = area.x.saturating_add(val_width).saturating_add(1);
                    let unit_y = row + glyph_rows - 1;
                    if unit_x < area.right() {
                        let unit_style = Style {
                            fg: Color::Indexed(8),
                            ..Style::default()
                        };
                        let _ = frame.draw_text(unit_x, unit_y, unit, unit_style);
                    }
                }
            }
        } else if area.height == 2 {
            // ── Compact two-row mode ─────────────────────────────────────────
            if let Some(ref title) = self.title {
                let _ = frame.draw_text(area.x, area.y, title, self.title_style.clone());
            }
            let val_y = if self.title.is_some() {
                area.y + 1
            } else {
                area.y
            };
            let val_text = match &self.unit {
                Some(u) => format!("{} {}", self.value, u),
                None => self.value.clone(),
            };
            let _ = frame.draw_text(area.x, val_y, &val_text, self.style.clone());
        } else {
            // ── Single-line compact mode ──────────────────────────────────────
            let text = match (&self.title, &self.unit) {
                (Some(t), Some(u)) => format!("{}: {} {}", t, self.value, u),
                (Some(t), None) => format!("{}: {}", t, self.value),
                (None, Some(u)) => format!("{} {}", self.value, u),
                (None, None) => self.value.clone(),
            };
            let _ = frame.draw_text(area.x, area.y, &text, self.style.clone());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ScreenBuffer;
    use crate::testing::snapshot_buffer;

    // ── Font width tests ─────────────────────────────────────────────────────

    #[test]
    fn compact_text_width() {
        // "98" = 3 + 1 + 3 = 7
        assert_eq!(font_text_width("98", BigFont::Compact), 7);
        assert_eq!(font_text_width("0", BigFont::Compact), 3);
        assert_eq!(font_text_width("", BigFont::Compact), 0);
    }

    #[test]
    fn large_text_width() {
        // "P022" = 4 chars × 5 wide + 3 gaps = 20 + 3 = 23
        assert_eq!(font_text_width("P022", BigFont::Large), 23);
        // single char = 5
        assert_eq!(font_text_width("0", BigFont::Large), 5);
    }

    #[test]
    fn hero_text_width() {
        // "42" = 2 chars × 7 wide + 1 gap = 14 + 1 = 15
        assert_eq!(font_text_width("42", BigFont::Hero), 15);
    }

    // ── Glyph coverage tests ─────────────────────────────────────────────────

    #[test]
    fn compact_covers_all_digits() {
        for ch in '0'..='9' {
            let g = compact_glyph(ch);
            assert!(g.is_some(), "compact missing digit '{}'", ch);
            let rows = g.unwrap();
            for (i, r) in rows.iter().enumerate() {
                assert_eq!(r.chars().count(), 3, "compact '{}' row {} width", ch, i);
            }
        }
    }

    #[test]
    fn large_covers_all_digits_and_common_letters() {
        for ch in ('0'..='9').chain("PNABCDEFGHIJKLMQRSTUVWXYZ".chars()) {
            let g = large_glyph(ch);
            assert!(g.is_some(), "large missing '{}'", ch);
            let rows = g.unwrap();
            for (i, r) in rows.iter().enumerate() {
                assert_eq!(r.chars().count(), 5, "large '{}' row {} width", ch, i);
            }
        }
    }

    #[test]
    fn hero_covers_all_digits_and_common_letters() {
        for ch in ('0'..='9').chain("PNABCDEFGHIJKLMQRSTUVWXYZ".chars()) {
            let g = hero_glyph(ch);
            assert!(g.is_some(), "hero missing '{}'", ch);
            let rows = g.unwrap();
            for (i, r) in rows.iter().enumerate() {
                assert_eq!(r.chars().count(), 7, "hero '{}' row {} width", ch, i);
            }
        }
    }

    // ── Rendering tests ──────────────────────────────────────────────────────

    #[test]
    fn compact_renders_block_digits_in_3_rows() {
        let widget = BigMetric::new("temp", "0").with_title("TEMP");
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 3));
        let mut frame = ScreenBuffer::new(20, 3);

        widget.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame).replace('·', " ");
        let lines: Vec<&str> = snap.lines().collect();
        assert!(lines[0].contains("TEMP"), "row 0 should have title");
        assert!(
            lines[1].contains('▄') || lines[1].contains('█'),
            "row 1 should have block chars: {}",
            lines[1]
        );
    }

    #[test]
    fn large_font_renders_3_rows_of_blocks() {
        let widget = BigMetric::new("queue", "P022").with_font(BigFont::Large);
        // large = 3 glyph rows, no title
        let ctx = RenderContext::new(Rect::new(0, 0, 40, 3));
        let mut frame = ScreenBuffer::new(40, 3);

        widget.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame).replace('·', " ");
        let lines: Vec<&str> = snap.lines().collect();
        let block_rows = lines
            .iter()
            .filter(|l| {
                l.contains('█')
                    || l.contains('▄')
                    || l.contains('▀')
                    || l.contains('▐')
                    || l.contains('▌')
            })
            .count();
        assert!(
            block_rows >= 3,
            "large font should produce 3 rows of block chars, got {}: {:?}",
            block_rows,
            lines
        );
    }

    #[test]
    fn hero_font_renders_4_rows() {
        let widget = BigMetric::new("hero", "42").with_font(BigFont::Hero);
        let ctx = RenderContext::new(Rect::new(0, 0, 30, 4));
        let mut frame = ScreenBuffer::new(30, 4);

        widget.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame).replace('·', " ");
        let lines: Vec<&str> = snap.lines().collect();
        let block_rows = lines
            .iter()
            .filter(|l| l.contains('█') || l.contains('▄') || l.contains('▀'))
            .count();
        assert!(
            block_rows >= 4,
            "hero font should produce 4 rows of block chars, got {}: {:?}",
            block_rows,
            lines
        );
    }

    #[test]
    fn with_scale_maps_to_font() {
        let w1 = BigMetric::new("a", "1").with_scale(1);
        assert_eq!(w1.font, BigFont::Compact);
        let w2 = BigMetric::new("b", "2").with_scale(2);
        assert_eq!(w2.font, BigFont::Large);
        let w3 = BigMetric::new("c", "3").with_scale(3);
        assert_eq!(w3.font, BigFont::Hero);
    }

    #[test]
    fn falls_back_to_compact_at_height_1() {
        let widget = BigMetric::new("cpu", "42")
            .with_title("CPU")
            .with_font(BigFont::Large);
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 1));
        let mut frame = ScreenBuffer::new(20, 1);

        widget.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame).replace('·', " ");
        assert!(snap.contains("CPU") && snap.contains("42"));
    }

    #[test]
    fn unit_appears_in_output() {
        let widget = BigMetric::new("v", "23").with_unit("V");
        let ctx = RenderContext::new(Rect::new(0, 0, 20, 2));
        let mut frame = ScreenBuffer::new(20, 2);

        widget.render(&ctx, &mut frame);

        let snap = snapshot_buffer(&frame).replace('·', " ");
        assert!(snap.contains('V'), "unit 'V' should appear in output");
    }
}
