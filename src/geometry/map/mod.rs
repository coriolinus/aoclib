#[cfg(feature = "map-render")]
mod render;
#[cfg(feature = "map-render")]
pub use render::{Animation, Style};

use crate::geometry::{tile::DisplayWidth, Direction, Point};
use bitvec::bitvec;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[cfg(feature = "map-render")]
use crate::geometry::tile::ToRgb;
#[cfg(feature = "map-render")]
use std::path::Path;
#[cfg(feature = "map-render")]
use std::time::Duration;

/// A Map keeps track of a tile grid.
///
/// Its coordinate system assumes that the origin is in the lower left,
/// for compatibility with [`Direction`].
///
/// While it is possible to clone a map, it is generally safe to assume that doing so
/// is a sign that there's a better approach possible.
///
/// ## Entry Points
///
/// - [`Map::new`] is most useful when the problem involves cartography.
/// - When a map is provided as the day's input, use [`Map::try_from`]
///
/// ## Panics
///
/// Several internal methods assume that the width and height of the map can be
/// represented in an `i32`. Very large maps may panic if that assumption is violated.
#[derive(Clone, Default)]
pub struct Map<T> {
    tiles: Vec<T>,
    width: usize,
    height: usize,
    offset: Point,
}

impl<T> Map<T> {
    /// Procedurally create a new `Map` from a function.
    pub fn procedural(width: usize, height: usize, procedure: impl Fn(Point) -> T) -> Map<T> {
        Self::procedural_offset(Point::default(), width, height, procedure)
    }

    /// Procedurally create a new `Map` from a function, with an offset origin.
    ///
    /// This offset can reduce dead space when the interesting part of a map is
    /// far from the origin.
    pub fn procedural_offset(
        offset: Point,
        width: usize,
        height: usize,
        procedure: impl Fn(Point) -> T,
    ) -> Map<T> {
        let area = width * height;
        let mut map = Map {
            tiles: Vec::with_capacity(area),
            width,
            height,
            offset,
        };
        for idx in 0..area {
            let point = map.index2point(idx).into();
            map.tiles.push(procedure(point));
        }
        map
    }

    /// Width of this map.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height of this map.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Offset of the lower left corner of this map from `(0, 0)`.
    #[inline]
    pub fn offset(&self) -> Point {
        self.offset
    }

    /// Lowest x coordinate which is in bounds of this map.
    #[inline]
    pub fn low_x(&self) -> i32 {
        self.offset.x
    }

    /// Highest x coordinate which is in bounds of this map.
    ///
    /// Note that this is inclusive; use `..=` when using this to bound a range.
    #[inline]
    pub fn high_x(&self) -> i32 {
        self.offset.x + self.width as i32 - 1
    }

    /// Lowest y coordinate which is in bounds of this map.
    #[inline]
    pub fn low_y(&self) -> i32 {
        self.offset.y
    }

    /// Highest y coordinate which is in bounds of this map.
    ///
    /// Note that this is inclusive; use `..=` when using this to bound a range.
    #[inline]
    pub fn high_y(&self) -> i32 {
        self.offset.y + self.height as i32 - 1
    }

    /// The coordinates of the bottom left corner of this map.
    ///
    /// This is inclusive; it is a valid index into the map.
    #[inline]
    pub fn bottom_left(&self) -> Point {
        Point::new(self.low_x(), self.low_y())
    }

    /// The coordinates of the top left corner of this map.
    ///
    /// This is inclusive; it is a valid index into the map.
    #[inline]
    pub fn top_left(&self) -> Point {
        Point::new(self.low_x(), self.high_y())
    }

    /// The coordinates of the bottom right corner of this map.
    ///
    /// This is inclusive; it is a valid index into the map.
    #[inline]
    pub fn bottom_right(&self) -> Point {
        Point::new(self.high_x(), self.low_y())
    }

    /// The coordinates of the top right corner of this map.
    ///
    /// This is inclusive; it is a valid index into the map.
    #[inline]
    pub fn top_right(&self) -> Point {
        Point::new(self.high_x(), self.high_y())
    }

