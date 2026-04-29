// Render model
// Core abstractions for rendering

pub struct ScreenBuffer {
    pub width: u16,
    pub height: u16,
}

impl ScreenBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}
