//! Layout algorithm for strings.

use super::{
    super::metrics::{Length, Point, Rect},
    metrics::kerning::LeftMap,
    Metrics,
};

pub struct String<'str> {
    /// The string that has been laid out.
    pub string: &'str str,
    /// The positions of each glyph.
    pub glyphs: Vec<Glyph>,
}

impl<'str> String<'str> {
    /// Lays out a string `string` at `top_left`, using `metrics`.
    pub fn layout(
        metrics: &Metrics,
        string: &'str (impl AsRef<str> + ?Sized),
        mut top_left: Point,
    ) -> String<'str> {
        // TODO(@MattWindsor91): newlines

        let str = string.as_ref();
        let mut result = String {
            string: str,
            glyphs: Vec::with_capacity(str.len()), // assuming best-case: ASCII
        };

        let mut last_char = LastChar::default();

        for char in str.chars() {
            // On the first iteration, this will just move by 0.
            top_left.offset_mut(last_char.width + last_char.kerning.get(char), 0);

            let src_rect = metrics.glyph_rect(char);
            last_char.width = src_rect.size.w;
            last_char.kerning = metrics.kerning.for_left(char);

            let dst = Rect {
                top_left,
                ..src_rect
            };
            result.glyphs.push(Glyph { src: src_rect, dst });
        }

        result
    }
}

/// A representation of a glyph and where to find it in the texture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LastChar<'met> {
    /// The last character's width.
    width: Length,
    /// The kerning information for the last character.
    kerning: LeftMap<'met>,
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph.
    pub dst: Rect,
}
