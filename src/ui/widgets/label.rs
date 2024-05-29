//! Label widgets.

use std::hash::Hash;

use crate::{metrics, resource::Map, text::Writer, Renderer, Result};

use super::super::{
    layout::{Boundable, LayoutContext, Layoutable},
    render::Renderable,
    update::Updatable,
};

/// A widget that displays a static single-line string with a static font.
///
/// `FontId`, `FgId`, and `BgId` are the usual font and colour ID types.
#[derive(Clone)]
pub struct Label<FontId, FgId, BgId> {
    /// The most recently computed bounding box for the label.
    bounds: metrics::Rect,

    /// The writer for the label.
    writer: Writer<FontId, FgId>,

    /// The background colour, if any.
    bg: Option<BgId>,

    /// The minimum amount of expected characters in the label.
    pub min_chars: u8,
}

impl<FontId, FgId, BgId> Label<FontId, FgId, BgId> {
    /// Constructs a label over the given writer.
    #[must_use]
    pub fn new(writer: Writer<FontId, FgId>) -> Self {
        Self {
            bounds: metrics::Rect::default(),
            writer,
            bg: None,
            min_chars: 0,
        }
    }

    /// Sets the alignment of the label.
    pub fn align_to(&mut self, alignment: metrics::anchor::X) {
        self.writer.align_to(alignment);
    }

    /// Sets the minimum character amount of the label.
    pub fn set_min_chars(&mut self, amount: u8) {
        self.min_chars = amount;
    }

    /// Sets the foreground colour of the label.
    pub fn set_fg(&mut self, fg: FgId) {
        self.writer.set_fg(fg);
    }

    /// Sets the foreground colour of the label.
    pub fn set_bg(&mut self, bg: impl Into<Option<BgId>>) {
        self.bg = bg.into();
    }

    /// Sets the font of the label.
    pub fn set_font(&mut self, font: FontId) {
        self.writer.set_font(font);
    }

    /// Converts `str` to a string then updates the label with it.
    ///
    /// Does not re-layout the string, and so the string will not be updated until `layout` is
    /// called.
    pub fn update_display(&mut self, str: impl std::fmt::Display) {
        self.writer.set_string(&str);
    }

    fn writer_pos(&self) -> metrics::Point {
        self.bounds.anchor(metrics::anchor::Anchor {
            x: self.writer.alignment(),
            y: metrics::anchor::Y::Top,
        })
    }
}

/// We can bound a label.
impl<FontId, FgId, BgId> Boundable for Label<FontId, FgId, BgId> {
    fn set_bounds(&mut self, bounds: metrics::Rect) {
        self.bounds = bounds;
        self.writer.move_to(self.writer_pos());
    }
}

/// We can layout a label, so long as the context serves font metrics for the font ID set in use.
impl<Ctx, FontId, FgId, BgId> Layoutable<Ctx> for Label<FontId, FgId, BgId>
where
    Ctx: LayoutContext<FontId>,
    FontId: Copy + Clone + Default + Eq + Hash,
{
    fn min_bounds(&self, ctx: &Ctx) -> metrics::Size {
        ctx.font_metrics()
            .get(self.writer.font)
            .text_size(i32::from(self.min_chars), 1)
    }

    fn layout(&mut self, ctx: &Ctx) {
        self.writer.layout(ctx.font_metrics());
    }
}

/// We can update a label.
impl<FontId, FgId, BgId> Updatable for Label<FontId, FgId, BgId> {
    type State = str;

    fn update(&mut self, s: &Self::State) {
        self.update_display(s);
    }
}

/// Delegates rendering to the writer.
impl<'r, FontId, FgId, BgId, R: Renderer<'r, FontId, FgId, BgId>> Renderable<R>
    for Label<FontId, FgId, BgId>
where
    FontId: Copy,
    FgId: Copy,
    BgId: Copy,
{
    fn render(&self, r: &mut R) -> Result<()> {
        if let Some(bg) = self.bg {
            r.fill(self.bounds, bg)?;
        }

        self.writer.render(r)
    }
}