    /// Iterate over the points and tiles of this map.
    pub fn iter(&self) -> impl Iterator<Item = (Point, &T)> {
        let index2point = self.make_offset_index2point();
        self.tiles
            .iter()
            .enumerate()
            .map(move |(idx, tile)| (index2point(idx), tile))
    }

    /// Iterate over the points of this tiles in this map, with mutable access to the tiles.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Point, &mut T)> {
        let index2point = self.make_offset_index2point();
        self.tiles
            .iter_mut()
            .enumerate()
            .map(move |(idx, tile)| (index2point(idx), tile))
    }

    /// Iterate over the points of this map without depending on the lifetime of `self`.
    pub fn points(&self) -> impl Iterator<Item = Point> {
        let index2point = self.make_offset_index2point();
        (0..self.tiles.len()).map(index2point)
    }

    /// `true` when a point is legal within the bounds of this map.
    #[inline]
    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= self.low_x()
            && point.y >= self.low_y()
            && point.x <= self.high_x()
            && point.y <= self.high_y()
    }

    /// convert a 2d point into a 1d index into the tiles
    ///
    /// **Note**: doesn't take the offset into account
    fn point2index(&self, x: usize, y: usize) -> usize {
        x + (y * self.width)
    }

    /// convert a 1d index in the tiles into a 2d point
    ///
    /// **Note**: doesn't take the offset into account
    fn index2point(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }

    /// make a function which converts a 1d index in the tiles into a 2d point without borrowing self
    fn make_index2point(&self) -> impl Fn(usize) -> (usize, usize) {
        let width = self.width;
        move |idx: usize| (idx % width, idx / width)
    }

    /// make a function which converts a 1d index in the tiles into a properly offset 2d point without borrowing self
    fn make_offset_index2point(&self) -> impl Fn(usize) -> Point {
        let offset = self.offset;
        let index2point = self.make_index2point();
        move |idx| {
            let unoffset: Point = index2point(idx).into();
            unoffset + offset
        }
    }

    /// Return an iterator of all legal points adjacent to the given point.
    ///
    /// This iterator will return up to 8 elements; it includes diagonals.
    pub fn adjacencies(&self, point: Point) -> impl '_ + Iterator<Item = Point> {
        self.orthogonal_adjacencies(point).chain(
            Direction::iter_diag()
                .map(move |(vertical, horizontal)| point + vertical + horizontal)
                .filter(move |&point| self.in_bounds(point)),
        )
    }

    /// Return an iterator of all legal points orthogonally adjacent to the given point.
    ///
    /// This iterator will return up to 4 elements; it does not include diagonals.
    pub fn orthogonal_adjacencies(&self, point: Point) -> impl '_ + Iterator<Item = Point> {
        Direction::iter()
            .map(move |direction| point + direction)
            .filter(move |&point| self.in_bounds(point))
    }

    /// Return an iterator of all legal points arrived at by applying the given deltas to the origin.
    ///
    /// The origin point is always the first item in this iteration.
    pub fn project(&self, origin: Point, dx: i32, dy: i32) -> impl '_ + Iterator<Item = Point> {
        std::iter::successors(Some(origin), move |&current| Some(current + (dx, dy)))
            .take_while(move |&point| self.in_bounds(point))
    }

    /// Create an iterator over the points on the edge of this map.
    ///
    /// Note that this iterator returns the points which are coordinates for each point on the edge,
    /// not the items of this map. You can use the [`Iterator::map`] combinator to map it to items from
    /// the map, if desired.
    ///
    /// The edge is traversed in increasing order. It is a [`std::iter::DoubleEndedIterator`], though, so
    /// it can be reversed if desired.
    ///
    /// The input `direction` indicates which edge should be traversed.
    pub fn edge(&self, direction: Direction) -> Edge {
        let (from, to, direction) = match direction {
            Direction::Left => (self.bottom_left(), self.top_left(), Direction::Up),
            Direction::Right => (self.bottom_right(), self.top_right(), Direction::Up),
            Direction::Down => (self.bottom_left(), self.bottom_right(), Direction::Right),
            Direction::Up => (self.top_left(), self.top_right(), Direction::Right),
        };

        Edge {
            from,
            to,
            direction,
            done: false,
        }
    }
}

