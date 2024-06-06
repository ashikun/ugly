//! Layout algorithm for strings.

use std::collections::HashMap;

use super::{
    super::metrics::{anchor, point, Length, Rect, Size},
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
    /// The positions of each glyph.
    pub glyphs: GlyphSet,
}

/// The set of glyph positions (source and destination) making up a string.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct GlyphSet {
    glyphs: HashMap<Rect, Vec<point::Delta>>,
}

impl GlyphSet {
    fn iter(&self) -> GlyphIter<'_> {
        self.glyphs
            .iter()
            .map(|(src, dsts)| Glyph { src: *src, dsts })
    }
}

type GlyphIter<'a> = std::iter::Map<
    std::collections::hash_map::Iter<'a, Rect, Vec<point::Delta>>,
    fn((&'a Rect, &'a Vec<point::Delta>)) -> Glyph<'a>,
>;

impl<'a> IntoIterator for &'a GlyphSet {
    type Item = Glyph<'a>;
    type IntoIter = GlyphIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl GlyphSet {
    fn push(&mut self, src: Rect, delta: point::Delta) {
        let dsts = self.glyphs.entry(src).or_default();
        dsts.push(delta);
    }

    fn extend(&mut self, src: Rect, deltas: impl IntoIterator<Item = point::Delta>) {
        let dsts = self.glyphs.entry(src).or_default();
        dsts.extend(deltas);
    }

    fn merge(&mut self, other: GlyphSet) {
        for (src, dsts) in other.glyphs {
            self.extend(src, dsts);
        }
    }

    fn realign(&mut self, alignment: anchor::X, line_width: Length, total_width: Length) {
        /* How much do we need to shift things to the right?
         * The `offset` effectively calculates the amount that can be found on the left of a width
         * if we balance it on the anchor point, so, by finding the gap we need to fill in between
         * the line and the total bounds, we can work out how much more needs to be on the left of
         * the line.
         */
        let dw = alignment.offset(total_width - line_width);

        if dw == 0 {
            // No need to realign in this case.
            return;
        }

        for delta in self.glyphs.values_mut().flat_map(|g| g.iter_mut()) {
            delta.dx += dw;
        }
    }
}

/// A string layout builder.
pub struct Builder<'a> {
    font_metrics: &'a Metrics,
    bounds: Rect,
    padded_h: Length,

    //
    // User settings
    //
    alignment: anchor::X,

    /// The cursor, as an offset on the top-left of the string layout.
    cursor: point::Delta,
    /// The metrics of the last character.
    last_char_metrics: Option<&'a chars::Entry>,

    finished_lines: Vec<Line>,
    current_line: Line,
}

impl<'a> Builder<'a> {
    /// Constructs a new layout builder with the given font metrics.
    #[must_use]
    pub fn new(font_metrics: &'a Metrics) -> Self {
        Self {
            bounds: Rect::default(),
            font_metrics,
            padded_h: font_metrics.padded_h(),
            alignment: anchor::X::default(),
            cursor: point::Delta::default(),
            last_char_metrics: None,
            current_line: Line {
                size: Size {
                    w: 0,
                    h: font_metrics.char.h,
                },
                glyphs: GlyphSet::default(),
            },
            finished_lines: vec![],
        }
    }

    /// Changes the alignment of the layout.
    #[must_use]
    pub fn with_alignment(mut self, alignment: anchor::X) -> Self {
        self.alignment = alignment;
        self
    }

    /// Builds the layout for a given string.
    #[must_use]
    pub fn build(mut self, string: std::string::String) -> String {
        if string.is_empty() {
            // No characters in the string.
            return String::default();
        };

        self.do_layout(&string);

        let mut glyphs = GlyphSet::default();
        for line in self.finished_lines {
            let mut line_glyphs = line.glyphs;
            line_glyphs.realign(self.alignment, line.size.w, self.bounds.size.w);
            glyphs.merge(line_glyphs);
        }

        String {
            string,
            bounds: self.bounds,
            glyphs,
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

        // Implicit newline at the end to tidy things up:
        self.line_feed();
    }

    fn carriage_return(&mut self) {
        self.cursor.dx = 0;
        self.last_char_metrics = None;
    }

    fn line_feed(&mut self) {
        if self.current_line.size.w == 0 {
            return;
        }

        self.cursor.dx = 0;
        self.cursor.dy += self.padded_h;

        self.bounds.size = self.bounds.size.stack_vertically(self.current_line.size);
        self.last_char_metrics = None;

        let line = std::mem::take(&mut self.current_line);
        self.finished_lines.push(line);

        // Any future lines will have padding from the line above.
        self.current_line.size.h = self.padded_h;
    }

    fn layout_char(&mut self, char: char) {
        let char_metrics = &self.font_metrics.chars[char];
        self.current_line.size.w += char_metrics.width;

        if let Some(metrics) = self.last_char_metrics.replace(char_metrics) {
            self.move_right_with_kerning(char, metrics);
        }

        let src = self.char_src_rect(char, char_metrics);
        self.current_line.glyphs.push(src, self.cursor);
    }

    fn move_right_with_kerning(&mut self, char: char, metrics: &chars::Entry) {
        let kerning = metrics.kerning(char);
        self.cursor.dx += metrics.width + kerning;
        self.current_line.size.w += kerning;
    }

    fn char_src_rect(&self, char: char, metrics: &chars::Entry) -> Rect {
        // TODO: cache
        let src_top_left = self.font_metrics.glyph_top_left(char);
        let size = Size {
            w: metrics.width,
            h: self.font_metrics.char.h,
        };
        src_top_left.to_rect(size, anchor::Anchor::TOP_LEFT)
    }
}

#[derive(Clone, Debug, Default)]
struct Line {
    /// The size, including any padding from the previous line.
    size: Size,
    glyphs: GlyphSet,
}

/// A representation of a glyph to be rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph<'a> {
    /// The glyph's source rectangle.
    pub src: Rect,
    /// Where to render the glyph (as a delta against the top-left points, assuming the size is the same as `src`).
    pub dsts: &'a [point::Delta],
}
