//! Iterator for producing a glyph layout.

use std::str::Chars;

use super::{
    super::metrics::{Point, Rect},
    Glyph, Metrics,
};

/// Iterator for laying out characters in a string.
///
/// This iterator takes a font metrics set and an iterator over characters, and produces a sequence
/// of `Glyph`s.
pub(super) struct LayoutIter<'met, 'str> {
    /// Ingress iterator for characters.
    chars: Chars<'str>,
    /// Information about the last character, used for spacing and kerning the next character.
    last_char: Option<GlyphSrc>,
    /// Current top-left position.
    top_left: Point,
    /// Font metrics, used for spacing and kerning.
    metrics: &'met Metrics,
}

/// A representation of a glyph and where to find it in the texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GlyphSrc {
    /// The character to be rendered.
    pub char: char,
    /// The glyph's location inside the texture map.
    pub rect: Rect,
}

impl<'met, 'str> Iterator for LayoutIter<'met, 'str> {
    type Item = Glyph;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.next().map(|x| self.layout_char(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }

    fn count(self) -> usize {
        self.chars.count()
    }
}

impl<'met, 'str> LayoutIter<'met, 'str> {
    pub(super) fn new(
        metrics: &'met Metrics,
        top_left: Point,
        str: &'str (impl AsRef<str> + ?Sized),
    ) -> Self {
        Self {
            chars: str.as_ref().chars(),
            last_char: None,
            top_left,
            metrics,
        }
    }

    fn layout_char(&mut self, char: char) -> Glyph {
        let src = GlyphSrc {
            char,
            rect: self.metrics.glyph_rect(char),
        };

        if let Some(old_src) = self.last_char.replace(src) {
            self.top_left.offset_mut(
                old_src.rect.size.w + self.metrics.kerning.spacing(old_src.char, char),
                0,
            );
        }

        let dst = Rect {
            top_left: self.top_left,
            ..src.rect
        };
        Glyph { src: src.rect, dst }
    }
}
