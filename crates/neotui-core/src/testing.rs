use crate::render::ScreenBuffer;

pub fn snapshot_buffer(buffer: &ScreenBuffer) -> String {
    (0..buffer.height)
        .map(|y| {
            (0..buffer.width)
                .map(|x| {
                    buffer
                        .get(x, y)
                        .map(|cell| match cell.symbol {
                            ' ' => '·',
                            other => other,
                        })
                        .unwrap_or('·')
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Cell, ScreenBuffer, Style};

    #[test]
    fn snapshot_buffer_marks_empty_cells_consistently() {
        let mut buffer = ScreenBuffer::new(3, 1);
        let _ = buffer.set(
            1,
            0,
            Cell {
                symbol: 'X',
                style: Style::default(),
            },
        );

        assert_eq!(snapshot_buffer(&buffer), "·X·");
    }
}
