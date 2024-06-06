//! Mid-level text composition interface.

use std::mem;

use crate::{error, font, metrics, render, resource::Map};

/// A formatted text renderer.
///
/// This type serves both as a builder for laying out and writing strings, as well as a basic cache
/// method: the writer will try to minimise re-layouting and font acquisition when strings and
/// options change.
#[derive(Debug, Clone)]
pub struct Writer<FontId, FgId> {
    /// The point used as the anchor for the writing.
    pos: metrics::Point,

    /// The alignment for the writing.
    alignment: metrics::anchor::X,

    /// The font being used for writing.
    pub(crate) font: FontId,

    /// The foreground colour being used for writing.
    fg: FgId,

    /// The most recently laid-out string.
    layout: font::layout::String,

    /// Can we reuse the last computed layout?
    layout_reusable: bool,
}

impl<FontId, FgId> Default for Writer<FontId, FgId>
where
    FontId: Default,
    FgId: Default,
{
    fn default() -> Self {
        Self::new(FontId::default(), FgId::default())
    }
}

impl<FontId, FgId> Writer<FontId, FgId> {
    /// Constructs a new writer with the given font and colours.
    pub fn new(font: FontId, fg: FgId) -> Self {
        Self {
            pos: metrics::Point::default(),
            alignment: metrics::anchor::X::Left,
            font,
            fg,
            layout: font::layout::String::default(),
            layout_reusable: false,
        }
    }
}

impl<FontId, FgId> Writer<FontId, FgId>
where
    FontId: Copy,
{
    /// Lays out the current string.
    ///
    /// If the string and parameters are the same as the last time this renderer was used, there
    /// will not be a full layout calculation.  This means it is useful to reuse writers across
    /// frames.
    pub fn layout(&mut self, metrics: &impl Map<font::Metrics, Id = FontId>) {
        // Optimistically assume that the next time we call `layout`, everything will be the same.
        let reusable = mem::replace(&mut self.layout_reusable, true);
        if !reusable {
            self.actually_layout(metrics);
        }
    }

    /// Lays out `str` using `metrics`.
    fn actually_layout(&mut self, metrics: &impl Map<font::Metrics, Id = FontId>) {
        let fm = metrics.get(self.font);

        let current_string = mem::take(&mut self.layout);

        self.layout = font::layout::Builder::new(fm)
            .with_alignment(self.alignment)
            .build(current_string.string);
        self.reposition_layout();
    }
}

impl<FontId, FgId> Writer<FontId, FgId>
where
    FontId: Copy,
    FgId: Copy,
{
    /// Renders the most recently written string.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't blit glyphs to the screen.
    pub fn render<BgId>(
        &self,
        r: &mut impl render::Renderer<FontId, FgId, BgId>,
    ) -> error::Result<()> {
        r.write(self.font, self.fg, &self.layout)
    }
}

impl<FontId, FgId> Writer<FontId, FgId> {
    /// Gets the alignment of this writer.
    pub fn alignment(&self) -> metrics::anchor::X {
        self.alignment
    }

    /// Sets the alignment of this writer to `alignment`.
    pub fn align_to(&mut self, alignment: metrics::anchor::X) {
        if self.alignment != alignment {
            self.alignment = alignment;

            // TODO(@MattWindsor91): we should be able to reuse the layout by shifting the glyphs.
            self.layout_reusable = false;
        }
    }

    /// Gets the position of this writer.
    pub fn pos(&self) -> metrics::Point {
        self.pos
    }

    /// Sets the position of this writer to `pos`.
    pub fn move_to(&mut self, pos: metrics::Point) {
        if self.pos != pos {
            self.pos = pos;
            self.reposition_layout();
        }
    }

    /// Sets the font of this writer to `id`.
    pub fn set_font(&mut self, font: FontId) {
        self.font = font;
    }

    /// Sets the foreground colour of this writer to `fg`.
    pub fn set_fg(&mut self, fg: FgId) {
        self.fg = fg;
    }

    /// Sets the string-to-be-rendered to `str`.
    pub fn set_string(&mut self, str: &(impl ToString + ?Sized)) {
        // Store the new string inside the layout; we'll recompute the rest in a bit.
        let old_str = mem::replace(&mut self.layout.string, str.to_string());
        // The layout needs to be junked if the string has changed.
        self.layout_reusable &= old_str == self.layout.string;
    }

    /// Moves the string layout to the correct position.
    fn reposition_layout(&mut self) {
        self.layout.bounds.top_left = self.pos;

        // No point doing offsets if the anchor is left; the offset would be 0.
        if let metrics::anchor::X::Left = self.alignment {
            return;
        }

        // `self.alignment.offset` is the number of pixels between the left and the anchor, so we
        // need to move so that the position (which is currently the left) is *on* that anchor.
        // This means the offset must be backwards.
        self.layout.bounds.top_left.x -= self.alignment.offset(self.layout.bounds.size.w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        render::{logger, Renderer},
        resource::DefaultingHashMap,
    };
    use std::collections::HashMap;

    #[test]
    fn layout_hello_world() {
        let mut met = font::Metrics::default();
        met.char.w = 8;
        met.char.h = 14;

        let metrics = DefaultingHashMap::new(HashMap::<(), _>::new(), met);

        let mut writer = Writer::<(), ()>::default();

        let mut r: logger::Logger<(), (), ()> = logger::Logger::default();

        let tl1 = metrics::Point { x: 20, y: 10 };

        r.clear(()).unwrap();
        // Testing repeated cached layouting.
        for _ in 0..2 {
            writer.move_to(tl1);
            writer.set_string("hello, world");
            writer.layout(&metrics);

            writer.render(&mut r).unwrap();
        }
        r.present();

        for c in r.log.drain(0..) {
            if let logger::Command::Write(_, _, s) = c {
                assert_eq!(s.string, "hello, world");
                assert_eq!(s.bounds.top_left, tl1);
            }
        }

        // Now we're moving and renaming, which will invalidate the cache.
        let tl2 = metrics::Point { x: 10, y: 20 };
        writer.move_to(tl2);
        writer.set_string("how's it going?");
        writer.layout(&metrics);

        r.clear(()).unwrap();
        writer.render(&mut r).unwrap();
        r.present();

        for c in r.log.drain(0..) {
            if let logger::Command::Write(_, _, s) = c {
                assert_eq!(s.string, "how's it going?");
                assert_eq!(s.bounds.top_left, tl2);
            }
        }
    }
}
