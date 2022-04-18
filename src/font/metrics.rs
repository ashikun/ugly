//! Font metrics.

pub mod chars;
pub mod kerning;
pub mod width;

use crate::font::layout;
use serde::{Deserialize, Serialize};

use crate::metrics::{anchor, Length, Point, Size};

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
            chars: chars::Table::new(self.width_overrides, self.char.w, self.kerning, self.pad.w)?,
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
    /// Map of characters to their width and kerning information.
    pub chars: chars::Table,
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
        layout::String::layout(self, str, Point::default())
            .glyphs
            .last()
            .map_or(0, |glyph| {
                glyph.dst.x(0, super::super::metrics::anchor::X::Right)
            })
    }

    /// Like `span_w`, but calculates the width of `c` including any proportionality adjustments.
    #[must_use]
    pub fn span_w_char(&self, c: char) -> Length {
        self.chars[c].width
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

    /// The top-left position of the glyph for `char` in the font.
    ///
    /// Getting the glyph size requires a combination of consulting the character map (for width)
    /// and this metrics structure (for height).  Since things that want the glyph top-left
    /// tend to have their own copy of the character map information by the time they call
    /// `glyph_top_left`, we don't expose any duplicate way of getting the size.
    #[must_use]
    pub fn glyph_top_left(&self, char: char) -> Point {
        // TODO(@MattWindsor91): glyph atlasing for >ASCII characters
        char_to_ascii(char).map_or_else(Point::default, |g| Point {
            x: glyph_axis(glyph_col(g), self.padded_w()),
            y: glyph_axis(glyph_row(g), self.padded_h()),
        })
    }
}

/// Calculates one axis of the top-left of the glyph.
#[must_use]
fn glyph_axis(index: u8, size: Length) -> Length {
    // Can't multiply _then_ convert, because of overflow on big fonts.
    Length::from(index) * size
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
