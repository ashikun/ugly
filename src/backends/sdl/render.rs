//! The SDL low-level graphics rendering layer.

use std::cell::RefMut;

use sdl2::render::{Canvas, RenderTarget};

use crate::{
    colour,
    error::{Error, Result},
    font, metrics, render,
};

/// The SDL window graphics renderer.
///
/// As usual, the renderer is parameterised over the IDs for fonts (`FId`), foreground colours
/// (`Fg`), and background colours (`Bg`).  Its lifetime `a` represents the parent resource manager.
/// `Tgt` represents a rendering context.
pub struct Renderer<'a, FId, Fg, Bg, Tgt: RenderTarget> {
    /// The target screen canvas.
    canvas: RefMut<'a, Canvas<Tgt>>,
    /// The font manager.
    font_manager: super::font::Manager<'a, FId, Fg, Tgt::Context>,
    /// The colour set.
    colour_set: &'a colour::MapSet<Fg, Bg>,
}

impl<'a, FId: font::Id, Fg: colour::id::Fg, Bg: colour::id::Bg, Tgt: RenderTarget>
    render::Renderer<FId, Fg, Bg> for Renderer<'a, FId, Fg, Bg, Tgt>
{
    fn write(
        &mut self,
        mut pos: metrics::Point,
        font: font::Spec<FId, Fg>,
        s: &str,
    ) -> Result<metrics::Point> {
        let texture = self.font_manager.texture(font)?;
        let metrics = self.font_manager.metrics(font.id)?;

        for glyph in metrics.layout_str(pos, s) {
            let src = super::metrics::convert_rect(&glyph.src);
            let dst = super::metrics::convert_rect(&glyph.dst);

            // Move from the end of the last character to the start of the next one.
            pos = glyph
                .dst
                .point(metrics.pad.w, 0, metrics::Anchor::TOP_RIGHT);

            self.canvas
                .copy(&texture, src, dst)
                .map_err(Error::Backend)?;
        }

        Ok(pos)
    }

    fn fill(&mut self, rect: metrics::Rect, colour: Bg) -> Result<()> {
        let rect = super::metrics::convert_rect(&rect);
        self.set_screen_bg(colour);
        self.canvas.fill_rect(rect).map_err(Error::Backend)
    }

    /// Clears the screen.
    fn clear(&mut self, colour: Bg) -> Result<()> {
        self.set_screen_bg(colour);
        self.canvas.clear();
        Ok(())
    }

    /// Refreshes the screen.
    fn present(&mut self) {
        self.canvas.present();
    }

    fn font_metrics(&self, id: FId) -> crate::Result<&font::Metrics> {
        self.font_manager.metrics(id)
    }
}

impl<'a, FId: font::Id, Fg: colour::id::Fg, Bg: colour::id::Bg, Tgt: RenderTarget>
    Renderer<'a, FId, Fg, Bg, Tgt>
{
    /// Constructs a [Renderer] using the given screen, font manager, and colour set.
    #[must_use]
    pub fn new(
        canvas: RefMut<'a, Canvas<Tgt>>,
        font_manager: super::font::Manager<'a, FId, Fg, Tgt::Context>,
        colour_set: &'a colour::MapSet<Fg, Bg>,
    ) -> Self {
        Self {
            canvas,
            font_manager,
            colour_set,
        }
    }

    // Sets the screen draw colour to `bg`.
    fn set_screen_bg(&mut self, bg: Bg) {
        self.canvas
            .set_draw_color(colour_to_sdl(self.colour_set.bg_or_black(bg)));
    }
}

/// Converts an `ugly` colour to a SDL one.
fn colour_to_sdl(c: colour::Definition) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(c.red_byte(), c.green_byte(), c.blue_byte(), c.alpha_byte())
}
