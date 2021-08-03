use crate::geometry::{Direction, Point};

/// Iterator over points on the edge of a [`Map`].
///
/// Created by the [`Map::edge`] function. See there for more details.
pub struct Edge {
    pub(crate) from: Point,
    pub(crate) to: Point,
    pub(crate) direction: Direction,
    pub(crate) done: bool,
}

impl Iterator for Edge {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next = self.from;
        self.from += self.direction;
        self.done = next == self.to;

        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.to - self.from).manhattan() as usize + 1;
        (size, Some(size))
    }
}

impl std::iter::ExactSizeIterator for Edge {}

impl std::iter::DoubleEndedIterator for Edge {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next = self.to;
        self.to += self.direction.reverse();
        self.done = next == self.from;

        Some(next)
    }
}