impl<T: Clone + Default> Map<T> {
    /// Create a new map of the specified dimensions.
    ///
    /// Its lower left corner is at `(0, 0)`.
    #[inline]
    pub fn new(width: usize, height: usize) -> Map<T> {
        Self::new_offset(Point::default(), width, height)
    }

    /// Create a new map of the specified dimensions.
    ///
    /// Its lower left corner is at `offset`.
    #[inline]
    pub fn new_offset(offset: Point, width: usize, height: usize) -> Map<T> {
        Map {
            tiles: vec![T::default(); width * height],
            width,
            height,
            offset,
        }
    }

    /// Create a copy of this map which has been flipped vertically: the axis of symmetry is horizontal.
    ///
    /// This does not adjust the offset; the corners remain where they previously were.
    pub fn flip_vertical(&self) -> Map<T> {
        let mut flipped = Map::new_offset(self.offset, self.width, self.height);

        for y in 0..(self.height as i32) {
            let flipped_y = self.high_y() - y;
            let non_flipped_y = self.low_y() + y;
            for x in self.low_x()..=self.high_x() {
                flipped[Point::new(x, flipped_y)] = self[Point::new(x, non_flipped_y)].clone();
            }
        }

        flipped
    }

    /// Create a copy of this map which has been flipped horizontally; the axis of symmetry is vertical.
    pub fn flip_horizontal(&self) -> Map<T> {
        let mut flipped = Map::new_offset(self.offset, self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let flipped_x = self.width - x - 1;
                flipped[(flipped_x, y)] = self[(x, y)].clone();
            }
        }

        flipped
    }

    /// Create a copy of this map which has been rotated counter-clockwise.
    pub fn rotate_left(&self) -> Map<T> {
        let mut rotated = Map::new(self.height, self.width);

        let rotated_origin = self.bottom_right();
        for point in self.points() {
            let rotated_point = point.rotate_left() + rotated_origin;
            rotated[rotated_point] = self[point].clone();
        }

        rotated
    }

    /// Create a copy of this map which has been rotated clockwise.
    pub fn rotate_right(&self) -> Map<T> {
        let mut rotated = Map::new(self.height, self.width);

        let rotated_origin = self.top_left();
        for point in self.points() {
            let rotated_point = point.rotate_right() + rotated_origin;
            rotated[rotated_point] = self[point].clone();
        }

        rotated
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Map<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tiles.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.offset.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Map<T> {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.tiles == other.tiles
            && self.offset == other.offset
    }
}

impl<T: Eq> Eq for Map<T> {}

