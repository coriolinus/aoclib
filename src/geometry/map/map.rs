use super::{a_star::AStarNode, tile::DisplayWidth, ContextInto, Edge, Traversable};
use crate::geometry::{Direction, Point};
use bitvec::bitvec;
use std::{
    collections::{BinaryHeap, HashMap, VecDeque},
    convert::TryFrom,
    fmt, hash,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[cfg(feature = "map-render")]
use {
    super::{
        render::{n_pixels_for, pixel_size, render_point, Encoder},
        tile::ToRgb,
        Animation, Style,
    },
    std::{path::Path, time::Duration},
};

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
pub struct Map<Tile> {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
    offset: Point,
}

impl<Tile> Map<Tile> {
    /// Procedurally create a new `Map` from a function.
    pub fn procedural(width: usize, height: usize, procedure: impl Fn(Point) -> Tile) -> Map<Tile> {
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
        procedure: impl Fn(Point) -> Tile,
    ) -> Map<Tile> {
        let area = width * height;
        let mut map = Map {
            tiles: Vec::with_capacity(area),
            width,
            height,
            offset,
        };
        for idx in 0..area {
            let point = map.index2point(idx);
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
    pub fn iter(&self) -> impl Iterator<Item = (Point, &Tile)> {
        let index2point = self.make_index2point();
        self.tiles
            .iter()
            .enumerate()
            .map(move |(idx, tile)| (index2point(idx), tile))
    }

    /// Iterate over the points of this tiles in this map, with mutable access to the tiles.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Point, &mut Tile)> {
        let index2point = self.make_index2point();
        self.tiles
            .iter_mut()
            .enumerate()
            .map(move |(idx, tile)| (index2point(idx), tile))
    }

    /// Iterate over the points of this map without depending on the lifetime of `self`.
    pub fn points(&self) -> impl Iterator<Item = Point> {
        let index2point = self.make_index2point();
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

    /// Make a function which returns `true` when the parameter is within the bounds of this map,
    /// without depending on the lifetime of `self`.
    pub fn make_in_bounds(&self) -> impl Fn(Point) -> bool {
        let low_x = self.low_x();
        let low_y = self.low_y();
        let high_x = self.high_x();
        let high_y = self.high_y();

        move |point| point.x >= low_x && point.y >= low_y && point.x <= high_x && point.y <= high_y
    }

    /// convert a 2d point into a 1d index into the tiles
    fn point2index(&self, x: usize, y: usize) -> usize {
        let x = (x as i32 - self.offset.x) as usize;
        let y = (y as i32 - self.offset.y) as usize;
        x + (y * self.width)
    }

    /// convert a 1d index in the tiles into a 2d point
    fn index2point(&self, idx: usize) -> Point {
        let unoffset: Point = (idx % self.width, idx / self.width).into();
        unoffset + self.offset
    }

    /// make a function which converts a 1d index in the tiles into a properly offset 2d point without borrowing self
    fn make_index2point(&self) -> impl Fn(usize) -> Point {
        let offset = self.offset;
        let width = self.width;

        move |idx| {
            let unoffset: Point = (idx % width, idx / width).into();
            unoffset + offset
        }
    }

    /// Return an iterator of all legal points adjacent to the given point.
    ///
    /// This iterator will return up to 8 elements; it includes diagonals.
    pub fn adjacencies(&self, point: Point) -> impl Iterator<Item = Point> {
        let in_bounds = self.make_in_bounds();
        self.orthogonal_adjacencies(point).chain(
            Direction::iter_diag()
                .map(move |(vertical, horizontal)| point + vertical + horizontal)
                .filter(move |&point| in_bounds(point)),
        )
    }

    /// Return an iterator of all legal points orthogonally adjacent to the given point,
    ///
    /// This iterator will return up to 4 elements; it does not include diagonals.
    pub fn orthogonal_adjacencies(&self, point: Point) -> impl Iterator<Item = Point> {
        let in_bounds = self.make_in_bounds();
        Direction::iter()
            .map(move |direction| point + direction)
            .filter(move |&point| in_bounds(point))
    }

    /// Return an iterator of all legal points adjacent to the given point,
    /// without depending on the lifetime of `self`.
    ///
    /// This iterator will return up to 8 elements; it includes diagonals.
    ///
    /// This introduces a bound that the `Tile` type must not contain any references.
    /// It is also slightly less efficient than [`self.adjacencies`]. In general,
    /// that function should be preferred unless there are lifetime conflicts.
    pub fn make_adjacencies(&self, point: Point) -> impl Iterator<Item = Point>
    where
        Tile: 'static,
    {
        let in_bounds = self.make_in_bounds();
        self.make_orthogonal_adjacencies(point).chain(
            Direction::iter_diag()
                .map(move |(vertical, horizontal)| point + vertical + horizontal)
                .filter(move |&point| in_bounds(point)),
        )
    }

    /// Return an iterator of all legal points orthogonally adjacent to the given point,
    /// without depending on the lifetime of `self`.
    ///
    /// This iterator will return up to 4 elements; it does not include diagonals.
    ///
    /// This introduces a bound that the `Tile` type must not contain any references.
    /// It is also slightly less efficient than [`self.orthogonal_adjacencies`]. In general,
    /// that function should be preferred unless there are lifetime conflicts.
    pub fn make_orthogonal_adjacencies(&self, point: Point) -> impl Iterator<Item = Point>
    where
        Tile: 'static,
    {
        let in_bounds = self.make_in_bounds();
        Direction::iter()
            .map(move |direction| point + direction)
            .filter(move |&point| in_bounds(point))
    }

    /// Return an iterator of all legal points arrived at by applying the given deltas to the origin.
    ///
    /// The origin point is always the first item in this iteration.
    pub fn project(&self, origin: Point, dx: i32, dy: i32) -> impl Iterator<Item = Point> {
        let in_bounds = self.make_in_bounds();
        std::iter::successors(Some(origin), move |&current| Some(current + (dx, dy)))
            .take_while(move |&point| in_bounds(point))
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

    /// Translate all points in this map by a given amount.
    ///
    /// Completes in `O(1)`.
    ///
    /// ## Example
    ///
    /// Using the symbols `X` and `O` to indicate tiles, and `.` to indicate out-of-bounds space
    /// away from the origin, we start with this map:
    ///
    /// ```notrust
    /// XOOX
    /// OXOX
    /// ```
    ///
    /// After applying `translate(2, 1)`, we end with this map:
    ///
    /// ```notrust
    /// ..XOOX
    /// ..OXOX
    /// ......
    /// ```
    pub fn translate(&mut self, dx: i32, dy: i32) {
        self.offset.x += dx;
        self.offset.y += dy;
    }

    /// Convert the underlying tile type of a map.
    ///
    /// This produces a new map whose tiles are of a new underlying type.
    pub fn convert_tile_type<NewTile>(self) -> Map<NewTile>
    where
        Tile: Into<NewTile>,
    {
        let mut tiles = Vec::with_capacity(self.tiles.len());
        tiles.extend(self.tiles.into_iter().map(Into::into));
        Map {
            tiles,
            width: self.width,
            height: self.height,
            offset: self.offset,
        }
    }
}

impl<Tile: Clone> Map<Tile> {
    /// Reduce the map to that portion which is interesting according to some user-defined metric.
    ///
    /// This can be helpful when preparing visualizations.
    pub fn extract_interesting_region(
        &self,
        is_interesting: impl Fn(Point, &Tile) -> bool,
    ) -> Self {
        let mut min = Point::new(i32::MAX, i32::MAX);
        let mut max = Point::new(i32::MIN, i32::MIN);

        for (point, tile) in self.iter() {
            if is_interesting(point, tile) {
                min.x = min.x.min(point.x);
                min.y = min.y.min(point.y);
                max.x = max.x.max(point.x);
                max.y = max.y.max(point.y);
            }
        }

        let width = (max.x - min.x + 1) as usize;
        let height = (max.y - min.y + 1) as usize;
        let offset = min;

        Self::procedural_offset(offset, width, height, |point| self[point].clone())
    }
}

impl<Tile: Clone + Default> Map<Tile> {
    /// Create a new map of the specified dimensions.
    ///
    /// Its lower left corner is at `(0, 0)`.
    #[inline]
    pub fn new(width: usize, height: usize) -> Map<Tile> {
        Self::new_offset(Point::default(), width, height)
    }

    /// Create a new map of the specified dimensions.
    ///
    /// Its lower left corner is at `offset`.
    #[inline]
    pub fn new_offset(offset: Point, width: usize, height: usize) -> Map<Tile> {
        Map {
            tiles: vec![Tile::default(); width * height],
            width,
            height,
            offset,
        }
    }

    /// Create a copy of this map which has been flipped vertically: the axis of symmetry is horizontal.
    ///
    /// This does not adjust the offset; the corners remain where they previously were.
    pub fn flip_vertical(&self) -> Map<Tile> {
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
    ///
    /// This does not adjust the offset; the corners remain where they previously were.
    pub fn flip_horizontal(&self) -> Map<Tile> {
        let mut flipped = Map::new_offset(self.offset, self.width, self.height);

        for y in self.low_y()..=self.high_y() {
            for x in 0..(self.width as i32) {
                let flipped_x = self.high_x() - x;
                let non_flipped_x = self.low_x() + x;
                flipped[Point::new(flipped_x, y)] = self[Point::new(non_flipped_x, y)].clone();
            }
        }

        flipped
    }

    /// Create a copy of this map which has been rotated counter-clockwise.
    ///
    /// This maintains the invariant that all points are in the positive quadrant, and assumes that
    /// the offset is `(0, 0)`. If necessary, apply `Self::translate` before and after this operation to
    /// produce an appropriate new offset.
    ///
    /// ## Panics
    ///
    /// If the offset is not `(0, 0)`.
    pub fn rotate_left(&self) -> Map<Tile> {
        assert_eq!(
            self.offset,
            Point::default(),
            "rotation is only legal when offset is `(0, 0)`"
        );

        let mut rotated = Map::new(self.height, self.width);

        let rotated_origin = rotated.bottom_right();
        for point in self.points() {
            let rotated_point = point.rotate_left() + rotated_origin;
            rotated[rotated_point] = self[point].clone();
        }

        rotated
    }

    /// Create a copy of this map which has been rotated clockwise.
    ///
    /// This maintains the invariant that all points are in the positive quadrant, and assumes that
    /// the offset is `(0, 0)`. If necessary, apply `Self::translate` before and after this operation to
    /// produce an appropriate new offset.
    ///
    /// ## Panics
    ///
    /// If the offset is not `(0, 0)`.
    pub fn rotate_right(&self) -> Map<Tile> {
        assert_eq!(
            self.offset,
            Point::default(),
            "rotation is only legal when offset is `(0, 0)`"
        );

        let mut rotated = Map::new(self.height, self.width);

        let rotated_origin = rotated.top_left();
        for point in self.points() {
            let rotated_point = point.rotate_right() + rotated_origin;
            rotated[rotated_point] = self[point].clone();
        }

        rotated
    }
}

impl<Tile> fmt::Debug for Map<Tile> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!("Map<{}>", std::any::type_name::<Tile>()))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("offset", &self.offset)
            .field("tiles", &format_args!("[...; {}]", self.tiles.len()))
            .finish()
    }
}

impl<Tile: hash::Hash> hash::Hash for Map<Tile> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.tiles.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.offset.hash(state);
    }
}

