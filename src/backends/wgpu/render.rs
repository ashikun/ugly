//! Rendering using `wgpu`.
use crate::{
    backends::wgpu::{core::Core, vertex},
    colour, font,
    metrics::Rect,
    resource, Result,
};

pub struct Renderer<
    'a,
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
> {
    bg: colour::Definition,
    fonts: Font,
    metrics: Font::MetricsMap,
    palette: colour::Palette<Fg, Bg>,

    pub core: Core<'a>,

    current_index: vertex::Index,
    shapes: Vec<vertex::Shape>,
}

impl<
        'a,
        Font: font::Map,
        Fg: resource::Map<colour::Definition>,
        Bg: resource::Map<colour::Definition>,
    > crate::Renderer<'a, Font, Fg, Bg> for Renderer<'a, Font, Fg, Bg>
{
    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.metrics
    }

    fn write(
        &mut self,
        font: font::Spec<Font::Id, Fg::Id>,
        str: &font::layout::String,
    ) -> Result<()> {
        Ok(())
    }

    fn fill(&mut self, rect: Rect, colour: Bg::Id) -> Result<()> {
        // Make a texture rect whose coordinates will always be negative
        let tex_rect = Rect::new(-2, -2, 1, 1);

        let material = vertex::Material {
            colour: self.lookup_bg(colour),
            texture: self.core.null_texture(),
            dimensions: tex_rect,
        };

        self.push_shape(|i| vertex::Shape::quad(i, rect, material));

        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        /* We clear at the beginning of every rendering cycle anyway, so
         * 'clear' is tantamount to changing the colour we clear to.
         */
        self.bg = self.lookup_bg(colour);

        Ok(())
    }

    fn present(&mut self) {
        let shapes = std::mem::take(&mut self.shapes);
        self.current_index = 0;

        self.core.render(self.bg, &shapes);
    }
}

impl<
        'a,
        Font: font::Map,
        Fg: resource::Map<colour::Definition>,
        Bg: resource::Map<colour::Definition>,
    > Renderer<'a, Font, Fg, Bg>
{
    /// Constructs a new `wgpu` renderer.
    ///
    /// # Errors
    ///
    /// Fails if we can't load font metrics.
    pub fn new(fonts: Font, palette: colour::Palette<Fg, Bg>, core: Core<'a>) -> Result<Self> {
        let metrics = fonts.load_metrics()?;

        let result = Self {
            bg: colour::Definition::default(),
            fonts,
            metrics,
            palette,
            core,
            current_index: 0,
            shapes: vec![],
        };

        Ok(result)
    }

    fn push_shape(&mut self, shape_fn: impl FnOnce(vertex::Index) -> vertex::Shape) {
        let shape = shape_fn(self.current_index);

        self.current_index += shape.num_vertices();
        self.shapes.push(shape);
    }

    /// Looks up a background colour.
    fn lookup_bg(&self, id: Bg::Id) -> colour::Definition {
        *self.palette.bg.get(id)
    }

    /// Looks up a foreground colour.
    fn lookup_fg(&self, id: Fg::Id) -> colour::Definition {
        *self.palette.fg.get(id)
    }
}
