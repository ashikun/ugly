//! Rendering using `wgpu`.
use itertools::Itertools;
use std::rc::Rc;

use crate::font::Metrics;
use crate::resource::Map;
use crate::{colour, font, metrics, resource, Result};

use super::{core::Core, instance::Instance, shape, texture::Texture, vertex};

pub struct Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map,
{
    pub core: Core<'a>,

    font_manager: font::Manager<Font, Rc<Texture>>,
    palette: colour::Palette<Fg, Bg>,

    bg: colour::Definition,
    shapes: shape::Queue,
}

// TODO: tidy this up
impl<'a, Font, Fg, Bg> crate::ui::layout::LayoutContext<Font::Id> for Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map,
{
    fn font_metrics(&self) -> &impl Map<Metrics, Id = Font::Id> {
        self.font_manager.metrics()
    }
}

impl<'a, Font, Fg, Bg> crate::Renderer<'a, Font::Id, Fg::Id, Bg::Id> for Renderer<'a, Font, Fg, Bg>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    fn write(&mut self, font: Font::Id, colour: Fg::Id, str: &font::layout::String) -> Result<()> {
        let colour = self.lookup_fg(colour);

        let texture = self
            .font_manager
            .data(font, |p| super::font::load(&mut self.core, p))
            .cloned()?;

        for glyph in str.glyphs.values() {
            let material = vertex::Material {
                texture: texture.clone(),
                colour,
                dimensions: glyph.src,
            };

            // Assuming that the source and dest are going to be the same
            let size = glyph.src.size;
            let init_dst = metrics::Rect {
                top_left: metrics::Point::default(),
                size,
            };

            let instances = glyph
                .dsts
                .iter()
                .map(|top_left| Instance {
                    delta: [top_left.x, top_left.y],
                })
                .collect_vec();

            let shape = shape::Shape::quad(init_dst, material).instanced(instances);

            self.push_shape(shape);
        }

        Ok(())
    }

    fn fill(&mut self, rect: metrics::Rect, colour: Bg::Id) -> Result<()> {
        // Make a texture rect whose coordinates will always be negative
        let tex_rect = metrics::Rect::new(-2, -2, 1, 1);

        let material = vertex::Material {
            colour: self.lookup_bg(colour),
            texture: self.core.null_texture(),
            dimensions: tex_rect,
        };

        self.push_shape(shape::Shape::quad(rect, material));

        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        /* We clear at the beginning of every rendering cycle anyway, so
         * 'clear' is tantamount to changing the colour we clear to.
         */
        let new_bg = self.lookup_bg(colour);
        self.bg = new_bg;

        Ok(())
    }

    fn present(&mut self) {
        let (buffers, manifests) = self.shapes.take();
        self.core.render(self.bg, &buffers, manifests);
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
        Self {
            core,
            bg: colour::Definition::default(),
            shapes: shape::Queue::default(),
            font_manager: font::Manager::new(resources.fonts, resources.metrics),
            palette: resources.palette,
        }
    }

    fn push_shape(&mut self, shape: shape::Shape) {
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