impl<Tile: PartialEq> PartialEq for Map<Tile> {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.offset == other.offset
            && self.tiles == other.tiles
    }
}

impl<Tile: Eq> Eq for Map<Tile> {}

impl<Tile, Row> From<&[Row]> for Map<Tile>
where
    Tile: Clone,
    Row: AsRef<[Tile]>,
{
    /// Convert an input 2d array into a map.
    ///
    /// Note that the input array must already be arranged with the y axis
    /// as the outer array and the orientation such that `source[0][0]` is the
    /// lower left corner of the map.
    ///
    /// Panics if the input array is not rectangular.
    fn from(source: &[Row]) -> Map<Tile> {
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

impl<Tile> Map<Tile>
where
    Tile: Clone + DisplayWidth + FromStr,
    <Tile as FromStr>::Err: 'static + std::error::Error + Send + Sync,
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

            let mut row = Vec::with_capacity(line.len() / Tile::DISPLAY_WIDTH);
            for chunk in Tile::chunks(&line) {
                row.push(Tile::from_str(&chunk).map_err(|err| {
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

impl<Tile> TryFrom<&str> for Map<Tile>
where
    Tile: Clone + DisplayWidth + FromStr,
    <Tile as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = MapConversionErr;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        <Self>::try_from(input.as_bytes())
    }
}

impl<Tile> TryFrom<std::fs::File> for Map<Tile>
where
    Tile: Clone + DisplayWidth + FromStr,
    <Tile as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = MapConversionErr;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: std::fs::File) -> Result<Self, Self::Error> {
        <Self>::try_from(std::io::BufReader::new(input))
    }
}

impl<Tile> TryFrom<&std::path::Path> for Map<Tile>
where
    Tile: Clone + DisplayWidth + FromStr,
    <Tile as FromStr>::Err: 'static + std::error::Error + Send + Sync,
{
    type Error = std::io::Error;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        <Self as TryFrom<std::fs::File>>::try_from(std::fs::File::open(path)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, Box::new(e)))
    }
}

