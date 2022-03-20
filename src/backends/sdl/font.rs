//! The font manager.
use std::{collections::HashMap, rc::Rc};

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

use crate::{colour, error::Result, font};

/// A font manager, using a SDL texture creator.
///
/// The lifeline `a` corresponds to the lifeline of the main resource manager.
pub struct Manager<'a, FId, Fg> {
    /// The texture creator used to load fonts.
    creator: &'a TextureCreator<WindowContext>,
    /// The map of current font textures.
    textures: HashMap<font::Spec<FId, Fg>, Rc<Texture<'a>>>,
    /// The font path set.
    font_set: &'a font::path::Map<FId>,
    /// The font metrics set.
    pub metrics_set: font::metrics::Map<FId>,
    /// The foreground colour set, used for setting up font colours.
    colour_set: &'a colour::Map<Fg>,
}

impl<'a, FId: font::Id, Fg: colour::id::Fg> Manager<'a, FId, Fg> {
    /// Creates a font manager with the given texture creator and config maps.
    #[must_use]
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        font_set: &'a font::path::Map<FId>,
        metrics_set: font::metrics::Map<FId>,
        colour_set: &'a colour::Map<Fg>,
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
    pub fn texture(&mut self, spec: font::Spec<FId, Fg>) -> Result<Rc<Texture<'a>>> {
        self.textures
            .get(&spec)
            .cloned()
            .map_or_else(|| self.cache(spec), Ok)
    }

    fn cache(&mut self, spec: font::Spec<FId, Fg>) -> Result<Rc<Texture<'a>>> {
        let tex = Rc::new(self.load(spec)?);
        self.textures.insert(spec, tex.clone());
        Ok(tex)
    }

    fn load(&mut self, spec: font::Spec<FId, Fg>) -> Result<Texture<'a>> {
        let id = spec.id;
        let path = &self
            .font_set
            .get(&id)
            .ok_or_else(|| font::Error::TextureLoad(format!("Missing texture file: {id:?}")))?
            .texture_path();
        let mut tex = self
            .creator
            .load_texture(path)
            .map_err(font::Error::TextureLoad)?;
        self.colourise(&mut tex, spec.colour);
        Ok(tex)
    }

    fn colourise(&self, texture: &mut Texture, colour: Fg) {
        let colour = colour::definition::fg_or_white(self.colour_set, colour);
        texture.set_color_mod(colour.red_byte(), colour.green_byte(), colour.blue_byte());
        texture.set_alpha_mod(colour.alpha_byte());
    }
}
