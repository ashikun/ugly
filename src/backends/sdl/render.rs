//! The SDL low-level graphics rendering layer.

use std::cell::RefMut;

use crate::font::Metrics;
use crate::{
    colour,
    error::{Error, Result},
    font, metrics, render,
};
use sdl2::{render::Canvas, video};

/// The SDL window graphics renderer.
///
/// As usual, the renderer is parameterised over the IDs for fonts (`FId`), foreground colours
/// (`Fg`), and background colours (`Bg`).  Its lifetime `a` represents the parent resource manager.
pub struct Renderer<'a, FId, Fg, Bg> {
    /// The target screen canvas.
    screen: RefMut<'a, Canvas<video::Window>>,
    /// The font manager.
    font_manager: super::font::Manager<'a, FId, Fg>,
    /// The colour set.
    colour_set: &'a colour::MapSet<Fg, Bg>,
}

impl<'a, FId: font::Id, Fg: colour::id::Fg, Bg: colour::id::Bg> render::Renderer<FId, Fg, Bg>
    for Renderer<'a, FId, Fg, Bg>
{
    fn write(
        &mut self,
        mut pos: metrics::Point,
        font: font::Spec<FId, Fg>,
        s: &str,
    ) -> Result<metrics::Point> {
        let texture = self.font_manager.texture(font)?;
        let metrics = self.get_metrics(font.id)?;

        for glyph in metrics.layout_str(pos, s.as_bytes()) {
            let src = super::metrics::convert_rect(&glyph.src);
            let dst = super::metrics::convert_rect(&glyph.dst);

            // Move from the end of the last character to the start of the next one.
            pos = glyph
                .dst
                .point(metrics.pad.w, 0, metrics::Anchor::TOP_RIGHT);

            self.screen.copy(&texture, src, dst).map_err(Error::Backend)?;
        }

        Ok(pos)
    }

    fn fill(&mut self, rect: metrics::Rect, colour: Bg) -> Result<()> {
        let rect = super::metrics::convert_rect(&rect);
        self.set_screen_bg(colour);
        self.screen.fill_rect(rect).map_err(Error::Backend)
    }

    /// Clears the screen.
    fn clear(&mut self, colour: Bg) -> Result<()> {
        self.set_screen_bg(colour);
        self.screen.clear();
        Ok(())
    }

    /// Refreshes the screen.
    fn present(&mut self) {
        self.screen.present();
    }

    fn font_metrics(&self) -> &font::metrics::Map<FId> {
        &self.font_manager.metrics_set
    }
}

impl<'a, FId: font::Id, Fg: colour::id::Fg, Bg: colour::id::Bg> Renderer<'a, FId, Fg, Bg> {
    /// Constructs a [Renderer] using the given screen, font manager, and colour set.
    #[must_use]
    pub fn new(
        screen: RefMut<'a, Canvas<video::Window>>,
        font_manager: super::font::Manager<'a, FId, Fg>,
        colour_set: &'a colour::MapSet<Fg, Bg>,
    ) -> Self {
        Self {
            screen,
            font_manager,
            colour_set,
        }
    }

    // Sets the screen draw colour to `bg`.
    fn set_screen_bg(&mut self, bg: Bg) {
        self.screen
            .set_draw_color(colour_to_sdl(self.colour_set.bg_or_black(bg)));
    }

    fn get_metrics(&mut self, id: FId) -> font::Result<Metrics> {
        // TODO(@MattWindsor91): can we replace font_metrics() with this?
        self.font_manager
            .metrics_set
            .get(&id)
            .cloned()
            .ok_or_else(|| font::Error::unknown_font(id))
    }
}

/// Converts an `ugly` colour to a SDL one.
fn colour_to_sdl(c: colour::Definition) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(c.red_byte(), c.green_byte(), c.blue_byte(), c.alpha_byte())
}
