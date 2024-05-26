//! Layout algorithm for strings.

use super::{
    super::metrics::{Anchor, Length, Point, Rect, Size},
    metrics::chars::Entry,
    Metrics,
};
use std::collections::HashMap;

/// A laid-out string.
///
/// The default [String] is empty and has no glyphs.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct String {
    /// The string that has been laid out.
    pub string: std::string::String,
    /// The bounding box.
    pub bounds: Rect,
    /// The positions of each glyph, grouped by character.
    pub glyphs: HashMap<char, Glyph>,
}

impl String {
    /// Lays out a string `string` at `top_left`, using `metrics`.
    ///
    /// The resulting [String] will take ownership of `string`.
    #[must_use]
    pub fn layout(metrics: &Metrics, string: std::string::String, mut top_left: Point) -> String {
        // TODO(@MattWindsor91): newlines
        if string.is_empty() {
            // No characters in the string.
            return String::default();
        };

        let len = string.len();

        let mut result = String {
            string,
            bounds: Rect {
                top_left,
                size: Size::default(),
            },
            glyphs: HashMap::with_capacity(len), // maybe every character is different
        };

        let mut char_metrics = &Entry::default();

        for char in result.string.chars() {
            // Adjust for the previous character's metrics.
            // On the first iteration, this will just move by 0.
            let kerning = char_metrics.kerning(char);
            top_left.offset_mut(char_metrics.width + kerning, 0);
            result.bounds.size.w += kerning;

            // Now load in this char's metrics.
            char_metrics = &metrics.chars[char];
            let size = Size {
                w: char_metrics.width,
                h: metrics.char.h,
            };

            result.bounds.size = result.bounds.size.stack_horizontally(size);
            result
                .glyphs
                .entry(char)
                .and_modify(|g| g.dsts.push(top_left))
                .or_insert_with(|| {
                    let src_top_left = metrics.glyph_top_left(char);
                    let src = src_top_left.to_rect(size, Anchor::TOP_LEFT);
                    Glyph {
                        src,
                        dsts: vec![top_left],
                    }
                });
        }

        result
    }

    /// Moves each glyph in the layout by `dx` to the right and `dx` down.
    pub fn offset_mut(&mut self, dx: Length, dy: Length) {
        // No need to traverse if we aren't offsetting by anything.
        if dx == 0 && dy == 0 {
            return;
        }

        for g in &mut self.glyphs.values_mut() {
            for dst in &mut g.dsts {
                dst.offset_mut(dx, dy);
            }
        }
    }
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph (top-left points, assuming the size is the same as `src`).
    pub dsts: Vec<Point>,
}
