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

struct Builder<'a> {
    font_metrics: &'a Metrics,
    bounds: Rect,
    glyphs: HashMap<char, Glyph>,
    padded_h: Length,

    cursor: Point,
    last_char_metrics: Option<&'a Entry>,
}

impl<'a> Builder<'a> {
    fn new(font_metrics: &'a Metrics, top_left: Point) -> Self {
        let bounds = Rect {
            top_left,
            size: Size::default(),
        };

        Self {
            bounds,
            glyphs: HashMap::new(),
            cursor: top_left,
            last_char_metrics: None,
            font_metrics,
            padded_h: font_metrics.padded_h(),
        }
    }

    fn build(mut self, string: std::string::String) -> String {
        for char in string.chars() {
            match char {
                '\r' => self.carriage_return(),
                '\n' => self.line_feed(),
                c => self.layout_char(c),
            }
        }

        String {
            string,
            bounds: self.bounds,
            glyphs: self.glyphs,
        }
    }

    fn carriage_return(&mut self) {
        self.cursor.x = self.bounds.top_left.x;
    }

    fn line_feed(&mut self) {
        self.cursor.x = self.bounds.top_left.x;
        self.cursor.y += self.padded_h;
        self.bounds.size.h += self.padded_h;
    }

    fn layout_char(&mut self, char: char) {
        let char_metrics = &self.font_metrics.chars[char];

        if let Some(metrics) = self.last_char_metrics.replace(char_metrics) {
            self.move_right_with_kerning(char, metrics);
        }

        let size = Size {
            w: char_metrics.width,
            h: self.font_metrics.char.h,
        };

        self.bounds.size = self.bounds.size.stack_horizontally(size);
        self.push_glyph(char, size);
    }

    fn move_right_with_kerning(&mut self, char: char, metrics: &Entry) {
        let kerning = metrics.kerning(char);
        self.cursor.offset_mut(metrics.width + kerning, 0);
        self.bounds.size.w += kerning;
    }

    fn push_glyph(&mut self, char: char, size: Size) {
        self.glyphs
            .entry(char)
            .and_modify(|g| g.dsts.push(self.cursor))
            .or_insert_with(|| {
                let src_top_left = self.font_metrics.glyph_top_left(char);
                let src = src_top_left.to_rect(size, Anchor::TOP_LEFT);
                Glyph {
                    src,
                    dsts: vec![self.cursor],
                }
            });
    }
}

impl String {
    /// Lays out a string `string` at `top_left`, using `metrics`.
    ///
    /// The resulting [String] will take ownership of `string`.
    #[must_use]
    pub fn layout(metrics: &Metrics, string: std::string::String, top_left: Point) -> String {
        if string.is_empty() {
            // No characters in the string.
            return String::default();
        };

        Builder::new(metrics, top_left).build(string)
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
