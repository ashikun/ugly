//! Font loading for `wgpu`.

use super::{texture, vertex, Error};
use crate::{colour, font, metrics};
use std::rc::Rc;

impl<'l> font::manager::Loader<'l> for super::Core<'l> {
    type Data = vertex::Material<()>;

    fn load(&'l self, path: impl AsRef<std::path::Path>) -> font::Result<Self::Data> {
        let texture = self.load_image(path).map_err(map_font_err)?;

        let data = vertex::Material {
            colour: colour::Definition::rgb(255, 255, 255),
            texture,
            dimensions: (),
        };

        Ok(data)
    }

    fn colourise(&self, data: &mut Self::Data, fg: colour::Definition) {
        // TODO: is this necessary?!
        data.colour = fg;
    }
}

fn map_font_err(error: Error) -> font::Error {
    // TODO: make this more granular
    match error {
        Error::Io(e) => font::Error::Io(e),
        e => font::Error::TextureLoad(e.to_string()),
    }
}
