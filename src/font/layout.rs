//! Layout algorithm for strings.

use super::{
    super::metrics::{Anchor, Point, Rect, Size},
    metrics::chars::Entry,
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

        let mut char_metrics = &Entry::default();

        for char in str.chars() {
            // Adjust for the previous character's metrics.
            // On the first iteration, this will just move by 0.
            top_left.offset_mut(char_metrics.width + char_metrics.kerning(char), 0);

            char_metrics = &metrics.chars[char];
            let src_size = Size {
                w: char_metrics.width,
                h: metrics.span_h(1),
            };
            let src_rect = metrics
                .glyph_top_left(char)
                .to_rect(src_size, Anchor::TOP_LEFT);

            let dst = Rect {
                top_left,
                ..src_rect
            };
            result.glyphs.push(Glyph { src: src_rect, dst });
        }

        result
    }
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph.
    pub dst: Rect,
}
