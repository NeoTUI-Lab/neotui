// Render model
// Core abstractions for framebuffer-style rendering

use std::io::{self, Write};

use crate::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Indexed(u8),
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Default)]
pub struct AnsiRenderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Padding {
    pub fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub fn uniform(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyle {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl BorderStyle {
    pub fn single() -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
        }
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::single()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
            underlined: false,
        }
    }
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_to_string(&self, buffer: &ScreenBuffer) -> String {
        let mut output = String::new();
        let mut active_style = Style::default();

        for y in 0..buffer.height {
            output.push_str(&format!("\x1b[{};1H", y + 1));

            for x in 0..buffer.width {
                let Some(cell) = buffer.get(x, y) else {
                    continue;
                };

                if cell.style != active_style {
                    output.push_str(&ansi_style_sequence(&cell.style));
                    active_style = cell.style.clone();
                }

                output.push(cell.symbol);
            }
        }

        output.push_str("\x1b[0m");
        output
    }

    pub fn render_to_writer<W>(&self, buffer: &ScreenBuffer, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        writer.write_all(self.render_to_string(buffer).as_bytes())?;
        writer.flush()
    }

    pub fn render_to_stdout(&self, buffer: &ScreenBuffer) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.render_to_writer(buffer, &mut stdout)
    }

    pub fn render_diff_to_string(&self, previous: &ScreenBuffer, current: &ScreenBuffer) -> String {
        let diff = FrameDiff::between(previous, current);

        if diff.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let mut active_style = Style::default();

        for change in diff.changes {
            output.push_str(&format!("\x1b[{};{}H", change.y + 1, change.x + 1));

            if change.current.style != active_style {
                output.push_str(&ansi_style_sequence(&change.current.style));
                active_style = change.current.style.clone();
            }

            output.push(change.current.symbol);
        }

        output.push_str("\x1b[0m");
        output
    }

    pub fn render_diff_to_writer<W>(
        &self,
        previous: &ScreenBuffer,
        current: &ScreenBuffer,
        writer: &mut W,
    ) -> io::Result<()>
    where
        W: Write,
    {
        let output = self.render_diff_to_string(previous, current);

        if output.is_empty() {
            return Ok(());
        }

        writer.write_all(output.as_bytes())?;
        writer.flush()
    }

    pub fn render_diff_to_stdout(
        &self,
        previous: &ScreenBuffer,
        current: &ScreenBuffer,
    ) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.render_diff_to_writer(previous, current, &mut stdout)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            style: Style::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellChange {
    pub x: u16,
    pub y: u16,
    pub previous: Cell,
    pub current: Cell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameDiff {
    pub changes: Vec<CellChange>,
    pub dirty_regions: Vec<DirtyRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenBuffer {
    pub width: u16,
    pub height: u16,
    cells: Vec<Cell>,
}

impl ScreenBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);

        Self {
            width,
            height,
            cells: vec![Cell::default(); len],
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::default(); usize::from(width) * usize::from(height)];
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).and_then(|index| self.cells.get(index))
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) -> bool {
        let Some(index) = self.index(x, y) else {
            return false;
        };

        self.cells[index] = cell;
        true
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn draw_text(&mut self, x: u16, y: u16, text: &str, style: Style) -> usize {
        if y >= self.height || x >= self.width || text.is_empty() {
            return 0;
        }

        let mut written = 0;

        for (offset, symbol) in text.chars().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };

            let Some(target_x) = x.checked_add(offset) else {
                break;
            };

            if target_x >= self.width {
                break;
            }

            let wrote = self.set(
                target_x,
                y,
                Cell {
                    symbol,
                    style: style.clone(),
                },
            );

            if !wrote {
                break;
            }

            written += 1;
        }

        written
    }

    pub fn draw_text_aligned(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        text: &str,
        style: Style,
        align: TextAlign,
    ) -> usize {
        if width == 0 || text.is_empty() {
            return 0;
        }

        let text_width = text.chars().count().min(usize::from(width));
        let text_width = u16::try_from(text_width).unwrap_or(width);

        let offset = match align {
            TextAlign::Left => 0,
            TextAlign::Center => width.saturating_sub(text_width) / 2,
            TextAlign::Right => width.saturating_sub(text_width),
        };

        let Some(start_x) = x.checked_add(offset) else {
            return 0;
        };

        self.draw_text(start_x, y, text, style)
    }

    pub fn draw_panel(
        &mut self,
        area: Rect,
        title: Option<&str>,
        style: Style,
        padding: Padding,
        border: BorderStyle,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }

        let right = area.right().saturating_sub(1);
        let bottom = area.bottom().saturating_sub(1);

        for x in area.x..=right {
            let _ = self.set(
                x,
                area.y,
                Cell {
                    symbol: border.horizontal,
                    style: style.clone(),
                },
            );
            let _ = self.set(
                x,
                bottom,
                Cell {
                    symbol: border.horizontal,
                    style: style.clone(),
                },
            );
        }

        for y in area.y..=bottom {
            let _ = self.set(
                area.x,
                y,
                Cell {
                    symbol: border.vertical,
                    style: style.clone(),
                },
            );
            let _ = self.set(
                right,
                y,
                Cell {
                    symbol: border.vertical,
                    style: style.clone(),
                },
            );
        }

        let _ = self.set(
            area.x,
            area.y,
            Cell {
                symbol: border.top_left,
                style: style.clone(),
            },
        );
        let _ = self.set(
            right,
            area.y,
            Cell {
                symbol: border.top_right,
                style: style.clone(),
            },
        );
        let _ = self.set(
            area.x,
            bottom,
            Cell {
                symbol: border.bottom_left,
                style: style.clone(),
            },
        );
        let _ = self.set(
            right,
            bottom,
            Cell {
                symbol: border.bottom_right,
                style: style.clone(),
            },
        );

        if let Some(title) = title {
            let title_width = area.width.saturating_sub(4);
            if title_width > 0 {
                let title = format!(" {} ", title);
                let _ = self.draw_text(area.x.saturating_add(1), area.y, &title, style.clone());
            }
        }

        panel_content_rect(area, padding)
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }

    fn cell_or_default(&self, x: u16, y: u16) -> Cell {
        self.get(x, y).cloned().unwrap_or_default()
    }
}

