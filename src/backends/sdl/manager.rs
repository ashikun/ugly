//! Top-level resource holder for an `ugly` session over SDL2.

use std::cell::RefCell;

use sdl2::render::{Canvas, RenderTarget, TextureCreator};

use crate::{colour, font, resource, Error, Result};

/// Manages top-level SDL resources.
///
/// As usual, `FId` is the type of font identifiers, `Fg` the type of foreground colour identifiers,
/// and `Bg` the type of background colour identifiers.
///
/// `c` is the lifetime of the configuration used by the manager.
///
/// `Tgt` is the type of the underlying render target (window or screen).
pub struct Manager<'c, Font, Fg, Bg, Tgt: Target> {
    canvas: RefCell<sdl2::render::Canvas<Tgt>>,
    textures: sdl2::render::TextureCreator<Tgt::Context>,
    fonts: &'c Font,
    colours: &'c colour::MapSet<Fg, Bg>,
}

impl<'c, Font, Fg, Bg, Tgt: Target> Manager<'c, Font, Fg, Bg, Tgt>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
{
    /// Creates a new rendering manager over a given rendering target.
    ///
    /// # Errors
    ///
    /// Fails if we can't construct the requisite canvas for the target.  
    pub fn new(target: Tgt, fonts: &'c Font, colours: &'c colour::MapSet<Fg, Bg>) -> Result<Self> {
        let canvas = target.into_canvas()?;
        let textures = Tgt::texture_creator(&canvas);
        Ok(Self {
            canvas: RefCell::new(canvas),
            textures,
            fonts,
            colours,
        })
    }

    /// Spawns a renderer targeting the SDL window.
    ///
    /// # Errors
    ///
    /// Fails if we can't set up the font metrics map.
    pub fn renderer(&self) -> Result<super::render::Renderer<Font, Fg, Bg, Tgt>> {
        let metrics = self.fonts.load_metrics()?;
        let font_manager =
            super::font::Manager::new(&self.textures, self.fonts, metrics, &self.colours.fg);
        Ok(super::render::Renderer::new(
            self.canvas.borrow_mut(),
            font_manager,
            self.colours,
        ))
    }
}

/// Extension trait to `RenderTarget` making it possible for the `Manager` to be generic over
/// screens and windows.
pub trait Target: RenderTarget + Sized {
    /// Wraps self in a `Canvas`.
    ///
    /// # Errors
    ///
    /// Fails if the conversion does not work at the SDL level.
    fn into_canvas(self) -> Result<Canvas<Self>>;

    /// Gets a texture creator for this target.
    ///
    /// This is not exposed as part of `RenderTarget` upstream; there are probably good reasons for
    /// this, but this trait is blissfully unaware of them.
    fn texture_creator(canvas: &Canvas<Self>) -> TextureCreator<Self::Context>;
}

impl<'s> Target for sdl2::surface::Surface<'s> {
    fn into_canvas(self) -> Result<Canvas<Self>> {
        self.into_canvas().map_err(Error::Backend)
    }

    fn texture_creator(canvas: &Canvas<Self>) -> TextureCreator<Self::Context> {
        canvas.texture_creator()
    }
}

impl Target for sdl2::video::Window {
    fn into_canvas(self) -> Result<Canvas<Self>> {
        self.into_canvas()
            .build()
            .map_err(|e| Error::Backend(e.to_string()))
    }

    fn texture_creator(canvas: &Canvas<Self>) -> TextureCreator<Self::Context> {
        canvas.texture_creator()
    }
}