impl<T, R> From<&[R]> for Map<T>
where
    T: Clone,
    R: AsRef<[T]>,
{
    /// Convert an input 2d array into a map.
    ///
    /// Note that the input array must already be arranged with the y axis
    /// as the outer array and the orientation such that `source[0][0]` is the
    /// lower left corner of the map.
    ///
    /// Panics if the input array is not rectangular.
    fn from(source: &[R]) -> Map<T> {
        let height = source.len();
        if height == 0 {
            return Map {
                tiles: Vec::new(),
                width: 0,
                height: 0,
                offset: Point::default(),
            };
        }

        let width = source[0].as_ref().len();
        assert!(
            source
                .as_ref()
                .iter()
                .all(|row| row.as_ref().len() == width),
            "input must be rectangular"
        );

        let mut tiles = Vec::with_capacity(width * height);
        for row in source.iter() {
            for tile in row.as_ref().iter() {
                tiles.push(tile.clone());
            }
        }

        Map {
            tiles,
            width,
            height,
            offset: Point::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MapConversionErr {
    #[error("converting tile from {1:?}")]
    TileConversion(
        #[source] Box<dyn 'static + std::error::Error + Send + Sync>,
        String,
    ),
    #[error("map must be rectangular")]
    NotRectangular,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<T> Map<T>
where
    T: Clone + DisplayWidth + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    /// Try to convert the contents of a reader into a map.
    ///
    /// We don't actually `impl<T, R> TryFrom<R> for Map<T>` because there's a
    /// coherence conflict with the stdlib blanket impl
    ///
    /// ```rust,ignore
    /// impl<T, U> std::convert::TryFrom<U> for T where U: std::convert::Into<T>;
    /// ```
    ///
    /// Because there's a chance that `R` also implements `Into<Map<T>>`, we can't do it.
    ///
    /// That doesn't stop us from doing it here, and implementing the official trait for
    /// a few concrete types
    pub fn try_from<R>(input: R) -> Result<Self, MapConversionErr>
    where
        R: std::io::BufRead,
    {
        let mut arr = Vec::new();

        for line in input.lines() {
            let line = line?;

            let mut row = Vec::with_capacity(line.len() / T::DISPLAY_WIDTH);
            for chunk in T::chunks(&line) {
                row.push(T::from_str(&chunk).map_err(|err| {
                    MapConversionErr::TileConversion(Box::new(err), chunk.to_string())
                })?);
            }
            if !row.is_empty() {
                arr.push(row);
            }
        }

        if !arr.is_empty() {
            let width = arr[0].len();
            if !arr.iter().all(|row| row.len() == width) {
                return Err(MapConversionErr::NotRectangular);
            }
        }

        // shift the origin
        arr.reverse();

        Ok(Map::from(arr.as_slice()))
    }
}

impl<T> TryFrom<&str> for Map<T>
where
    T: Clone + DisplayWidth + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = MapConversionErr;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        <Self>::try_from(input.as_bytes())
    }
}

impl<T> TryFrom<std::fs::File> for Map<T>
where
    T: Clone + DisplayWidth + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = MapConversionErr;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: std::fs::File) -> Result<Self, Self::Error> {
        <Self>::try_from(std::io::BufReader::new(input))
    }
}

impl<T> TryFrom<&std::path::Path> for Map<T>
where
    T: Clone + DisplayWidth + FromStr,
    <T as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = std::io::Error;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        <Self as TryFrom<std::fs::File>>::try_from(std::fs::File::open(path)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, Box::new(e)))
    }
}

impl<T> Index<(usize, usize)> for Map<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &T {
        self.tiles.index(self.point2index(x, y))
    }
}

impl<T> Index<Point> for Map<T> {
    type Output = T;

    /// Panics if `point.x < 0 || point.y < 0`
    fn index(&self, point: Point) -> &T {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index((point.x as usize, point.y as usize))
    }
}

impl<T> IndexMut<(usize, usize)> for Map<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut T {
        self.tiles.index_mut(self.point2index(x, y))
    }
}

impl<T> IndexMut<Point> for Map<T> {
    /// Panics if `point.x < 0 || point.y < 0`
    fn index_mut(&mut self, point: Point) -> &mut T {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index_mut((point.x as usize, point.y as usize))
    }
}

impl<T> fmt::Display for Map<T>
where
    T: fmt::Display + DisplayWidth,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{:width$}", self.index((x, y)), width = T::DISPLAY_WIDTH)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(feature = "map-render")]
