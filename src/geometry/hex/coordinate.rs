use std::ops::{Add, AddAssign};

use super::direction::Direction;

/// Axial hex coordinates.
///
/// See [reference](https://www.redblobgames.com/grids/hexagons/#coordinates).
///
/// Constraint: `q + r + s == 0`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Coordinate {
    pub q: i32,
    pub r: i32,
}

impl Coordinate {
    pub fn neighbors(self) -> impl 'static + Iterator<Item = Coordinate> {
        Direction::iter().map(move |direction| self + direction)
    }
}

impl AddAssign<Direction> for Coordinate {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::East => {
                self.q += 1;
            }
            Direction::Southeast => {
                self.r += 1;
            }
            Direction::Southwest => {
                self.q -= 1;
                self.r += 1;
            }
            Direction::West => {
                self.q -= 1;
            }
            Direction::Northwest => {
                self.r -= 1;
            }
            Direction::Northeast => {
                self.q += 1;
                self.r -= 1;
            }
        }
    }
}

impl Add<Direction> for Coordinate {
    type Output = Coordinate;

    fn add(mut self, rhs: Direction) -> Self::Output {
        self += rhs;
        self
    }
}
