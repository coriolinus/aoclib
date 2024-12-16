use smallstr::SmallString;
use std::{convert::TryFrom, marker::PhantomData, ops::Not};

/// Number of characters below which the [`Chunks`] iterator does not allocate.
pub const CHUNK_WIDTH: usize = 4;

/// A type implementing `DisplayWidth` has a constant width for display and parsing.
///
/// This makes it suitable for 2d cartesian maps.
pub trait DisplayWidth {
    const DISPLAY_WIDTH: usize;

    /// Split a string into an iterator of chunks of characters of length `DISPLAY_WIDTH`
    fn chunks(s: &str) -> Chunks<Self> {
        Chunks(s.chars(), PhantomData)
    }
}

/// Iterator of chunks of equal width from a string.
///
/// Created with [`DisplayWidth::chunks`]. Never heap-allocates if `T::DISPLAY_WIDTH <= CHUNK_WIDTH`.
pub struct Chunks<'a, T: ?Sized>(std::str::Chars<'a>, PhantomData<T>);

impl<T: DisplayWidth> Iterator for Chunks<'_, T> {
    // 4 bytes in a max-width char
    type Item = SmallString<[u8; 4 * CHUNK_WIDTH]>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut s = SmallString::new();
        for _ in 0..T::DISPLAY_WIDTH {
            s.push(self.0.next()?);
        }
        Some(s)
    }
}

/// A type implementing `ToRgb` can be converted to a single color.
///
/// This is useful for rendering map tiles.
pub trait ToRgb {
    fn to_rgb(&self) -> [u8; 3];
}

/// A Tile which is compatible with booleans
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    parse_display::Display,
    parse_display::FromStr,
)]
#[derive(Default)]
pub enum Bool {
    #[display("#")]
    True,
    #[display(".")]
    #[default]
    False,
}


impl DisplayWidth for Bool {
    const DISPLAY_WIDTH: usize = 1;
}

impl PartialEq<bool> for Bool {
    fn eq(&self, other: &bool) -> bool {
        (*self == Bool::True) == *other
    }
}

impl Not for Bool {
    type Output = Bool;

    fn not(self) -> Bool {
        match self {
            Bool::True => Bool::False,
            Bool::False => Bool::True,
        }
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> bool {
        match b {
            Bool::True => true,
            Bool::False => false,
        }
    }
}

impl From<bool> for Bool {
    fn from(b: bool) -> Bool {
        if b {
            Bool::True
        } else {
            Bool::False
        }
    }
}

impl ToRgb for Bool {
    fn to_rgb(&self) -> [u8; 3] {
        if (*self).into() {
            // warm white
            [253, 244, 220]
        } else {
            [0, 0, 0]
        }
    }
}

/// A Tile which contains a single digit.
///
/// Its range is `0..=9`.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    parse_display::Display,
    parse_display::FromStr,
)]
#[from_str(regex = r"(?P<0>\d)")]
pub struct Digit(u8);

impl DisplayWidth for Digit {
    const DISPLAY_WIDTH: usize = 1;
}

impl From<Digit> for u8 {
    fn from(Digit(value): Digit) -> Self {
        value
    }
}

impl TryFrom<u8> for Digit {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value < 10).then_some(Digit(value)).ok_or(())
    }
}

impl ToRgb for Digit {
    fn to_rgb(&self) -> [u8; 3] {
        const STEP: u8 = u8::MAX / 9;
        let value = self.0 * STEP;
        [value, value, value]
    }
}

/// A Tile which contains two digits.
///
/// Its range is `0..=99`.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    parse_display::Display,
    parse_display::FromStr,
)]
#[display(" {0:02}")]
#[from_str(regex = r"  ?(?P<0>\d?\d)")]
pub struct TwoDigits(u8);

impl DisplayWidth for TwoDigits {
    const DISPLAY_WIDTH: usize = 3;
}

impl From<TwoDigits> for u8 {
    fn from(TwoDigits(value): TwoDigits) -> Self {
        value
    }
}

impl TryFrom<u8> for TwoDigits {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value < 100).then_some(TwoDigits(value)).ok_or(())
    }
}

impl ToRgb for TwoDigits {
    fn to_rgb(&self) -> [u8; 3] {
        const STEP: u8 = u8::MAX / 99;
        let value = self.0 * STEP;
        [value, value, value]
    }
}
