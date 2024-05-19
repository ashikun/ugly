//! Rendering using `wgpu`.
use crate::backends::wgpu::texture::Texture;
use crate::{
    backends::wgpu::{core::Core, vertex},
    colour, font,
    metrics::Rect,
    resource, Result,
};
use std::rc::Rc;

#[ouroboros::self_referencing]
pub struct Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map + 'a,
    Fg: resource::Map<colour::Definition> + 'a,
    Bg: resource::Map<colour::Definition> + 'a,
{
    bg: colour::Definition,

    resources: resource::Set<Font, Fg, Bg>,

    pub core: Core<'a>,

    #[covariant]
    #[borrows(resources)]
    font_manager: font::Manager<'this, Font, Rc<Texture>>,

    current_index: vertex::Index,
    shapes: vertex::ShapeQueue,
}

impl<'a, Font, Fg, Bg> crate::Renderer<'a, Font, Fg, Bg> for Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map + 'a,
    Fg: resource::Map<colour::Definition> + 'a,
    Bg: resource::Map<colour::Definition> + 'a,
{
    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.borrow_resources().metrics
    }

    fn write(
        &mut self,
        font: font::Spec<Font::Id, Fg::Id>,
        str: &font::layout::String,
    ) -> Result<()> {
        // TODO: instancing
        let colour = self.lookup_fg(font.colour);

        let texture = self.with_mut(|this| this.font_manager.data(font.id, this.core).cloned())?;

        for glyph in &str.glyphs {
            let material = vertex::Material {
                texture: texture.clone(),
                colour,
                dimensions: glyph.src,
            };

            self.push_shape(vertex::Shape::quad(glyph.dst, material));
        }

        Ok(())
    }

    fn fill(&mut self, rect: Rect, colour: Bg::Id) -> Result<()> {
        // Make a texture rect whose coordinates will always be negative
        let tex_rect = Rect::new(-2, -2, 1, 1);

        let material = vertex::Material {
            colour: self.lookup_bg(colour),
            texture: self.borrow_core().null_texture(),
            dimensions: tex_rect,
        };

        self.push_shape(vertex::Shape::quad(rect, material));

        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        /* We clear at the beginning of every rendering cycle anyway, so
         * 'clear' is tantamount to changing the colour we clear to.
         */
        let new_bg = self.lookup_bg(colour);
        self.with_bg_mut(|bg| *bg = new_bg);

        Ok(())
    }

    fn present(&mut self) {
        self.with_mut(|f| f.core.render(*f.bg, &f.shapes.take()));
    }
}

impl<'a, Font, Fg, Bg> Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    /// Constructs a new `wgpu` renderer.
    pub fn from_core(core: Core<'a>, resources: resource::Set<Font, Fg, Bg>) -> Self {
        RendererBuilder {
            resources,
            core,
            bg: colour::Definition::default(),
            current_index: 0,
            shapes: vertex::ShapeQueue::default(),
            font_manager_builder: |res| font::Manager::new(&res.fonts, &res.metrics),
        }
        .build()
    }

    fn push_shape(&mut self, shape: vertex::Shape) {
        self.with_shapes_mut(|shapes| shapes.push(shape));
    }

    /// Looks up a background colour.
    fn lookup_bg(&self, id: Bg::Id) -> colour::Definition {
        *self.borrow_resources().palette.bg.get(id)
    }

    /// Looks up a foreground colour.
    fn lookup_fg(&self, id: Fg::Id) -> colour::Definition {
        *self.borrow_resources().palette.fg.get(id)
    }
}
