//! Layout algorithm for strings.

use super::{
    super::metrics::{Anchor, Length, Point, Rect, Size},
    metrics::chars::Entry,
    Metrics,
};

/// A laid-out string.
///
/// The default [String] is empty and has no glyphs.
#[derive(Default)]
pub struct String {
    /// The string that has been laid out.
    pub string: std::string::String,
    /// The positions of each glyph.
    pub glyphs: Vec<Glyph>,
}

impl String {
    /// Lays out a string `string` at `top_left`, using `metrics`.
    ///
    /// The resulting [String] will take ownership of `string`.
    #[must_use]
    pub fn layout(metrics: &Metrics, string: std::string::String, mut top_left: Point) -> String {
        // TODO(@MattWindsor91): newlines

        let len = string.len();
        let mut result = String {
            string,
            glyphs: Vec::with_capacity(len), // assuming best-case: ASCII
        };

        let mut char_metrics = &Entry::default();

        for char in result.string.chars() {
            // Adjust for the previous character's metrics.
            // On the first iteration, this will just move by 0.
            top_left.offset_mut(char_metrics.width + char_metrics.kerning(char), 0);

            char_metrics = &metrics.chars[char];
            let src_size = Size {
                w: char_metrics.width,
                h: metrics.char.h,
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

    /// Calculates the bounds of this string.
    ///
    /// This is the rectangle formed by the top-left of the first character and the bottom-right
    /// of the last character.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        let tl = self
            .glyphs
            .first()
            .map(|x| x.src.top_left)
            .unwrap_or_default();
        let br = self
            .glyphs
            .last()
            .map(|x| x.src.anchor(Anchor::BOTTOM_RIGHT))
            .unwrap_or_default();

        Rect::from_points(tl, br)
    }

    /// Moves each glyph in the layout by `dx` to the right and `dx` down.
    pub fn offset_mut(&mut self, dx: Length, dy: Length) {
        // No need to traverse if we aren't offsetting by anything.
        if dx == 0 && dy == 0 {
            return;
        }

        for g in &mut self.glyphs {
            g.dst.top_left.offset_mut(dx, dy);
        }
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
