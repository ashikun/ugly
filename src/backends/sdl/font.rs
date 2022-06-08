//! Specifics pertaining to how we set up the font manager for SDL2.

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
};

use crate::{colour, font};

/// Adaptor to allow SDL texture creators to load in font PNGs as SDL textures.
pub struct Loader<'a, Ctx> {
    pub creator: &'a TextureCreator<Ctx>,
}

impl<'l, Ctx> font::manager::Loader<'l> for Loader<'l, Ctx> {
    type Data = Texture<'l>;

    fn load(&'l self, path: impl AsRef<std::path::Path>) -> font::Result<Self::Data> {
        self.creator
            .load_texture(path)
            .map_err(font::Error::TextureLoad)
    }

    fn colourise(&self, mut data: Self::Data, fg: colour::Definition) -> Self::Data {
        data.set_color_mod(fg.r, fg.g, fg.b);
        data.set_alpha_mod(fg.a);
        data
    }
}

/// Shorthand for the type of font manager the SDL backend uses.
pub type Manager<'a, Font, Fg, Ctx> = font::manager::Cached<'a, Font, Fg, Loader<'a, Ctx>>;
