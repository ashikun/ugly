//! Mid-level text composition interface.

use std::{marker, mem};

use crate::{colour, error, font, metrics, render, resource::Map};

/// A formatted text renderer.
///
/// This type serves both as a builder for laying out and writing strings, as well as a basic cache
/// method: the writer will try to minimise re-layouting and font acquisition when strings and
/// options change.
#[derive(Default, Debug, Clone)]
pub struct Writer<Font: font::Map, Fg: Map<colour::Definition>, Bg> {
    /// The point used as the anchor for the writing.
    pos: metrics::Point,

    /// The alignment for the writing.
    alignment: metrics::anchor::X,

    /// The spec of the font being used for writing.
    font_spec: font::Spec<Font::Id, Fg::Id>,

    /// The string currently being built inside this writer.
    current_str: String,

    /// The most recently laid-out string.
    layout: font::layout::String,

    /// Phantom type for the background colour.
    bg_phantom: marker::PhantomData<Bg>,

    /// Can we reuse the last computed layout?
    layout_reusable: bool,
}

impl<Font, Fg, Bg> Writer<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    /// Constructs a writer, using the given font metrics.
    ///
    /// The writer initially points to the origin and uses a left anchor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pos: metrics::Point::default(),
            alignment: metrics::anchor::X::Left,
            font_spec: font::Spec::default(),
            current_str: String::default(),
            layout: font::layout::String::default(),
            bg_phantom: marker::PhantomData::default(),
            layout_reusable: false,
        }
    }

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

            // TODO(@MattWindsor91): we should be able to reuse the layout by shifting the glyphs.
            self.layout_reusable = false;
        }
    }

    /// Gets the font spec of this writer.
    pub fn font_spec(&self) -> &font::Spec<Font::Id, Fg::Id> {
        &self.font_spec
    }

    /// Sets the font spec of this writer to `spec`.
    pub fn set_font_spec(&mut self, spec: font::Spec<Font::Id, Fg::Id>) {
        self.font_spec = spec;
    }

    /// Sets the font of this writer to `id`.
    pub fn set_font(&mut self, id: Font::Id) {
        self.font_spec.id = id;
    }

    /// Sets the foreground colour of this writer to `fg`.
    pub fn set_fg(&mut self, fg: Fg::Id) {
        self.font_spec.colour = fg;
    }

    /// Sets the string-to-be-rendered to `str`.
    pub fn set_string(&mut self, str: &(impl ToString + ?Sized)) {
        let old_str = mem::replace(&mut self.current_str, str.to_string());
        // The layout needs to be junked if the string has changed.
        self.layout_reusable &= old_str == self.current_str;
    }

    /// Renders the most recently written string.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't blit glyphs to the screen.
    pub fn render<'f, R: render::Renderer<'f, Font, Fg, Bg>>(
        &self,
        r: &mut R,
    ) -> error::Result<()> {
        r.write(self.font_spec, &self.layout)
    }

    /// Lays out the current string, consuming it in the process.
    ///
    /// Subsequent calls to the writing functions will now build a new string.
    ///
    /// If the string and parameters are the same as the last time this renderer was used, there
    /// will not be a full layout calculation.  This means it is useful to reuse writers across
    /// frames.
    pub fn layout(&mut self, metrics: &Font::MetricsMap) {
        // Optimistically assume that the next time we call `layout`, everything will be the same.
        let reusable = mem::replace(&mut self.layout_reusable, true);
        if !reusable {
            self.actually_layout(metrics);
        }
    }

    /// Lays out `str` using `metrics`.
    fn actually_layout(&mut self, metrics: &Font::MetricsMap) {
        let fm = metrics.get(self.font_spec.id);

        self.layout = font::layout::String::layout(fm, self.current_str.clone(), self.pos);
        self.align_layout();
    }

    /// Adjusts the string layout if this is not left-aligned text.
    fn align_layout(&mut self) {
        if matches!(self.alignment, metrics::anchor::X::Left) {
            return;
        }
        self.layout
            .offset_mut(self.alignment.offset(self.layout.bounds().size.w), 0);
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

        let mut writer = Writer::<
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
        >::new();

        let mut r: logger::Logger<
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
        > = logger::Logger::new(metrics);

        let tl1 = metrics::Point { x: 20, y: 10 };

        r.clear(()).unwrap();
        // Testing repeated cached layouting.
        for _ in 0..2 {
            writer.move_to(tl1);
            writer.set_string("hello, world");
            writer.layout(r.font_metrics());

            writer.render(&mut r).unwrap();
        }
        r.present();

        for c in r.log.drain(0..) {
            if let logger::Command::Write(_, s) = c {
                assert_eq!(s.string, "hello, world");
                assert_eq!(s.bounds().top_left, tl1);
            }
        }

        // Now we're moving and renaming, which will invalidate the cache.
        let tl2 = metrics::Point { x: 10, y: 20 };
        writer.move_to(tl2);
        writer.set_string("how's it going?");
        writer.layout(r.font_metrics());

        r.clear(()).unwrap();
        writer.render(&mut r).unwrap();
        r.present();

        for c in r.log.drain(0..) {
            if let logger::Command::Write(_, s) = c {
                assert_eq!(s.string, "how's it going?");
                assert_eq!(s.bounds().top_left, tl2);
            }
        }
    }
}
