use std::convert::TryFrom;

use super::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Direction {
    Right,
    Left,
    #[default]
    Up,
    Down,
}

impl Direction {
    /// `(dx, dy)`, for `Right` is `+x` and `Up` is `+y`
    pub fn deltas(self) -> (i32, i32) {
        use Direction::*;
        match self {
            Up => (0, 1),
            Down => (0, -1),
            Right => (1, 0),
            Left => (-1, 0),
        }
    }

    pub fn turn_right(self) -> Direction {
        use Direction::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }

    pub fn turn_left(self) -> Direction {
        use Direction::*;
        match self {
            Up => Left,
            Left => Down,
            Down => Right,
            Right => Up,
        }
    }

    pub fn reverse(self) -> Direction {
        use Direction::*;
        match self {
            Up => Down,
            Left => Right,
            Down => Up,
            Right => Left,
        }
    }

    /// Iterate over the four orthogonal directions
    pub fn iter() -> impl Iterator<Item = Direction> {
        use Direction::*;
        [Up, Down, Left, Right].iter().copied()
    }

    /// Iterate over the four diagonal direction-pairs
    ///
    /// Each pair takes the form `(vertical, horizontal)`.
    pub fn iter_diag() -> impl Iterator<Item = (Direction, Direction)> {
        use Direction::*;
        [(Up, Left), (Up, Right), (Down, Left), (Down, Right)]
            .iter()
            .copied()
    }
}


/// Inverse of [`Direction::deltas`].
impl TryFrom<Point> for Direction {
    type Error = ();

    fn try_from(value: Point) -> Result<Self, Self::Error> {
        match value {
            Point { x: 0, y: 1 } => Ok(Direction::Up),
            Point { x: 0, y: -1 } => Ok(Direction::Down),
            Point { x: 1, y: 0 } => Ok(Direction::Right),
            Point { x: -1, y: 0 } => Ok(Direction::Left),
            _ => Err(()),
        }
    }
}