impl<Tile> Map<Tile>
where
    Tile: ToRgb,
{
    /// Render this map as a [`gif::Frame`].
    pub(crate) fn render_frame(&self, style: Style) -> gif::Frame {
        use render::{n_pixels_for, pixel_height, pixel_width};

        // 16 pixels per light: 3x3 with a 1px margin
        // 3 subpixels per pixel; 1 each for r, g, b
        let width = self.width;
        let mut subpixels = vec![0; n_pixels_for(self.width, self.height) * 3];

        for (point, tile) in self.iter() {
            render::render_point(point, tile, &mut subpixels, width, style)
        }

        gif::Frame::from_rgb(pixel_width(width), pixel_height(self.height), &subpixels)
    }

    fn make_gif_encoder(&self, output: &Path) -> Result<render::Encoder, RenderError> {
        use render::{pixel_height, pixel_width};

        let output = std::fs::File::create(output)?;
        let output = std::io::BufWriter::new(output);

        gif::Encoder::new(
            output,
            pixel_width(self.width),
            pixel_height(self.height),
            &[],
        )
        .map_err(Into::into)
    }

    /// Render this map as a still image into an output file.
    ///
    /// _Depends on the `map-render` feature._
    ///
    /// The output image is a gif under all circumstances. It is useful, though
    /// unenforced, that the output file name matches `*.gif`.
    pub fn render(&self, output: &Path, style: Style) -> Result<(), RenderError> {
        let mut output = self.make_gif_encoder(output)?;
        output.write_frame(&self.render_frame(style))?;
        Ok(())
    }

    /// Prepare an animation from this map.
    ///
    /// _Depends on the `map-render` feature._
    ///
    /// This returns an `Animation` object which can have frames added to it.
    /// This method does not automatically render this `Map` frame to the `Animation`.
    /// This enables you to set up the animation ahead of time with dummy data.
    ///
    /// The major constraint is that all subsequent maps added as frames must
    /// have dimensions identical to this Map's.
    ///
    /// The animation will loop infinitely, displaying each frame for
    /// `frame_duration`.
    pub fn prepare_animation(
        &self,
        output: &Path,
        frame_duration: Duration,
        style: Style,
    ) -> Result<Animation, RenderError> {
        let encoder = self.make_gif_encoder(output)?;
        Animation::new(encoder, frame_duration, style).map_err(Into::into)
    }
}

/// An error which can arise during rendering.
///
/// _Depends on the `map-render` feature._
#[cfg(feature = "map-render")]
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("encoding gif")]
    Gif(#[from] gif::EncodingError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Can a visitor move through this map tile?
pub enum Traversable {
    /// Obstructed tiles cannot be moved into.
    Obstructed,
    /// Free tiles can be moved through.
    Free,
    /// Halt tiles can be moved into, but not past.
    Halt,
}

/// Safe fast value-to-value conversion which consumes the input value and references some context.
///
/// This trait should be implemented in preference to [`ContextInto`][ContextInto].
pub trait ContextFrom<T> {
    type Context;

    fn ctx_from(t: T, position: Point, context: &Self::Context) -> Self;
}

impl<A, B> ContextFrom<A> for B
where
    B: From<A>,
{
    type Context = ();

    fn ctx_from(a: A, _position: Point, _context: &()) -> B {
        B::from(a)
    }
}

/// Safe fast value-to-value conversion which consumes the input value and references some context.
///
/// This differs from [`Into`][std::convert::Into] in that it requires a context.
/// Also, because of a blanket implementation, it cannot be manually implemented for a given `T`
/// for any type which also implements `Into<T>`.
pub trait ContextInto<T> {
    type Context;

    fn ctx_into(self, position: Point, context: &Self::Context) -> T;
}

impl<A, B> ContextInto<B> for A
where
    B: ContextFrom<A>,
{
    type Context = <B as ContextFrom<A>>::Context;

    fn ctx_into(self, position: Point, context: &Self::Context) -> B {
        B::ctx_from(self, position, context)
    }
}

impl<T> Map<T>
where
    T: Clone + ContextInto<Traversable, Context = ()>,
{
    /// Visit every non-obstructed tile reachable from the initial point.
    ///
    /// If the visitor ever returns true, processing halts and no further
    /// points are visited.
    pub fn reachable_from<F>(&self, point: Point, visit: F)
    where
        F: FnMut(&T, Point) -> bool,
    {
        self.reachable_from_ctx(&(), point, visit)
    }

    /// navigate between the given points using A*
    // https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    pub fn navigate(&self, from: Point, to: Point) -> Option<Vec<Direction>> {
        self.navigate_ctx(&(), from, to)
    }
}

