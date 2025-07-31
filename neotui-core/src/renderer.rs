use crate::layout::Rect;

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Vec<char>>,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Plain,
    Double,
    Rounded,
}

impl ScreenBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![' '; width as usize]; height as usize];
        Self { width, height, cells }
    }

    pub fn clear(&mut self) {
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = ' ';
            }
        }
    }

    pub fn draw_text(&mut self, x: u16, y: u16, text: &str) {
        if (y as usize) < self.cells.len() {
            let row = &mut self.cells[y as usize];
            for (i, c) in text.chars().enumerate() {
                let idx = x as usize + i;
                if idx < row.len() {
                    row[idx] = c;
                }
            }
        }
    }

    pub fn flush(&self) {
        for row in &self.cells {
            let line: String = row.iter().collect();
            println!("{}", line);
        }
    }

    pub fn area(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub fn draw_border(&mut self, area: Rect, style: BorderStyle) {
        let x = area.x as usize;
        let y = area.y as usize;
        let w = area.width as usize;
        let h = area.height as usize;

        // Check for area validity
        if w < 2 || h < 2 || y + h > self.height as usize || x + w > self.width as usize {
            return;
        }

        let (right, bottom) = (x + w - 1, y + h - 1);

        // Select border characters based on style
        let (tl, tr, bl, br, hline, vline) = match style {
            BorderStyle::Plain => ('┌', '┐', '└', '┘', '─', '│'),
            BorderStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
            BorderStyle::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
        };

        // Draw corners
        self.cells[y][x] = tl;
        self.cells[y][right] = tr;
        self.cells[bottom][x] = bl;
        self.cells[bottom][right] = br;

        // Draw horizontal lines
        for i in x + 1..right {
            self.cells[y][i] = hline;
            self.cells[bottom][i] = hline;
        }

        // Draw vertical lines
        for j in y + 1..bottom {
            self.cells[j][x] = vline;
            self.cells[j][right] = vline;
        }
    }}

pub struct Renderer {
    buffer: ScreenBuffer,
}

impl Renderer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: ScreenBuffer::new(width, height),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn draw_text(&mut self, x: u16, y: u16, text: &str) {
        self.buffer.draw_text(x, y, text);
    }

    pub fn flush(&self) {
        self.buffer.flush();
    }

    pub fn area(&self) -> Rect {
        self.buffer.area()
    }

    pub fn buffer_mut(&mut self) -> &mut ScreenBuffer {
        &mut self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_draw_text() {
        let mut renderer = Renderer::new(10, 2);
        renderer.clear();
        renderer.draw_text(0, 0, "Hello");
        renderer.flush(); // prints to stdout
    }

    #[test]
    fn test_renderer_area() {
        let renderer = Renderer::new(80, 24);
        let area = renderer.area();
        assert_eq!(area.width, 80);
        assert_eq!(area.height, 24);
    }
}
