// Render model
// Core abstractions for framebuffer-style rendering

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

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(usize::from(y) * usize::from(self.width) + usize::from(x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
