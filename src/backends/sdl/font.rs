//! Specifics pertaining to how we set up the font manager for SDL2.

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
};

use crate::{colour, font};

impl<'l, Ctx> font::manager::Loader<'l> for TextureCreator<Ctx> {
    type Data = Texture<'l>;

    fn load(&'l self, path: impl AsRef<std::path::Path>) -> font::Result<Self::Data> {
        self.load_texture(path).map_err(font::Error::TextureLoad)
    }

    fn colourise(&self, data: &mut Self::Data, fg: colour::Definition) {
        data.set_color_mod(fg.r, fg.g, fg.b);
        data.set_alpha_mod(fg.a);
    }
}

/// Shorthand for the type of font manager the SDL backend uses.
pub type Manager<'a, Font, Fg, Ctx> = font::manager::Manager<'a, Font, Fg, TextureCreator<Ctx>>;
