//! Font loading for `wgpu`.

use crate::{colour, font};

use super::{vertex, Error};

impl<'l> font::manager::Loader for super::Core<'l> {
    type Data<'a> = vertex::Material<()> where Self: 'a;

    fn load(&mut self, path: impl AsRef<std::path::Path>) -> font::Result<Self::Data<'_>> {
        let texture = self.load_image(path).map_err(map_font_err)?;

        let data = vertex::Material {
            colour: colour::Definition::rgb(255, 255, 255),
            texture,
            dimensions: (),
        };

        Ok(data)
    }
}

fn map_font_err(error: Error) -> font::Error {
    // TODO: make this more granular
    match error {
        Error::Io(e) => font::Error::Io(e),
        e => font::Error::TextureLoad(e.to_string()),
    }
}