impl<T: Clone + ContextInto<Traversable>> Map<T> {
    /// Visit every non-obstructed tile reachable from the initial point.
    ///
    /// If the visitor ever returns true, processing halts and no further
    /// points are visited.
    pub fn reachable_from_ctx<F>(
        &self,
        context: &<T as ContextInto<Traversable>>::Context,
        point: Point,
        mut visit: F,
    ) where
        F: FnMut(&T, Point) -> bool,
    {
        let mut visited = bitvec!(0; self.tiles.len());
        let mut queue = VecDeque::new();
        queue.push_back(point);

        let idx = |point: Point| point.x as usize + (point.y as usize * self.width);

        while let Some(point) = queue.pop_front() {
            // we may have scheduled a single point more than once via alternate paths;
            // we should only actually visit once.
            if visited[idx(point)] {
                continue;
            }

            visited.set(idx(point), true);
            let traversable = self[point].clone().ctx_into(point, context);
            if traversable != Traversable::Obstructed && visit(&self[point], point) {
                break;
            }

            if traversable == Traversable::Free {
                for direction in Direction::iter() {
                    let neighbor = point + direction;
                    if !visited[idx(neighbor)] {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    /// navigate between the given points using A*
    // https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    pub fn navigate_ctx(
        &self,
        context: &<T as ContextInto<Traversable>>::Context,
        from: Point,
        to: Point,
    ) -> Option<Vec<Direction>> {
        let mut open_set = BinaryHeap::new();
        open_set.push(AStarNode {
            cost: 0,
            position: from,
        });

        // key: node
        // value: node preceding it on the cheapest known path from start
        let mut came_from = HashMap::new();

        // gscore
        // key: position
        // value: cost of cheapest path from start to node
        let mut cheapest_path_cost = HashMap::new();
        cheapest_path_cost.insert(from, 0_u32);

        // fscore
        // key: position
        // value: best guess as to total cost from here to finish
        let mut total_cost_guess = HashMap::new();
        total_cost_guess.insert(from, (to - from).manhattan() as u32);

        while let Some(AStarNode { cost, position }) = open_set.pop() {
            if position == to {
                let mut current = position;
                let mut path = Vec::new();
                while let Some((direction, predecessor)) = came_from.remove(&current) {
                    current = predecessor;
                    path.push(direction);
                }
                debug_assert!(path.len() as i32 >= (to - from).manhattan());
                path.reverse();
                return Some(path);
            }

            for direction in Direction::iter() {
                let neighbor = position + direction;
                if !self.in_bounds(neighbor) {
                    continue;
                }
                match self[neighbor].clone().ctx_into(neighbor, context) {
                    Traversable::Obstructed => {}
                    Traversable::Free | Traversable::Halt => {
                        let tentative_cheapest_path_cost = cost + 1;
                        if tentative_cheapest_path_cost
                            < cheapest_path_cost
                                .get(&neighbor)
                                .cloned()
                                .unwrap_or(u32::MAX)
                        {
                            // this path to the neighbor is better than any previous one
                            came_from.insert(neighbor, (direction, position));
                            cheapest_path_cost.insert(neighbor, tentative_cheapest_path_cost);
                            total_cost_guess.insert(
                                neighbor,
                                tentative_cheapest_path_cost + (to - neighbor).manhattan() as u32,
                            );

                            // this thing with the iterator is not very efficient, but for some weird reason BinaryHeap
                            // doesn't have a .contains method; see
                            // https://github.com/rust-lang/rust/issues/66724
                            if !open_set.iter().any(|elem| elem.position == neighbor) {
                                open_set.push(AStarNode {
                                    cost: tentative_cheapest_path_cost,
                                    position: neighbor,
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

/// A* State
// https://doc.rust-lang.org/std/collections/binary_heap/#examples
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct AStarNode {
    cost: u32,
    position: Point,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for AStarNode {
    fn cmp(&self, other: &AStarNode) -> std::cmp::Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &AStarNode) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Iterator over points on the edge of a [`Map`].
///
/// Created by the [`Map::edge`] function. See there for more details.
pub struct Edge {
    from: Point,
    to: Point,
    direction: Direction,
    done: bool,
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
