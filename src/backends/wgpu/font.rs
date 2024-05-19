//! Font loading for `wgpu`.

use crate::font;

use super::{texture::Texture, Error};

impl<'l> font::manager::Loader for super::Core<'l> {
    type Data<'a> = std::rc::Rc<Texture> where Self: 'a;

    fn load(&mut self, path: impl AsRef<std::path::Path>) -> font::Result<Self::Data<'_>> {
        self.load_image(path).map_err(map_font_err)
    }
}

fn map_font_err(error: Error) -> font::Error {
    // TODO: make this more granular
    match error {
        Error::Io(e) => font::Error::Io(e),
        e => font::Error::TextureLoad(e.to_string()),
    }
}