pub fn panel_content_rect(area: Rect, padding: Padding) -> Rect {
    let bordered = area.inset(1, 1);
    let horizontal = padding.left.saturating_add(padding.right);
    let vertical = padding.top.saturating_add(padding.bottom);

    Rect::new(
        bordered.x.saturating_add(padding.left),
        bordered.y.saturating_add(padding.top),
        bordered.width.saturating_sub(horizontal),
        bordered.height.saturating_sub(vertical),
    )
}

impl FrameDiff {
    pub fn between(previous: &ScreenBuffer, current: &ScreenBuffer) -> Self {
        let width = previous.width.max(current.width);
        let height = previous.height.max(current.height);
        let mut changes = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let previous_cell = previous.cell_or_default(x, y);
                let current_cell = current.cell_or_default(x, y);

                if previous_cell != current_cell {
                    changes.push(CellChange {
                        x,
                        y,
                        previous: previous_cell,
                        current: current_cell,
                    });
                }
            }
        }

        let dirty_regions = build_dirty_regions(&changes);

        Self {
            changes,
            dirty_regions,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

fn ansi_style_sequence(style: &Style) -> String {
    let mut codes = vec!["0".to_string()];

    if style.bold {
        codes.push("1".to_string());
    }

    if style.italic {
        codes.push("3".to_string());
    }

    if style.underlined {
        codes.push("4".to_string());
    }

    codes.extend(color_to_ansi_codes(&style.fg, true));
    codes.extend(color_to_ansi_codes(&style.bg, false));

    format!("\x1b[{}m", codes.join(";"))
}

fn color_to_ansi_codes(color: &Color, foreground: bool) -> Vec<String> {
    let base = if foreground { 30 } else { 40 };
    let extended_prefix = if foreground { "38" } else { "48" };

    match color {
        Color::Reset => vec![(base + 9).to_string()],
        Color::Black => vec![base.to_string()],
        Color::Red => vec![(base + 1).to_string()],
        Color::Green => vec![(base + 2).to_string()],
        Color::Yellow => vec![(base + 3).to_string()],
        Color::Blue => vec![(base + 4).to_string()],
        Color::Magenta => vec![(base + 5).to_string()],
        Color::Cyan => vec![(base + 6).to_string()],
        Color::White => vec![(base + 7).to_string()],
        Color::Indexed(index) => vec![
            extended_prefix.to_string(),
            "5".to_string(),
            index.to_string(),
        ],
        Color::Rgb { r, g, b } => vec![
            extended_prefix.to_string(),
            "2".to_string(),
            r.to_string(),
            g.to_string(),
            b.to_string(),
        ],
    }
}

fn build_dirty_regions(changes: &[CellChange]) -> Vec<DirtyRegion> {
    let mut regions = Vec::new();
    let mut current: Option<DirtyRegion> = None;

    for change in changes {
        match &mut current {
            Some(region)
                if region.y == change.y
                    && region.x + region.width == change.x
                    && region.height == 1 =>
            {
                region.width += 1;
            }
            Some(region) => {
                regions.push(region.clone());
                current = Some(DirtyRegion {
                    x: change.x,
                    y: change.y,
                    width: 1,
                    height: 1,
                });
            }
            None => {
                current = Some(DirtyRegion {
                    x: change.x,
                    y: change.y,
                    width: 1,
                    height: 1,
                });
            }
        }
    }

    if let Some(region) = current {
        regions.push(region);
    }

    merge_vertical_regions(regions)
}

fn merge_vertical_regions(regions: Vec<DirtyRegion>) -> Vec<DirtyRegion> {
    let mut merged = Vec::new();

    for region in regions {
        if let Some(last) = merged.last_mut() {
            if last.x == region.x && last.width == region.width && last.y + last.height == region.y
            {
                last.height += region.height;
                continue;
            }
        }

        merged.push(region);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Debug, Default)]
    struct TrackingWriter {
        bytes: Vec<u8>,
        flush_count: usize,
    }

    impl Write for TrackingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flush_count += 1;
            Ok(())
        }
    }

    #[test]
    fn buffer_initializes_with_default_cells() {
        let buffer = ScreenBuffer::new(4, 3);

        assert_eq!(buffer.width, 4);
        assert_eq!(buffer.height, 3);
        assert_eq!(buffer.len(), 12);
        assert!(buffer.cells().iter().all(|cell| *cell == Cell::default()));
    }

    #[test]
    fn buffer_set_and_get_cell() {
        let mut buffer = ScreenBuffer::new(2, 2);
        let cell = Cell {
            symbol: 'N',
            style: Style {
                fg: Color::Cyan,
                bold: true,
                ..Style::default()
            },
        };

        assert!(buffer.set(1, 0, cell.clone()));
        assert_eq!(buffer.get(1, 0), Some(&cell));
        assert_eq!(buffer.get(0, 0), Some(&Cell::default()));
    }

    #[test]
    fn buffer_rejects_out_of_bounds_writes() {
        let mut buffer = ScreenBuffer::new(1, 1);

        assert!(!buffer.set(2, 0, Cell::default()));
        assert_eq!(buffer.get(2, 0), None);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn buffer_clear_restores_default_cells() {
        let mut buffer = ScreenBuffer::new(2, 1);
        let _ = buffer.set(
            0,
            0,
            Cell {
                symbol: 'X',
                style: Style {
                    fg: Color::Red,
                    ..Style::default()
                },
            },
        );

        buffer.clear();

        assert!(buffer.cells().iter().all(|cell| *cell == Cell::default()));
    }

    #[test]
    fn buffer_resize_recreates_storage() {
        let mut buffer = ScreenBuffer::new(2, 2);
        let _ = buffer.set(
            1,
            1,
            Cell {
                symbol: 'Z',
                ..Cell::default()
            },
        );

        buffer.resize(3, 1);

        assert_eq!(buffer.width, 3);
        assert_eq!(buffer.height, 1);
        assert_eq!(buffer.len(), 3);
        assert!(buffer.cells().iter().all(|cell| *cell == Cell::default()));
    }

    #[test]
    fn draw_text_writes_symbols_with_style() {
        let mut buffer = ScreenBuffer::new(5, 1);
        let style = Style {
            fg: Color::Green,
            bold: true,
            ..Style::default()
        };

        let written = buffer.draw_text(1, 0, "Neo", style.clone());

        assert_eq!(written, 3);
        assert_eq!(
            buffer.get(1, 0),
            Some(&Cell {
                symbol: 'N',
                style: style.clone(),
            })
        );
        assert_eq!(buffer.get(3, 0), Some(&Cell { symbol: 'o', style }));
    }

    #[test]
    fn draw_text_clips_at_buffer_edge() {
        let mut buffer = ScreenBuffer::new(4, 1);

        let written = buffer.draw_text(2, 0, "Hello", Style::default());

        assert_eq!(written, 2);
        assert_eq!(buffer.get(2, 0).map(|cell| cell.symbol), Some('H'));
        assert_eq!(buffer.get(3, 0).map(|cell| cell.symbol), Some('e'));
    }

    #[test]
    fn draw_text_ignores_out_of_bounds_origin() {
        let mut buffer = ScreenBuffer::new(3, 1);

        assert_eq!(buffer.draw_text(3, 0, "X", Style::default()), 0);
        assert_eq!(buffer.draw_text(0, 1, "X", Style::default()), 0);
        assert!(buffer.cells().iter().all(|cell| *cell == Cell::default()));
    }

    #[test]
    fn draw_text_aligned_centers_text_within_width() {
        let mut buffer = ScreenBuffer::new(7, 1);

        let written = buffer.draw_text_aligned(0, 0, 7, "Neo", Style::default(), TextAlign::Center);

        assert_eq!(written, 3);
        assert_eq!(buffer.get(2, 0).map(|cell| cell.symbol), Some('N'));
        assert_eq!(buffer.get(3, 0).map(|cell| cell.symbol), Some('e'));
        assert_eq!(buffer.get(4, 0).map(|cell| cell.symbol), Some('o'));
    }

    #[test]
    fn draw_text_aligned_right_aligns_text_within_width() {
        let mut buffer = ScreenBuffer::new(6, 1);

        let written = buffer.draw_text_aligned(0, 0, 6, "UI", Style::default(), TextAlign::Right);

        assert_eq!(written, 2);
        assert_eq!(buffer.get(4, 0).map(|cell| cell.symbol), Some('U'));
        assert_eq!(buffer.get(5, 0).map(|cell| cell.symbol), Some('I'));
    }

    #[test]
    fn draw_text_aligned_clips_long_text_to_available_width() {
        let mut buffer = ScreenBuffer::new(4, 1);

        let written =
            buffer.draw_text_aligned(0, 0, 2, "Hello", Style::default(), TextAlign::Center);

        assert_eq!(written, 2);
        assert_eq!(buffer.get(0, 0).map(|cell| cell.symbol), Some('H'));
        assert_eq!(buffer.get(1, 0).map(|cell| cell.symbol), Some('e'));
    }

    #[test]
    fn ansi_renderer_renders_plain_buffer() {
        let mut buffer = ScreenBuffer::new(3, 1);
        let _ = buffer.draw_text(0, 0, "Neo", Style::default());

        let output = AnsiRenderer::new().render_to_string(&buffer);

        assert_eq!(output, "\x1b[1;1HNeo\x1b[0m");
    }

    #[test]
    fn ansi_renderer_emits_style_sequences() {
        let mut buffer = ScreenBuffer::new(2, 1);
        let _ = buffer.set(
            0,
            0,
            Cell {
                symbol: 'N',
                style: Style {
                    fg: Color::Green,
                    bold: true,
                    ..Style::default()
                },
            },
        );
        let _ = buffer.set(1, 0, Cell::default());

        let output = AnsiRenderer::new().render_to_string(&buffer);

        assert!(output.contains("\x1b[0;1;32;49mN"));
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    fn ansi_renderer_moves_cursor_for_each_row() {
        let mut buffer = ScreenBuffer::new(2, 2);
        let _ = buffer.draw_text(0, 0, "A", Style::default());
        let _ = buffer.draw_text(0, 1, "B", Style::default());

        let output = AnsiRenderer::new().render_to_string(&buffer);

        assert!(output.contains("\x1b[1;1H"));
        assert!(output.contains("\x1b[2;1H"));
    }

    #[test]
    fn ansi_renderer_writes_and_flushes_output() {
        let mut buffer = ScreenBuffer::new(2, 1);
        let _ = buffer.draw_text(0, 0, "Hi", Style::default());
        let mut writer = TrackingWriter::default();

        AnsiRenderer::new()
            .render_to_writer(&buffer, &mut writer)
            .expect("renderer should flush ANSI output");

        assert_eq!(writer.flush_count, 1);
        assert_eq!(
            String::from_utf8(writer.bytes).expect("ANSI output should be valid utf-8"),
            "\u{1b}[1;1HHi\u{1b}[0m"
        );
    }

    #[test]
    fn frame_diff_tracks_changed_cells() {
        let previous = ScreenBuffer::new(3, 1);
        let mut current = ScreenBuffer::new(3, 1);
        let _ = current.draw_text(1, 0, "N", Style::default());

        let diff = FrameDiff::between(&previous, &current);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].x, 1);
        assert_eq!(diff.changes[0].y, 0);
        assert_eq!(diff.changes[0].previous, Cell::default());
        assert_eq!(diff.changes[0].current.symbol, 'N');
    }

    #[test]
    fn frame_diff_builds_dirty_regions() {
        let mut previous = ScreenBuffer::new(3, 2);
        let mut current = ScreenBuffer::new(3, 2);
        let _ = previous.draw_text(0, 0, "abc", Style::default());
        let _ = previous.draw_text(0, 1, "abc", Style::default());
        let _ = current.draw_text(1, 0, "bc", Style::default());
        let _ = current.draw_text(1, 1, "bc", Style::default());

        let diff = FrameDiff::between(&previous, &current);

        assert_eq!(
            diff.dirty_regions,
            vec![DirtyRegion {
                x: 0,
                y: 0,
                width: 1,
                height: 2,
            }]
        );
    }

    #[test]
    fn frame_diff_handles_resize_as_new_dirty_area() {
        let previous = ScreenBuffer::new(1, 1);
        let mut current = ScreenBuffer::new(2, 1);
        let _ = current.draw_text(1, 0, "X", Style::default());

        let diff = FrameDiff::between(&previous, &current);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].x, 1);
        assert_eq!(diff.dirty_regions.len(), 1);
    }

    #[test]
    fn ansi_renderer_renders_only_changed_cells_for_diff() {
        let previous = ScreenBuffer::new(3, 1);
        let mut current = ScreenBuffer::new(3, 1);
        let _ = current.draw_text(1, 0, "N", Style::default());

        let output = AnsiRenderer::new().render_diff_to_string(&previous, &current);

        assert_eq!(output, "\x1b[1;2HN\x1b[0m");
    }

    #[test]
    fn ansi_renderer_skips_flush_for_empty_diff() {
        let previous = ScreenBuffer::new(2, 1);
        let current = ScreenBuffer::new(2, 1);
        let mut writer = TrackingWriter::default();

        AnsiRenderer::new()
            .render_diff_to_writer(&previous, &current, &mut writer)
            .expect("empty diff should be a no-op");

        assert_eq!(writer.flush_count, 0);
        assert!(writer.bytes.is_empty());
    }

    #[test]
    fn draw_panel_renders_border_and_title() {
        let mut buffer = ScreenBuffer::new(12, 5);
        let style = Style {
            fg: Color::Yellow,
            ..Style::default()
        };

        let content = buffer.draw_panel(
            Rect::new(0, 0, 12, 5),
            Some("Stats"),
            style.clone(),
            Padding::uniform(0),
            BorderStyle::default(),
        );

        assert_eq!(buffer.get(0, 0).map(|cell| cell.symbol), Some('+'));
        assert_eq!(buffer.get(11, 4).map(|cell| cell.symbol), Some('+'));
        assert_eq!(buffer.get(1, 0).map(|cell| cell.symbol), Some(' '));
        assert_eq!(buffer.get(2, 0).map(|cell| cell.symbol), Some('S'));
        assert_eq!(buffer.get(6, 0).map(|cell| cell.symbol), Some('s'));
        assert_eq!(content, Rect::new(1, 1, 10, 3));
    }

    #[test]
    fn panel_content_rect_applies_padding_inside_border() {
        let content = panel_content_rect(Rect::new(0, 0, 10, 6), Padding::symmetric(1, 2));

        assert_eq!(content, Rect::new(2, 3, 6, 0));
        assert!(content.is_empty());
    }

    #[test]
    fn draw_panel_handles_small_area_without_panic() {
        let mut buffer = ScreenBuffer::new(2, 2);

        let content = buffer.draw_panel(
            Rect::new(0, 0, 2, 2),
            Some("X"),
            Style::default(),
            Padding::default(),
            BorderStyle::default(),
        );

        assert_eq!(buffer.get(0, 0).map(|cell| cell.symbol), Some('+'));
        assert_eq!(buffer.get(1, 1).map(|cell| cell.symbol), Some('+'));
        assert!(content.is_empty());
    }
}
