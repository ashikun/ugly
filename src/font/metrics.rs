//! Font metrics.

pub mod kerning;
mod layout_iter;
pub mod width;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    font,
    metrics::{
        anchor::{self, Anchor},
        Length, Point, Rect, Size,
    },
};

// We hardcode the general layout of a font texture using the following
// constants:

/// The number of columns in a font.
const NUM_COLS: u8 = 32;

/// An on-disk font metrics specification.
///
/// This is expanded into a proper metrics set before consumption.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Spec {
    /// Dimensions of one character in the font, without padding.
    ///
    /// This is also the size of one cell in the texture grid.
    pub char: Size,
    /// Dimensions of padding between characters in the font.
    pub pad: Size,
    /// Class-based width overrides for specific characters.
    ///
    /// Each override maps each of the characters in its given string to the given length.
    ///
    /// The font grid is determined by `char`, so this cannot make a character
    /// wider than `char.x`.
    #[serde(default)]
    pub width_overrides: width::Spec,
    /// Class-based kerning for specific characters.
    #[serde(default)]
    pub kerning: kerning::Spec,
}

impl Spec {
    /// Expands this metrics spec into a full metrics set.
    ///
    /// This precomputes width overrides.
    ///
    /// # Errors
    ///
    /// Fails if the metrics spec is ill-formed (eg, a width override tries to make a character
    /// longer than its grid width).
    pub fn into_metrics(self) -> super::Result<Metrics> {
        Ok(Metrics {
            char: self.char,
            pad: self.pad,
            widths: self.width_overrides.into_map(self.char.w)?,
            kerning: self.kerning.into_map(self.pad.w)?,
        })
    }
}

/// A font metrics set.
///
/// The default metrics set has everything set to zero, and is useless for anything other than
/// preventing a panic or hard error if font metrics are missing.
#[derive(Clone, Debug, Default)]
pub struct Metrics {
    /// Height of one character in the font, without padding.
    pub char: Size,
    /// Dimensions of padding between characters in the font.
    pub pad: Size,
    /// Proportional character width map.
    pub widths: width::Map,
    /// Kerning map.
    pub kerning: kerning::Map,
}

impl Metrics {
    /// The padded width of one character in the font.
    #[must_use]
    pub fn padded_w(&self) -> Length {
        self.char.w + self.pad.w
    }

    /// The padded height of one character in the font.
    #[must_use]
    pub fn padded_h(&self) -> Length {
        self.char.h + self.pad.h
    }

    /// Signed maximal size of a horizontal span `size` characters wide.
    ///
    /// This is the result of multiplying `size` by the padded baseline width
    /// of the font, ignoring any kerning or proportionality adjustments.
    /// This is useful for aligning items on a character grid but may
    /// overestimate widths on proportional fonts.
    ///
    /// If `size` is negative, the result will be negative.
    #[must_use]
    pub fn span_w(&self, size: Length) -> Length {
        self.padded_w() * size
    }

    /// Like `span_w`, but accurately calculates the width of `str`.
    ///
    /// This performs the same positioning calculations as text rendering, and
    /// is accurate in the face of any proportionality in the font.
    #[must_use]
    pub fn span_w_str(&self, str: &str) -> Length {
        // Pretend to lay out the string, then work out where the last character went.
        self.layout_str(Point::default(), str)
            .last()
            .map_or(0, |glyph| {
                glyph.dst.x(0, super::super::metrics::anchor::X::Right)
            })
    }

    /// Like `span_w`, but calculates the width of `c` including any proportionality adjustments.
    #[must_use]
    pub fn span_w_char(&self, c: char) -> Length {
        self.widths.get(c)
    }

    /// Calculates the relative X-coordinate of `anchor` within `str`.
    #[must_use]
    pub fn x_anchor_of_str(&self, str: &str, anchor: anchor::X) -> Length {
        // No need to do layout calculations if we're already at the left.
        if matches!(anchor, anchor::X::Left) {
            0
        } else {
            anchor.offset(self.span_w_str(str))
        }
    }

