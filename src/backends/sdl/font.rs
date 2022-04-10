//! The font manager.

use std::hash::Hash;
use std::{collections::HashMap, rc::Rc};

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
};

use crate::{colour, error::Result, font, resource};

/// A font manager, using a SDL texture creator.
///
/// The lifeline `a` corresponds to the lifeline of the main resource manager.
pub struct Manager<'a, Font, Fg, Ctx>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
{
    /// The texture creator used to load fonts.
    creator: &'a TextureCreator<Ctx>,
    /// The map of current font textures.
    textures: HashMap<font::Spec<Font::Id, Fg::Id>, Rc<Texture<'a>>>,
    /// The font path set.
    font_set: &'a Font,
    /// The font metrics set.
    pub metrics_set: Font::MetricsMap,
    /// The foreground colour set, used for setting up font colours.
    colour_set: &'a Fg,
}

impl<'a, Font, Fg, Ctx> Manager<'a, Font, Fg, Ctx>
where
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Font::Id: Eq + Hash,
    Fg::Id: Eq + Hash,
{
    /// Creates a font manager with the given texture creator and config maps.
    #[must_use]
    pub fn new(
        creator: &'a TextureCreator<Ctx>,
        font_set: &'a Font,
        metrics_set: Font::MetricsMap,
        colour_set: &'a Fg,
    ) -> Self {
        Self {
            creator,
            textures: HashMap::new(),
            font_set,
            metrics_set,
            colour_set,
        }
    }

    /// Gets the given font spec as a texture, or loads it if
    /// it hasn't yet been loaded.
    ///
    /// # Errors
    ///
    /// Returns an error if we need to load the font but SDL cannot for some
    /// reason, or the font is not configured.
    pub fn texture(&mut self, spec: font::Spec<Font::Id, Fg::Id>) -> Result<Rc<Texture<'a>>> {
        self.textures
            .get(&spec)
            .cloned()
            .map_or_else(|| self.cache(spec), Ok)
    }

    fn cache(&mut self, spec: font::Spec<Font::Id, Fg::Id>) -> Result<Rc<Texture<'a>>> {
        let tex = Rc::new(self.load(spec)?);
        self.textures.insert(spec, tex.clone());
        Ok(tex)
    }

    fn load(&mut self, spec: font::Spec<Font::Id, Fg::Id>) -> Result<Texture<'a>> {
        let id = spec.id;
        let path = &self.font_set.get(id).texture_path();
        let mut tex = self
            .creator
            .load_texture(path)
            .map_err(font::Error::TextureLoad)?;
        self.colourise(&mut tex, spec.colour);
        Ok(tex)
    }

    fn colourise(&self, texture: &mut Texture, colour: Fg::Id) {
        let colour = self.colour_set.get(colour);
        texture.set_color_mod(colour.red_byte(), colour.green_byte(), colour.blue_byte());
        texture.set_alpha_mod(colour.alpha_byte());
    }
}
