use crate::geometry::{tile::ToRgb, Point};
use rand::Rng as _;
// use std::time::Duration;
// use std::{convert::TryFrom, path::Path};

/// If `sparkle`, each lit point illuminates 5 pixels in the shape of a cross, plus
/// up to 4 more, chosen randomly, which form a sparkling effect
///
/// Otherwise, the lit point illuminates all 9 pixels in the square.
pub fn render_point<Tile: ToRgb>(
    position: Point,
    tile: &Tile,
    subpixels: &mut [u8],
    width: usize,
    sparkle: bool,
) {
    let mut rng = rand::thread_rng();

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

    // central cross shape
    for offset in [
        Point::new(1, 0),
        Point::new(0, 1),
        Point::new(1, 1),
        Point::new(2, 1),
        Point::new(1, 2),
    ]
    .iter()
    {
        let idx = linear_idx(*offset);
        subpixels[idx..idx + 3].copy_from_slice(&rgb);
    }

    // corners
    for offset in [
        Point::new(0, 0),
        Point::new(0, 2),
        Point::new(2, 0),
        Point::new(2, 2),
    ]
    .iter()
    {
        if !sparkle || rng.gen::<bool>() {
            let idx = linear_idx(*offset);
            subpixels[idx..idx + 3].copy_from_slice(&rgb);
        }
    }
}

// each light is 4px wide, with a 2px margin on either side
pub fn pixel_width(width: usize) -> u16 {
    ((width + 1) * 4) as u16
}

// each light is 4px high, with a 2px margin on either side
pub fn pixel_height(height: usize) -> u16 {
    ((height + 1) * 4) as u16
}

// total pixels
pub fn n_pixels_for(width: usize, height: usize) -> usize {
    pixel_width(width) as usize * pixel_height(height) as usize
}
