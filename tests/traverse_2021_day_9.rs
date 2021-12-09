//! Run with: `cargo test --test traverse_2021_day_9`

use aoclib::geometry::{
    map::{ContextInto, Traversable},
    tile::DisplayWidth,
    Point,
};
use std::convert::TryFrom;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::FromStr)]
struct Digit(aoclib::geometry::map::tile::Digit);

impl DisplayWidth for Digit {
    const DISPLAY_WIDTH: usize = aoclib::geometry::map::tile::Digit::DISPLAY_WIDTH;
}

impl From<Digit> for u8 {
    fn from(digit: Digit) -> u8 {
        digit.0.into()
    }
}

impl ContextInto<Traversable> for Digit {
    type Context = ();

    fn ctx_into(self, _position: Point, _context: &Self::Context) -> Traversable {
        match self.into() {
            9 => Traversable::Obstructed,
            _ => Traversable::Free,
        }
    }
}

type Map = aoclib::geometry::Map<Digit>;

fn read_input(example: &str) -> Result<(Map, Vec<Point>), Error> {
    let map = <Map as TryFrom<&str>>::try_from(example.trim())?;
    let low_points = map
        .iter()
        .filter(|(point, height)| {
            map.orthogonal_adjacencies(*point)
                .all(|adj| map[adj] > **height)
        })
        .map(|(point, _)| point)
        .collect();
    Ok((map, low_points))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("could not read map")]
    MapConv(#[from] aoclib::geometry::map::MapConversionErr),
}

const EXAMPLE: &str = r"
2199943210
3987894921
9856789892
8767896789
9899965678
";

#[test]
fn test_example_finds_region_without_overflow() {
    let (map, low_points) = read_input(EXAMPLE).unwrap();
    dbg!(&low_points);
    let mut region_sizes: Vec<_> = low_points
        .iter()
        .map(|point| {
            let mut size: u64 = 0;
            map.reachable_from(*point, |_point, _tile| {
                size += 1;
                false
            });
            size
        })
        .collect();
    region_sizes.sort_unstable();
    let basin_size_product: u64 = region_sizes.iter().rev().take(3).product();
    println!("product of 3 largest basin sizes: {}", basin_size_product);
}
