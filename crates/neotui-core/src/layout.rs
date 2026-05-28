// Layout model
// Core geometry primitives for layout calculations

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    Fixed(u16),
    Percentage(u16),
    Flex(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_position_size(position: Position, size: Size) -> Self {
        Self {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn position(&self) -> Position {
        Position::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.x
            && position.x < self.right()
            && position.y >= self.y
            && position.y < self.bottom()
    }

    pub fn inset(&self, horizontal: u16, vertical: u16) -> Self {
        let inset_width = horizontal.saturating_mul(2);
        let inset_height = vertical.saturating_mul(2);

        Self {
            x: self.x.saturating_add(horizontal),
            y: self.y.saturating_add(vertical),
            width: self.width.saturating_sub(inset_width),
            height: self.height.saturating_sub(inset_height),
        }
    }

    pub fn intersect(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right <= x || bottom <= y {
            return Rect::new(x, y, 0, 0);
        }

        Rect::new(x, y, right - x, bottom - y)
    }
}

pub fn split(area: Rect, axis: Axis, constraints: &[Constraint]) -> Vec<Rect> {
    if constraints.is_empty() {
        return Vec::new();
    }

    let total_space = match axis {
        Axis::Vertical => area.height,
        Axis::Horizontal => area.width,
    };

    let mut lengths = vec![0u16; constraints.len()];
    let mut used = 0u16;
    let mut flex_total = 0u16;

    for (index, constraint) in constraints.iter().enumerate() {
        let length = match *constraint {
            Constraint::Fixed(value) => value.min(total_space.saturating_sub(used)),
            Constraint::Percentage(percent) => {
                let percent = percent.min(100);
                ((u32::from(total_space) * u32::from(percent)) / 100) as u16
            }
            Constraint::Flex(weight) => {
                flex_total = flex_total.saturating_add(weight.max(1));
                0
            }
        };

        lengths[index] = length;
        used = used.saturating_add(length);
    }

    let remaining = total_space.saturating_sub(used);
    if remaining > 0 && flex_total > 0 {
        let mut distributed = 0u16;

        for (index, constraint) in constraints.iter().enumerate() {
            if let Constraint::Flex(weight) = *constraint {
                let weight = weight.max(1);
                let share =
                    ((u32::from(remaining) * u32::from(weight)) / u32::from(flex_total)) as u16;
                lengths[index] = lengths[index].saturating_add(share);
                distributed = distributed.saturating_add(share);
            }
        }

        let leftover = remaining.saturating_sub(distributed);
        if leftover > 0 {
            for (index, constraint) in constraints.iter().enumerate().rev() {
                if matches!(constraint, Constraint::Flex(_)) {
                    lengths[index] = lengths[index].saturating_add(leftover);
                    break;
                }
            }
        }
    }

    build_rects(area, axis, &lengths)
}

pub fn split_vertical(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    split(area, Axis::Vertical, constraints)
}

pub fn split_horizontal(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    split(area, Axis::Horizontal, constraints)
}

fn build_rects(area: Rect, axis: Axis, lengths: &[u16]) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(lengths.len());
    let mut cursor = 0u16;

    for &length in lengths {
        let rect = match axis {
            Axis::Vertical => Rect::new(
                area.x,
                area.y.saturating_add(cursor),
                area.width,
                length.min(area.height.saturating_sub(cursor)),
            ),
            Axis::Horizontal => Rect::new(
                area.x.saturating_add(cursor),
                area.y,
                length.min(area.width.saturating_sub(cursor)),
                area.height,
            ),
        };

        rects.push(rect);
        cursor = cursor.saturating_add(length);
    }

    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_builds_from_position_and_size() {
        let rect = Rect::from_position_size(Position::new(2, 3), Size::new(10, 4));

        assert_eq!(rect.x, 2);
        assert_eq!(rect.y, 3);
        assert_eq!(rect.width, 10);
        assert_eq!(rect.height, 4);
    }

    #[test]
    fn rect_exposes_position_and_size() {
        let rect = Rect::new(1, 2, 5, 6);

        assert_eq!(rect.position(), Position::new(1, 2));
        assert_eq!(rect.size(), Size::new(5, 6));
    }

    #[test]
    fn rect_contains_points_inside_bounds() {
        let rect = Rect::new(2, 2, 4, 3);

        assert!(rect.contains(Position::new(2, 2)));
        assert!(rect.contains(Position::new(5, 4)));
        assert!(!rect.contains(Position::new(6, 4)));
        assert!(!rect.contains(Position::new(5, 5)));
    }

    #[test]
    fn rect_inset_shrinks_area_safely() {
        let rect = Rect::new(1, 1, 8, 6);
        let inset = rect.inset(2, 1);

        assert_eq!(inset, Rect::new(3, 2, 4, 4));
    }

    #[test]
    fn rect_inset_never_underflows() {
        let rect = Rect::new(0, 0, 2, 2);
        let inset = rect.inset(5, 5);

        assert_eq!(inset, Rect::new(5, 5, 0, 0));
        assert!(inset.is_empty());
    }

    #[test]
    fn rect_intersection_returns_overlap() {
        let left = Rect::new(0, 0, 5, 5);
        let right = Rect::new(3, 2, 5, 4);

        assert_eq!(left.intersect(&right), Rect::new(3, 2, 2, 3));
    }

    #[test]
    fn rect_intersection_returns_empty_when_disjoint() {
        let left = Rect::new(0, 0, 2, 2);
        let right = Rect::new(3, 3, 2, 2);

        let intersection = left.intersect(&right);

        assert!(intersection.is_empty());
        assert_eq!(intersection, Rect::new(3, 3, 0, 0));
    }

    #[test]
    fn size_knows_when_it_is_empty() {
        assert!(Size::new(0, 1).is_empty());
        assert!(Size::new(1, 0).is_empty());
        assert!(!Size::new(1, 1).is_empty());
    }

    #[test]
    fn split_vertical_supports_fixed_constraints() {
        let rects = split_vertical(
            Rect::new(0, 0, 10, 6),
            &[Constraint::Fixed(2), Constraint::Fixed(4)],
        );

        assert_eq!(rects, vec![Rect::new(0, 0, 10, 2), Rect::new(0, 2, 10, 4)]);
    }

    #[test]
    fn split_horizontal_supports_percentage_constraints() {
        let rects = split_horizontal(
            Rect::new(0, 0, 10, 3),
            &[Constraint::Percentage(30), Constraint::Percentage(70)],
        );

        assert_eq!(rects, vec![Rect::new(0, 0, 3, 3), Rect::new(3, 0, 7, 3)]);
    }

    #[test]
    fn split_vertical_distributes_remaining_space_to_flex() {
        let rects = split_vertical(
            Rect::new(0, 0, 8, 10),
            &[
                Constraint::Fixed(2),
                Constraint::Flex(1),
                Constraint::Flex(2),
            ],
        );

        assert_eq!(
            rects,
            vec![
                Rect::new(0, 0, 8, 2),
                Rect::new(0, 2, 8, 2),
                Rect::new(0, 4, 8, 6)
            ]
        );
    }

    #[test]
    fn split_clamps_when_fixed_constraints_exceed_available_space() {
        let rects = split_horizontal(
            Rect::new(0, 0, 5, 1),
            &[Constraint::Fixed(4), Constraint::Fixed(4)],
        );

        assert_eq!(rects, vec![Rect::new(0, 0, 4, 1), Rect::new(4, 0, 1, 1)]);
    }

    #[test]
    fn split_returns_empty_when_no_constraints_are_provided() {
        let rects = split(Rect::new(0, 0, 5, 5), Axis::Vertical, &[]);
        assert!(rects.is_empty());
    }

    #[test]
    fn split_assigns_leftover_space_to_last_flex_constraint() {
        let rects = split_horizontal(
            Rect::new(0, 0, 10, 1),
            &[
                Constraint::Flex(1),
                Constraint::Flex(1),
                Constraint::Flex(1),
            ],
        );

        assert_eq!(
            rects,
            vec![
                Rect::new(0, 0, 3, 1),
                Rect::new(3, 0, 3, 1),
                Rect::new(6, 0, 4, 1),
            ]
        );
    }

    #[test]
    fn split_clamps_percentage_before_distribution() {
        let rects = split_vertical(
            Rect::new(0, 0, 4, 10),
            &[Constraint::Percentage(150), Constraint::Flex(1)],
        );

        assert_eq!(rects, vec![Rect::new(0, 0, 4, 10), Rect::new(0, 10, 4, 0)]);
    }
}
