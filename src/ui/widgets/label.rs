//! Label widgets.

use super::super::{
    super::{colour, font, metrics, resource::Map, text::Writer, Renderer, Result},
    layout::Layoutable,
    render::Renderable,
    update::Updatable,
};

/// A widget that displays a static single-line string with a static font.
///
/// `Font`, `Fg`, and `Bg` are the usual font and colour ID types.
#[derive(Clone)]
pub struct Label<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    /// The most recently computed bounding box for the label.
    bounds: metrics::Rect,

    /// The writer for the label.
    writer: Writer<Font, Fg, Bg>,

    /// The minimum amount of expected characters in the label.
    pub min_chars: u8,
}

impl<Font, Fg, Bg> Label<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    /// Constructs a label with the given font specification.
    #[must_use]
    pub fn new(font_spec: font::Spec<Font::Id, Fg::Id>) -> Self {
        let mut writer = Writer::new();
        writer.set_font_spec(font_spec);

        Self {
            bounds: metrics::Rect::default(),
            writer,
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
    pub fn set_fg(&mut self, fg: Fg::Id) {
        self.writer.set_fg(fg);
    }

    /// Sets the font of the label.
    pub fn set_font(&mut self, font: Font::Id) {
        self.writer.set_font(font);
    }

    /// Sets the font spec of the label.
    pub fn set_font_spec(&mut self, spec: font::Spec<Font::Id, Fg::Id>) {
        self.writer.set_font_spec(spec);
    }

    /// Converts `str` to a string then updates the label with it.
    pub fn update_display(&mut self, metrics: &Font::MetricsMap, str: impl std::fmt::Display) {
        self.writer.move_to(self.writer_pos());
        self.writer.set_string(&str);
        self.writer.layout(metrics);
    }

    fn writer_pos(&self) -> metrics::Point {
        self.bounds.anchor(metrics::anchor::Anchor {
            x: self.writer.alignment(),
            y: metrics::anchor::Y::Top,
        })
    }
}

/// We can layout a label, so long as the context serves font metrics for the font ID set in use.
impl<Font, Fg, Bg, Ctx> Layoutable<Ctx> for Label<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
    Ctx: Context<Font>,
{
    fn min_bounds(&self, ctx: &Ctx) -> metrics::Size {
        ctx.font_metrics()
            .get(self.writer.font_spec().id)
            .text_size(i32::from(self.min_chars), 1)
    }

    fn layout(&mut self, _: &Ctx, bounds: metrics::Rect) {
        self.bounds = bounds;
    }
}

/// We can update a label, so long as the context serves font metrics for the font ID set in use.
impl<Font, Fg, Bg, Ctx> Updatable<Ctx> for Label<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
    Ctx: Context<Font>,
{
    type State = str;

    fn update(&mut self, ctx: &Ctx, s: &Self::State) {
        self.update_display(ctx.font_metrics(), s);
    }
}

/// Delegates rendering to the writer.
impl<'r, Font, Fg, Bg, R: Renderer<'r, Font, Fg, Bg>> Renderable<R> for Label<Font, Fg, Bg>
where
    Font: font::Map,
    Fg: Map<colour::Definition>,
    Bg: Map<colour::Definition>,
{
    fn render(&self, r: &mut R) -> Result<()> {
        self.writer.render(r)
    }
}

/// Trait required for layout and update contexts over labels.
pub trait Context<Font: font::Map> {
    /// Gets the font metrics map.
    fn font_metrics(&self) -> &Font::MetricsMap;
}
