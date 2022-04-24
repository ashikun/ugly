//! Mid-level text composition interface.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{marker, mem};

use crate::{
    colour, error, font, metrics, render,
    resource::{self, Map},
};

/// A formatted text renderer.
///
/// This type serves both as a builder for laying out and writing strings, as well as a basic cache
/// method: the writer will not produce a
#[derive(Default)]
pub struct Writer<Font: font::Map, Fg: resource::Map<colour::Definition>, Bg> {
    /// The user-supplied options.
    pub options: Options<Font::Id, Fg::Id>,

    /// The string currently being built inside this writer.
    current_str: String,

    /// The most recently laid-out string.
    layout: font::layout::String,

    /// The last (string, options) hash.
    last_hash: Option<u64>,

    bg_phantom: marker::PhantomData<Bg>,
}

/// The set of user-specifiable options on the writer.
///
/// These can be written to and read from at will.
#[derive(Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Options<FId, FgId> {
    /// The point used as the anchor for the writing.
    pub pos: metrics::Point,

    /// The alignment for the writing.
    pub alignment: metrics::anchor::X,

    /// The specification of the font being used for writing.
    pub font_spec: font::Spec<FId, FgId>,
}

impl<Font, Fg, Bg> Writer<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    /// Constructs a writer, using the given font metrics.
    ///
    /// The writer initially points to the origin and uses a left anchor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: Options::default(),
            current_str: String::default(),
            layout: font::layout::String::default(),
            bg_phantom: marker::PhantomData::default(),
            last_hash: None,
        }
    }

    /// Renders the most recently `layout`-ed string.
    ///
    /// # Errors
    ///
    /// Fails if the renderer can't blit glyphs to the screen.
    pub fn render<R: render::Renderer<Font, Fg, Bg>>(&self, r: &mut R) -> error::Result<()> {
        r.write(self.options.font_spec, &self.layout)
    }

    /// Lays out the current string, consuming it in the process.
    ///
    /// Subsequent calls to the writing functions will now build a new string.
    ///
    /// If the string and parameters are the same as the last time this renderer was used, there
    /// will not be a full layout calculation.  This means it is useful to reuse writers across
    /// frames.
    pub fn layout(&mut self, metrics: &Font::MetricsMap) {
        // TODO(@MattWindsor91): don't hash on colour; it doesn't affect layout.
        let mut hasher = DefaultHasher::new();
        (&self.current_str, &self.options).hash(&mut hasher);
        let hash = hasher.finish();

        let str = mem::take(&mut self.current_str);
        if self.last_hash.replace(hash) != Some(hash) {
            self.actually_layout(metrics, str);
        }
    }

    /// Lays out `str` using `metrics`.
    fn actually_layout(&mut self, metrics: &Font::MetricsMap, str: String) {
        let fm = metrics.get(self.options.font_spec.id);

        self.layout = font::layout::String::layout(fm, str, self.options.pos);
        self.align_layout();
    }

    /// Adjusts the string layout if this is not left-aligned text.
    fn align_layout(&mut self) {
        if matches!(self.options.alignment, metrics::anchor::X::Left) {
            return;
        }
        self.layout.offset_mut(
            self.options.alignment.offset(self.layout.bounds().size.w),
            0,
        );
    }
}

/// We can use writers with Rust's formatting system.
///
/// This does not directly render to the screen, but instead concatenates onto the current string
/// waiting to be laid out.
impl<Font, Fg, Bg> std::fmt::Write for Writer<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.current_str.push_str(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        render::{Command, Logger, Renderer},
        resource::DefaultingHashMap,
    };
    use std::collections::HashMap;
    use std::fmt::Write;

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

        let mut r: Logger<
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
            DefaultingHashMap<(), _>,
        > = Logger {
            log: vec![],
            metrics,
        };

        let tl1 = metrics::Point { x: 20, y: 10 };

        r.clear(()).unwrap();
        // Testing repeated cached layouting.
        for _ in 0..2 {
            writer.options.pos = tl1;
            writer.write_str("hell").unwrap();
            writer.write_str("o, w").unwrap();
            writer.write_str("orld").unwrap();
            writer.layout(r.font_metrics());

            writer.render(&mut r).unwrap();
        }
        r.present();

        for c in r.log.drain(0..) {
            if let Command::Write(_, s) = c {
                assert_eq!(s.string, "hello, world");
                assert_eq!(s.bounds().top_left, tl1);
            }
        }

        // Now we're moving and renaming, which will invalidate the cache.
        let tl2 = metrics::Point { x: 10, y: 20 };
        writer.options.pos = tl2;
        writer.write_str("how's it going?").unwrap();
        writer.layout(r.font_metrics());

        r.clear(()).unwrap();
        writer.render(&mut r).unwrap();
        r.present();

        for c in r.log.drain(0..) {
            if let Command::Write(_, s) = c {
                assert_eq!(s.string, "how's it going?");
                assert_eq!(s.bounds().top_left, tl2);
            }
        }
    }
}
