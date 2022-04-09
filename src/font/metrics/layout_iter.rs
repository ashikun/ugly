//! Iterator for producing a glyph layout.

use std::str::Chars;

use super::{
    super::metrics::{Length, Point, Rect},
    kerning::LeftMap,
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
    last_char: LastChar<'met>,
    /// Current top-left position.
    top_left: Point,
    /// Font metrics, used for spacing and kerning.
    metrics: &'met Metrics,
}

/// A representation of a glyph and where to find it in the texture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LastChar<'met> {
    /// The last character's width.
    width: Length,
    /// The kerning information for the last character.
    kerning: LeftMap<'met>,
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
            last_char: LastChar::default(),
            top_left,
            metrics,
        }
    }

    fn layout_char(&mut self, char: char) -> Glyph {
        // On the first iteration, this will just move by 0.
        self.top_left
            .offset_mut(self.last_char.width + self.last_char.kerning.get(char), 0);

        let src_rect = self.metrics.glyph_rect(char);
        self.last_char.width = src_rect.size.w;
        self.last_char.kerning = self.metrics.kerning.for_left(char);

        let dst = Rect {
            top_left: self.top_left,
            ..src_rect
        };
        Glyph { src: src_rect, dst }
    }
}
