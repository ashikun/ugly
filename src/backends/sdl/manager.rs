//! Top-level resource holder for an `ugly` session over SDL2.

use std::cell::RefCell;

use crate::{colour, font, Result};

/// Manages top-level SDL resources.
///
/// As usual, `FId` is the type of font identifiers, `Fg` the type of foreground colour identifiers,
/// and `Bg` the type of background colour identifiers.
pub struct Manager<FId, Fg, Bg> {
    screen: RefCell<sdl2::render::Canvas<sdl2::video::Window>>,
    textures: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    fonts: font::path::Map<FId>,
    colours: colour::MapSet<Fg, Bg>,
}

impl<FId: font::Id, Fg: colour::id::Fg, Bg: colour::id::Bg> Manager<FId, Fg, Bg> {
    /// Creates a new rendering manager over a given SDL2 canvas.
    pub fn new(
        screen: sdl2::render::Canvas<sdl2::video::Window>,
        fonts: font::path::Map<FId>,
        colours: colour::MapSet<Fg, Bg>,
    ) -> Self {
        let textures = screen.texture_creator();
        Self {
            screen: RefCell::new(screen),
            textures,
            fonts,
            colours,
        }
    }

    /// Spawns a renderer targeting the SDL window.
    fn renderer(&self) -> Result<super::render::Renderer<FId, Fg, Bg>> {
        let metrics = font::metrics::load_map(&self.fonts)?;
        let font_manager =
            super::font::Manager::new(&self.textures, &self.fonts, metrics, &self.colours.fg);
        Ok(super::render::Renderer::new(
            self.screen.borrow_mut(),
            font_manager,
            &self.colours,
        ))
    }
}
