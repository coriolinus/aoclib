use crate::geometry::{tile::ToRgb, Map, Point};
use rand::Rng;
use std::time::Duration;

/// How each tile gets rendered.
#[derive(Debug, Clone, Copy)]
pub enum Style {
    /// Fill the entire 4x4 area with the tile's color.
    Fill,
    /// Fill a 3x3 area with the tile's color, leaving a 1 pixel black grid pattern between.
    Grid,
    /// Fill a 3x3 cross with the tile's color, leaving black space between.
    Cross,
    /// Fill a 3x3 cross with the tile's color, plus up to 4 more chosen randomly,
    /// for a sparkling effect.
    SparkleCross,
}

impl Style {
    fn offsets(self) -> Box<dyn Iterator<Item = Point>> {
        match self {
            Style::Cross => Box::new(std::array::IntoIter::new([
                Point::new(1, 0),
                Point::new(0, 1),
                Point::new(1, 1),
                Point::new(2, 1),
                Point::new(1, 2),
            ])),
            Style::SparkleCross => {
                let mut rng = rand::thread_rng();

                let corners = std::array::IntoIter::new([
                    Point::new(0, 0),
                    Point::new(0, 2),
                    Point::new(2, 0),
                    Point::new(2, 2),
                ])
                .filter(move |_point| rng.gen());

                Box::new(Style::Cross.offsets().chain(corners))
            }
            Style::Grid => Box::new((0..3).flat_map(|y| (0..3).map(move |x| Point::new(x, y)))),
            Style::Fill => Box::new((0..4).flat_map(|y| (0..4).map(move |x| Point::new(x, y)))),
        }
    }
}

pub fn render_point<Tile: ToRgb>(
    position: Point,
    tile: &Tile,
    subpixels: &mut [u8],
    width: usize,
    style: Style,
) {
    let x = |point: Point| point.x as usize;
    let y = |point: Point| point.y as usize;

    let row_pixels = pixel_width(width) as usize;

    // the linear index of a position has the following components:
    //
    // - 2: offset from left edge
    // - 2 * row_pixels: offset from top
    // - x(position) * 4: x component of position
    // - y(position) * 4 * row_pixels: y component of position
    // - x(offset): x offset
    // - y(offset) * row_pixels: y offset
    //
    // It is multiplied by 3, because that is how many bytes each pixel takes
    //
    // Note: this requires that the offset be in the positive quadrant
    let linear_idx = |offset: Point| {
        (2 + (2 * row_pixels)
            + (x(position) * 4)
            + (y(position) * 4 * row_pixels)
            + x(offset)
            + (y(offset) * row_pixels))
            * 3
    };

    let rgb = tile.to_rgb();

    for offset in style.offsets() {
        let idx = linear_idx(offset);
        subpixels[idx..idx + 3].copy_from_slice(&rgb);
    }
}

/// Each tile is 4px wide, with a 2px margin on the outside edges of the image.
pub fn pixel_width(width: usize) -> u16 {
    ((width + 1) * 4) as u16
}

/// Each tile is 4px high, with a 2px margin on the outside edges of the image.
pub fn pixel_height(height: usize) -> u16 {
    ((height + 1) * 4) as u16
}

/// Total pixels in an image for a map
///
/// Each tile is 4px high and 4px wide, with a 2px margin on the outside
/// edges of the image.
pub fn n_pixels_for(width: usize, height: usize) -> usize {
    pixel_width(width) as usize * pixel_height(height) as usize
}

pub type Encoder = gif::Encoder<std::io::BufWriter<std::fs::File>>;

/// An `Animation` holds a handle to an unfinished gif animation.
///
/// _Depends on the `map-render` feature._
///
/// It is created with [`Map::prepare_animation`].
///
/// The gif is finalized when this struct is dropped.
pub struct Animation {
    encoder: Encoder,
    style: Style,
}

impl Animation {
    /// Create a new animation from an encoder and frame duration.
    ///
    /// This animation will repeat infinitely, displaying each frame for
    /// `frame_duration`.
    pub(crate) fn new(
        mut encoder: Encoder,
        frame_duration: Duration,
        style: Style,
    ) -> Result<Animation, gif::EncodingError> {
        encoder.set_repeat(gif::Repeat::Infinite)?;

        // delay is set in hundredths of a second
        encoder.write_extension(gif::ExtensionData::new_control_ext(
            (frame_duration.as_millis() / 10) as u16,
            gif::DisposalMethod::Any,
            false,
            None,
        ))?;

        Ok(Animation { encoder, style })
    }

    /// Write a frame to this animation.
    ///
    /// This frame will be visible for the duration specified at the animation's
    /// creation.
    pub fn write_frame<Tile: ToRgb>(&mut self, map: &Map<Tile>) -> Result<(), gif::EncodingError> {
        self.encoder.write_frame(&map.render_frame(self.style))
    }
}
