#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn split_vertically(&self, count: usize) -> Vec<Rect> {
        let mut areas = Vec::new();
        if count == 0 {
            return areas;
        }

        let each_height = self.height / count as u16;
        let mut y = self.y;

        for _ in 0..count {
            areas.push(Rect::new(self.x, y, self.width, each_height));
            y += each_height;
        }

        areas
    }
}
