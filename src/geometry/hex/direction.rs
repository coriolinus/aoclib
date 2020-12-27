use std::str::FromStr;

/// Direction in a hexagonal coordinate system
///
/// Assumes that the major orientation is horizontal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    East,
    Southeast,
    Southwest,
    West,
    Northwest,
    Northeast,
}

impl Direction {
    /// Iterate through all `Direction`s, clockwise from `East`.
    pub fn iter() -> impl Iterator<Item = Direction> {
        std::iter::successors(Some(Direction::East), |direction| {
            use Direction::*;

            match direction {
                East => Some(Southeast),
                Southeast => Some(Southwest),
                Southwest => Some(West),
                West => Some(Northwest),
                Northwest => Some(Northeast),
                Northeast => None,
            }
        })
    }

    /// Attempt to parse a direction from the head of the given string.
    ///
    /// Returns `(maybe_direction, unused_portion)`.
    ///
    /// Legal inputs (case sensitive): `e`, `se`, `sw`, `w`, `nw`, `ne`.
    pub fn try_parse(s: &str) -> (Option<Direction>, &str) {
        let mut chars = s.chars();
        let first = chars.next();
        let second = chars.next();
        match (first, second) {
            (Some('e'), _) => (Some(Direction::East), &s[1..]),
            (Some('s'), Some('e')) => (Some(Direction::Southeast), &s[2..]),
            (Some('s'), Some('w')) => (Some(Direction::Southwest), &s[2..]),
            (Some('w'), _) => (Some(Direction::West), &s[1..]),
            (Some('n'), Some('w')) => (Some(Direction::Northwest), &s[2..]),
            (Some('n'), Some('e')) => (Some(Direction::Northeast), &s[2..]),
            _ => (None, s),
        }
    }
}

/// Helper for parsing a line of directions.
pub struct Directions(pub Vec<Direction>);

impl FromStr for Directions {
    type Err = ParseDirectionsError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut directions = Vec::with_capacity(s.len());

        while !s.is_empty() {
            let (direction, remaining) = Direction::try_parse(s);
            match direction {
                None => return Err(ParseDirectionsError),
                Some(direction) => directions.push(direction),
            }

            s = remaining;
        }

        Ok(Directions(directions))
    }
}

/// Parsing failed for a line of hex directions
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Parsing hex direction failed")]
pub struct ParseDirectionsError;
