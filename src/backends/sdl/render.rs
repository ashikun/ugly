//! The SDL low-level graphics rendering layer.

use std::cell::RefMut;

use sdl2::render::{Canvas, RenderTarget};

use crate::{
    colour,
    error::{Error, Result},
    font, metrics, render, resource,
};

/// The SDL window graphics renderer.
///
/// As usual, the renderer is parameterised over maps for font metrics (`FMet`), foreground colours
/// (`Fg`), and background colours (`Bg`).  Its lifetime `a` represents the parent resource manager.
/// `Tgt` represents a rendering context.
pub struct Renderer<'a, Font, Fg, Bg, Tgt>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Tgt: RenderTarget,
{
    /// The target screen canvas.
    canvas: RefMut<'a, Canvas<Tgt>>,
    /// The font manager.
    font_manager: super::font::Manager<'a, Font, Fg, Tgt::Context>,
    /// The colour set.
    colour_set: &'a colour::Palette<Fg, Bg>,
}

impl<'a, Font, Fg, Bg, Tgt> render::Renderer<Font, Fg, Bg> for Renderer<'a, Font, Fg, Bg, Tgt>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
    Tgt: RenderTarget,
{
    fn write(
        &mut self,
        font: font::Spec<Font::Id, Fg::Id>,
        string: &font::layout::String,
    ) -> Result<()> {
        let texture = self.font_manager.texture(font)?;

        for glyph in &string.glyphs {
            let src = super::metrics::convert_rect(&glyph.src);
            let dst = super::metrics::convert_rect(&glyph.dst);

            self.canvas
                .copy(&texture, src, dst)
                .map_err(Error::Backend)?;
        }

        Ok(())
    }

    fn fill(&mut self, rect: metrics::Rect, colour: Bg::Id) -> Result<()> {
        let col = *self.colour_set.bg.get(colour);
        // Don't fill if the colour is fully transparent; there's no point.
        if !col.is_transparent() {
            let rect = super::metrics::convert_rect(&rect);
            self.set_screen_bg(col);
            self.canvas.fill_rect(rect).map_err(Error::Backend)?;
        }
        Ok(())
    }

    /// Clears the screen.
    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        self.set_screen_bg(*self.colour_set.bg.get(colour));
        self.canvas.clear();
        Ok(())
    }

    /// Refreshes the screen.
    fn present(&mut self) {
        self.canvas.present();
    }

    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.font_manager.metrics_set
    }
}

impl<'a, Font, Fg, Bg, Tgt: RenderTarget> Renderer<'a, Font, Fg, Bg, Tgt>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    /// Constructs a [Renderer] using the given screen, font manager, and colour set.
    #[must_use]
    pub fn new(
        canvas: RefMut<'a, Canvas<Tgt>>,
        font_manager: super::font::Manager<'a, Font, Fg, Tgt::Context>,
        colour_set: &'a colour::Palette<Fg, Bg>,
    ) -> Self {
        Self {
            canvas,
            font_manager,
            colour_set,
        }
    }

    // Sets the screen draw colour to `bg`.
    fn set_screen_bg(&mut self, bg: colour::Definition) {
        self.canvas.set_draw_color(colour_to_sdl(bg));
    }
}

/// Converts an `ugly` colour to a SDL one.
fn colour_to_sdl(c: colour::Definition) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(c.r, c.g, c.b, c.a)
}
