//! Mid-level text composition interface.

use std::marker;

use crate::{
    colour, error, font, metrics, render,
    resource::{self, Map},
};

/// Helper for positioned writing of strings.
pub struct Writer<'r, Font: font::Map, Fg: resource::Map<colour::Definition>, Bg, R> {
    /// The point used as the anchor for the writing.
    pos: metrics::Point,

    /// The alignment for the writing.
    alignment: metrics::anchor::X,

    /// The specification of the font being used for writing.
    font_spec: font::Spec<Font::Id, Fg::Id>,

    /// Reference to the renderer being borrowed to do the rendering.
    renderer: &'r mut R,

    bg_phantom: marker::PhantomData<Bg>,
}

impl<'r, Font, Fg, Bg, R: render::Renderer<Font, Fg, Bg>> Writer<'r, Font, Fg, Bg, R>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<Option<colour::Definition>>,
{
    /// Constructs a writer on `renderer`, using the font spec `font_spec`.
    ///
    /// The writer initially points to the origin and uses a left anchor.
    pub fn new(renderer: &'r mut R) -> Self {
        Self {
            renderer,
            font_spec: font::Spec::default(),
            pos: metrics::Point::default(),
            alignment: metrics::anchor::X::Left,
            bg_phantom: marker::PhantomData::default(),
        }
    }

    /// Changes the writer to use font `font_spec`.
    #[must_use]
    pub fn with_font(self, font_spec: font::Spec<Font::Id, Fg::Id>) -> Self {
        let font::Spec { id, colour } = font_spec;
        self.with_font_id(id).with_colour(colour)
    }

    /// Changes the writer to use font ID `id`.
    #[must_use]
    pub fn with_font_id(mut self, id: Font::Id) -> Self {
        self.font_spec.id = id;
        self
    }

    /// Changes the writer to use foreground colour `fg`.
    #[must_use]
    pub fn with_colour(mut self, fg: Fg::Id) -> Self {
        // No need to recalculate the font metrics if we're just changing the colour
        self.font_spec.colour = fg;
        self
    }

    /// Moves the writer to position `pos`.
    #[must_use]
    pub fn with_pos(mut self, pos: metrics::Point) -> Self {
        self.pos = pos;
        self
    }

    /// Re-aligns the writer to anchor `anchor`.
    #[must_use]
    pub fn align(mut self, anchor: metrics::anchor::X) -> Self {
        self.alignment = anchor;
        self
    }

    fn string_top_left(&self, s: &str) -> metrics::Point {
        let m = self.renderer.font_metrics().get(self.font_spec.id);
        self.pos.offset(-m.x_anchor_of_str(s, self.alignment), 0)
    }
}

/// We can use a writer's underlying renderer through it.
impl<'r, Font, Fg, Bg, R> render::Renderer<Font, Fg, Bg> for Writer<'r, Font, Fg, Bg, R>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<Option<colour::Definition>>,
    R: render::Renderer<Font, Fg, Bg>,
{
    fn write(
        &mut self,
        pos: metrics::Point,
        font: font::Spec<Font::Id, Fg::Id>,
        s: &str,
    ) -> error::Result<metrics::Point> {
        self.renderer.write(pos, font, s)
    }

    fn fill(&mut self, rect: super::metrics::Rect, colour: Bg::Id) -> error::Result<()> {
        self.renderer.fill(rect, colour)
    }

    fn clear(&mut self, colour: Bg::Id) -> error::Result<()> {
        self.renderer.clear(colour)
    }

    fn present(&mut self) {
        self.renderer.present();
    }

    fn font_metrics(&self) -> &Font::MetricsMap {
        self.renderer.font_metrics()
    }
}

/// We can use writers with Rust's formatting system.
impl<'r, Font, Fg, Bg, R> std::fmt::Write for Writer<'r, Font, Fg, Bg, R>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<Option<colour::Definition>>,
    R: render::Renderer<Font, Fg, Bg>,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.pos = self
            .renderer
            .write(self.string_top_left(s), self.font_spec, s)
            .map_err(|_| std::fmt::Error)?;

        Ok(())
    }

    /// Forces a formatting write to send one string to `write_str`.
    ///
    /// This is to make non-left-aligned writes work as one would expect.
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        let cow = args.as_str().map_or_else(
            || std::borrow::Cow::from(args.to_string()),
            std::borrow::Cow::from,
        );
        self.write_str(&cow)
    }
}
