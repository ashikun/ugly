//! Layout algorithm for strings.

use std::collections::HashMap;

use super::{
    super::metrics::{Anchor, Length, Point, Rect, Size},
    metrics::chars,
    Metrics,
};

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

/// A string layout builder.
pub struct Builder<'a> {
    font_metrics: &'a Metrics,
    bounds: Rect,
    glyphs: HashMap<char, Glyph>,
    padded_h: Length,

    /// The cursor, relative to the top-left of the string layout.
    cursor: Point,
    /// The metrics of the last character.
    last_char_metrics: Option<&'a chars::Entry>,
}

impl<'a> Builder<'a> {
    /// Constructs a new layout builder with the given font metrics.
    #[must_use]
    pub fn new(font_metrics: &'a Metrics) -> Self {
        Self {
            bounds: Rect::default(),
            font_metrics,
            padded_h: font_metrics.padded_h(),
            glyphs: HashMap::new(),
            cursor: Point::default(),
            last_char_metrics: None,
        }
    }

    /// Builds the layout for a given string.
    #[must_use]
    pub fn build(mut self, string: std::string::String) -> String {
        if string.is_empty() {
            // No characters in the string.
            return String::default();
        };

        self.do_layout(&string);

        String {
            string,
            bounds: self.bounds,
            glyphs: self.glyphs,
        }
    }

    /// Pretends to lay out a given string, but only retrieves the bounds.
    #[must_use]
    pub fn dry_run(mut self, string: &str) -> Rect {
        // TODO: disable glyph storage?
        self.do_layout(string);
        self.bounds
    }

    fn do_layout(&mut self, string: &str) {
        for char in string.chars() {
            match char {
                '\r' => self.carriage_return(),
                '\n' => self.line_feed(),
                c => self.layout_char(c),
            }
        }
    }

    fn carriage_return(&mut self) {
        self.cursor.x = 0;
        self.last_char_metrics = None;
    }

    fn line_feed(&mut self) {
        self.cursor.x = 0;
        self.cursor.y += self.padded_h;
        self.bounds.size.h += self.padded_h;
        self.last_char_metrics = None;
    }

    fn layout_char(&mut self, char: char) {
        let char_metrics = &self.font_metrics.chars[char];
        self.bounds.size.w += char_metrics.width;

        if let Some(metrics) = self.last_char_metrics.replace(char_metrics) {
            self.move_right_with_kerning(char, metrics);
        }

        self.push_glyph(char, char_metrics);
    }

    fn move_right_with_kerning(&mut self, char: char, metrics: &chars::Entry) {
        let kerning = metrics.kerning(char);
        self.cursor.x += metrics.width + kerning;
        self.bounds.size.w += kerning;
    }

    fn push_glyph(&mut self, char: char, metrics: &chars::Entry) {
        let glyph = self.glyphs.entry(char).or_insert_with(|| {
            let src_top_left = self.font_metrics.glyph_top_left(char);
            let size = Size {
                w: metrics.width,
                h: self.font_metrics.char.h,
            };
            let src = src_top_left.to_rect(size, Anchor::TOP_LEFT);

            Glyph { src, dsts: vec![] }
        });

        let dst = GlyphDst { delta: self.cursor };
        glyph.dsts.push(dst);
    }
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph (top-left points, assuming the size is the same as `src`).
    pub dsts: Vec<GlyphDst>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlyphDst {
    /// The delta from the string's top-left.
    pub delta: Point,
}