    /// Signed maximal size of a vertical span `size` characters tall.
    ///
    /// This is the result of multiplying `size` by the padded baseline height
    /// of the font.
    ///
    /// If `size` is negative, the result will be negative.
    #[must_use]
    pub fn span_h(&self, size: Length) -> Length {
        self.padded_h() * size
    }

    /// Converts a size in chars into a size in pixels.
    #[must_use]
    pub fn text_size(&self, w_chars: Length, h_chars: Length) -> Size {
        Size {
            w: self.span_w(w_chars),
            h: self.span_h(h_chars),
        }
    }

    /// Calculates layout for a byte-string as a series of [Glyph]s.
    pub fn layout_str<'a, S: AsRef<str> + ?Sized>(
        &'a self,
        start: Point,
        string: &'a S,
    ) -> impl Iterator<Item = Glyph> + 'a {
        layout_iter::LayoutIter::new(self, start, string)
    }

    /// Bounding box for a glyph in the texture.
    #[must_use]
    fn glyph_rect(&self, char: char) -> Rect {
        self.glyph_top_left(char)
            .to_rect(self.glyph_size(char), Anchor::TOP_LEFT)
    }

    /// The top-left position of the glyph for `char` in the font.
    #[must_use]
    fn glyph_top_left(&self, char: char) -> Point {
        // TODO(@MattWindsor91): glyph atlasing for >ASCII characters
        char_to_ascii(char).map_or_else(Point::default, |g| Point {
            x: glyph_axis(glyph_col(g), self.padded_w()),
            y: glyph_axis(glyph_row(g), self.padded_h()),
        })
    }

    /// The size of the glyph for `char` in the font.
    #[must_use]
    fn glyph_size(&self, c: char) -> Size {
        Size {
            w: self.widths.get(c),
            h: self.char.h,
        }
    }
}

/// Calculates one axis of the top-left of the glyph.
#[must_use]
fn glyph_axis(index: u8, size: Length) -> Length {
    // Can't multiply _then_ convert, because of overflow on big fonts.
    Length::from(index) * size
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph.
    pub dst: Rect,
}

/// The column of a glyph in the font.
#[must_use]
pub fn glyph_col(char: u8) -> u8 {
    char % NUM_COLS
}

/// The row of a glyph in the font.
#[must_use]
pub fn glyph_row(char: u8) -> u8 {
    char / NUM_COLS
}

/// Shorthand for a hashmap of metrics.
pub type Map<FId> = HashMap<FId, Metrics>;

/// Loads a map of metrics from a map of paths.
///
/// # Errors
///
/// Fails if a metrics file isn't present, or is malformed, et cetera.
pub fn load_map<FId: super::Id>(paths: &font::Map<FId>) -> super::Result<Map<FId>> {
    paths.iter().map(|(k, v)| Ok((*k, v.metrics()?))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big_font() -> Metrics {
        Spec {
            char: Size { w: 9, h: 9 },
            pad: Size { w: 1, h: 1 },
            width_overrides: [("iI", 1)].into_iter().collect(),
            kerning: kerning::Spec::default(),
        }
        .into_metrics()
        .expect("should not fail to expand metrics")
    }

    /// Tests that the X co-ordinate of `glyph_top_left` works correctly without
    /// overflow on a big bitmap.
    #[test]
    fn glyph_x_overflow() {
        assert_eq!(big_font().glyph_top_left(char::from(31)).x, 310);
    }

    /// Tests that the Y co-ordinate of `glyph_top_left` works correctly without
    /// overflow on a big bitmap.
    #[test]
    fn glyph_y_overflow() {
        assert_eq!(big_font().glyph_top_left(char::from(255)).y, 70);
    }

    /// Tests that `span_w_str` appears to handle overrides properly.
    #[test]
    fn span_w_str_overrides() {
        // 3*9 normal + 2*1 overrides + 4*1 padding
        assert_eq!(big_font().span_w_str("Icing"), 33)
    }
}

fn char_to_ascii(c: char) -> Option<u8> {
    u8::try_from(c).ok()
}
