//! Font loading for `wgpu`.

use crate::font;

use super::{texture::Texture, Error};

pub(super) fn load(
    core: &mut super::Core,
    path: &std::path::Path,
) -> font::Result<std::rc::Rc<Texture>> {
    core.load_image(path).map_err(map_font_err)
}

fn map_font_err(error: Error) -> font::Error {
    // TODO: make this more granular
    match error {
        Error::Io(e) => font::Error::Io(e),
        e => font::Error::TextureLoad(e.to_string()),
    }
}