impl<Tile> Index<(usize, usize)> for Map<Tile> {
    type Output = Tile;

    fn index(&self, (x, y): (usize, usize)) -> &Tile {
        self.tiles.index(self.point2index(x, y))
    }
}

impl<Tile> Index<Point> for Map<Tile> {
    type Output = Tile;

    /// Panics if `point.x < 0 || point.y < 0`
    fn index(&self, point: Point) -> &Tile {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index((point.x as usize, point.y as usize))
    }
}

impl<Tile> IndexMut<(usize, usize)> for Map<Tile> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Tile {
        self.tiles.index_mut(self.point2index(x, y))
    }
}

impl<Tile> IndexMut<Point> for Map<Tile> {
    /// Panics if `point.x < 0 || point.y < 0`
    fn index_mut(&mut self, point: Point) -> &mut Tile {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index_mut((point.x as usize, point.y as usize))
    }
}

impl<Tile> fmt::Display for Map<Tile>
where
    Tile: fmt::Display + DisplayWidth,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (self.low_y()..=self.high_y()).rev() {
            for x in self.low_x()..=self.high_x() {
                write!(
                    f,
                    "{:width$}",
                    self.index(Point::new(x, y)),
                    width = Tile::DISPLAY_WIDTH
                )?;
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
        // 16 pixels per light: 3x3 with a 1px margin
        // 3 subpixels per pixel; 1 each for r, g, b
        let width = self.width;
        let mut subpixels = vec![0; n_pixels_for(self.width, self.height) * 3];

        for (point, tile) in self.iter() {
            render_point(point - self.offset, tile, &mut subpixels, width, style)
        }

        gif::Frame::from_rgb(pixel_size(width), pixel_size(self.height), &subpixels)
    }

    fn make_gif_encoder(&self, output: &Path) -> Result<Encoder, RenderError> {
        let output = std::fs::File::create(output)?;
        let output = std::io::BufWriter::new(output);

        gif::Encoder::new(output, pixel_size(self.width), pixel_size(self.height), &[])
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

impl<Tile> Map<Tile>
where
    Tile: Clone + ContextInto<Traversable, Context = ()>,
{
    /// Visit every non-obstructed tile reachable from the initial point.
    ///
    /// If the visitor ever returns true, processing halts and no further
    /// points are visited.
    pub fn reachable_from(&self, point: Point, visit: impl FnMut(Point, &Tile) -> bool) {
        self.reachable_from_ctx(&(), point, visit)
    }

    /// navigate between the given points using A*
    // https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    pub fn navigate(&self, from: Point, to: Point) -> Option<Vec<Direction>> {
        self.navigate_ctx(&(), from, to)
    }
}

impl<Tile: Clone + ContextInto<Traversable>> Map<Tile> {
    /// Visit every non-obstructed tile reachable from the initial point.
    ///
    /// If the visitor ever returns true, processing halts and no further
    /// points are visited.
    pub fn reachable_from_ctx(
        &self,
        context: &<Tile as ContextInto<Traversable>>::Context,
        point: Point,
        mut visit: impl FnMut(Point, &Tile) -> bool,
    ) {
        let mut visited = bitvec!(0; self.tiles.len());
        let mut queue = VecDeque::new();
        queue.push_back(point);

        let idx = |point: Point| self.point2index(point.x as usize, point.y as usize);

        while let Some(point) = queue.pop_front() {
            // we may have scheduled a single point more than once via alternate paths;
            // we should only actually visit once.
            let index = idx(point);
            if visited[index] {
                continue;
            }

            visited.set(index, true);
            let traversable = self[point].clone().ctx_into(point, context);
            if traversable == Traversable::Obstructed {
                continue;
            }

            if visit(point, self.index(point)) {
                break;
            }

            if traversable == Traversable::Free {
                for neighbor in self.orthogonal_adjacencies(point) {
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
        context: &<Tile as ContextInto<Traversable>>::Context,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::map::tile::Digit;
    use std::{collections::HashSet, convert::TryInto};

    #[test]
    fn test_procedural() {
        let map = Map::procedural(2, 2, |point| point.x + point.y);
        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.offset, Point::default());
        assert_eq!(map.tiles, vec![0, 1, 1, 2]);
        assert!(map.iter().all(|(point, &tile)| point.x + point.y == tile));
    }

    #[test]
    fn test_procedural_offset() {
        let map = Map::procedural_offset(Point::new(2, 1), 2, 2, |point| point.x + point.y);
        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.offset, Point::new(2, 1));
        assert_eq!(map.tiles, vec![3, 4, 4, 5]);
        assert!(map.iter().all(|(point, &tile)| point.x + point.y == tile));
    }

    #[test]
    fn test_point_index_conversion_no_offset() {
        const EDGE: usize = 256;
        const AREA: usize = EDGE * EDGE;

        let map = Map::<()>::new(EDGE, EDGE);
        let mut emitted_points = HashSet::new();
        for idx in 0..AREA {
            let point = map.index2point(idx);
            assert!(
                emitted_points.insert(point),
                "no duplicate point should ever be emitted"
            );
            assert_eq!(idx, map.point2index(point.x as usize, point.y as usize));
        }
    }

    #[test]
    fn test_point_index_conversion_with_offset() {
        const EDGE: usize = 256;
        const AREA: usize = EDGE * EDGE;

        let map = Map::<()>::new_offset(Point::new(3, 2), EDGE, EDGE);
        let mut emitted_points = HashSet::new();
        for idx in 0..AREA {
            let point = map.index2point(idx);
            assert!(
                emitted_points.insert(point),
                "no duplicate point should ever be emitted"
            );
            assert_eq!(idx, map.point2index(point.x as usize, point.y as usize));
        }
    }

    #[test]
    fn test_boundaries_no_offset() {
        const EDGE: usize = 256;

        let map = Map::<()>::new(EDGE, EDGE);

        assert_eq!(map.low_x(), 0);
        assert_eq!(map.high_x(), 255);
        assert_eq!(map.low_y(), 0);
        assert_eq!(map.high_y(), 255);
    }

    #[test]
    fn test_boundaries_with_offset() {
        const EDGE: usize = 256;

        let map = Map::<()>::new_offset(Point::new(3, 2), EDGE, EDGE);

        assert_eq!(map.low_x(), 3);
        assert_eq!(map.high_x(), EDGE as i32 + 3 - 1);
        assert_eq!(map.low_y(), 2);
        assert_eq!(map.high_y(), EDGE as i32 + 2 - 1);
    }

    #[test]
    fn test_translate() {
        let mut map = Map::procedural(2, 2, |point| point.x + point.y);
        map.translate(2, 1);

        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.offset, Point::new(2, 1));
        assert_eq!(map.tiles, vec![0, 1, 1, 2]);
        assert!(map.iter().all(|(point, &tile)| {
            let point = point - map.offset;
            point.x + point.y == tile
        }));
    }

    #[test]
    fn test_extract_interesting_region() {
        let map = Map::procedural(2, 2, |point| point.x + point.y);
        let map = map.extract_interesting_region(|point, _tile| point.x != 0);

        assert_eq!(map.width, 1);
        assert_eq!(map.height, 2);
        assert_eq!(map.offset, Point::new(1, 0));
        assert_eq!(map.tiles, vec![1, 2]);
    }

    #[test]
    fn test_flip_vertical() {
        let map = Map::procedural_offset(Point::new(3, 2), 2, 3, |point| point.x + point.y);
        let bottom_left = map.bottom_left();
        let top_right = map.top_right();
        assert_eq!(map.tiles, vec![5, 6, 6, 7, 7, 8]);

        let flip_map = map.flip_vertical();
        assert_eq!(flip_map.bottom_left(), bottom_left);
        assert_eq!(flip_map.top_right(), top_right);

        assert_eq!(flip_map.width, 2);
        assert_eq!(flip_map.height, 3);
        assert_eq!(flip_map.tiles, vec![7, 8, 6, 7, 5, 6]);

        assert_eq!(flip_map.flip_vertical(), map);
    }

    #[test]
    fn test_flip_horizontal() {
        let map = Map::procedural_offset(Point::new(3, 2), 2, 3, |point| point.x + point.y);
        let bottom_left = map.bottom_left();
        let top_right = map.top_right();
        assert_eq!(map.tiles, vec![5, 6, 6, 7, 7, 8]);

        let flip_map = map.flip_horizontal();
        assert_eq!(flip_map.bottom_left(), bottom_left);
        assert_eq!(flip_map.top_right(), top_right);

        assert_eq!(flip_map.width, 2);
        assert_eq!(flip_map.height, 3);
        assert_eq!(flip_map.tiles, vec![6, 5, 7, 6, 8, 7]);

        assert_eq!(flip_map.flip_horizontal(), map);
    }

    #[test]
    fn test_rotate_left() {
        let map = Map::<Digit>::procedural(3, 2, |point| {
            ((point.x + point.y) as u8).try_into().unwrap()
        });
        assert_eq!(
            map.tiles
                .iter()
                .map(|&digit| digit.into())
                .collect::<Vec<u8>>(),
            vec![0, 1, 2, 1, 2, 3]
        );

        let rotated_map = map.rotate_left();

        println!("{}", map);
        println!("{}", rotated_map);

        assert_eq!(rotated_map.width, 2);
        assert_eq!(rotated_map.height, 3);
        assert_eq!(rotated_map.offset, Point::default());
        assert_eq!(
            rotated_map
                .tiles
                .iter()
                .map(|&digit| digit.into())
                .collect::<Vec<u8>>(),
            vec![1, 0, 2, 1, 3, 2],
        );

        assert_eq!(rotated_map.rotate_right(), map);
    }

    #[test]
    fn test_rotate_right() {
        let map = Map::<Digit>::procedural(3, 2, |point| {
            ((point.x + point.y) as u8).try_into().unwrap()
        });
        assert_eq!(
            map.tiles
                .iter()
                .map(|&digit| digit.into())
                .collect::<Vec<u8>>(),
            vec![0, 1, 2, 1, 2, 3]
        );

        let rotated_map = map.rotate_right();

        println!("{}", map);
        println!("{}", rotated_map);

        assert_eq!(rotated_map.width, 2);
        assert_eq!(rotated_map.height, 3);
        assert_eq!(rotated_map.offset, Point::default());
        assert_eq!(
            rotated_map
                .tiles
                .iter()
                .map(|&digit| digit.into())
                .collect::<Vec<u8>>(),
            vec![2, 3, 1, 2, 0, 1],
        );

        assert_eq!(rotated_map.rotate_left(), map);
    }
}
