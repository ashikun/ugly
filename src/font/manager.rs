//! Font management.
//!
//! This part of the font subsystem deals with storing font texture data, predominantly.
use std::{hash::Hash, path::Path};

use super::{
    super::{
        colour,
        resource::{Map, MutableMap},
    },
    Result, Spec,
};

/// An index for a loaded font in a [Manager].
///
/// The index may be arbitrary, as long as it can be traded in for a valid font.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Index(usize);

/// The default value for a font index is something that is unlikely to be assigned as an actual
/// index, to allow font index maps to be instantiated easily and safely using `default()`.
impl Default for Index {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

impl Index {
    /// Gets whether this index is the default, unset value.
    ///
    /// ```
    /// use crate::ugly::font::Index;
    ///
    /// assert!(Index::default().is_unset());
    /// ```
    #[must_use]
    pub const fn is_unset(&self) -> bool {
        self.0 == usize::MAX
    }
}

/// Trait of objects that can load and manipulate font data.
///
/// The lifetime `'l` is the lifetime of the loader when loading, and will usually also become the
/// lifetime of aspects of the `Data`.
pub trait Loader<'l> {
    /// The type of font data loaded by this loader.
    type Data;

    /// Loads font texture data from a path.
    ///
    /// # Errors
    ///
    /// Fails if the font cannot be loaded from `path`.
    fn load(&'l self, path: impl AsRef<Path>) -> Result<Self::Data>;

    /// Applies the given foreground colour to font texture data.
    fn colourise(&self, data: &mut Self::Data, fg: colour::Definition);
}

/// A backend-agnostic, cached font manager.
pub struct Manager<'a, Font, Fg, Load>
where
    Font: super::Map,
    Load: Loader<'a>,
    Fg: Map<colour::Definition>,
{
    /// Mapping from specs to already-cached font indices.
    slot_mapping: Font::IndexMap,
    /// The cache of already-loaded fonts.
    cache: Vec<Load::Data>,

    /// Loader for font data from PNG files.
    loader: &'a Load,

    /// The font path set.
    font_set: &'a Font,
    /// The font metrics set.
    metrics_set: Font::MetricsMap,
    /// The foreground colour set, used for setting up font colours.
    colour_set: &'a Fg,
}

impl<'a, Font, Fg, Load> Manager<'a, Font, Fg, Load>
where
    Font: super::Map,
    Fg: Map<colour::Definition>,
    Load: Loader<'a>,
    Fg::Id: Eq + Hash,
    Font::Id: Eq + Hash,
{
    /// Creates a font manager with the given texture creator and config maps.
    #[must_use]
    pub fn new(
        loader: &'a Load,
        font_set: &'a Font,
        metrics_set: Font::MetricsMap,
        colour_set: &'a Fg,
    ) -> Self {
        Self {
            loader,
            cache: Vec::new(),
            slot_mapping: Font::IndexMap::default(),
            font_set,
            metrics_set,
            colour_set,
        }
    }

    /// Gets a reference to this font manager's metrics set.
    pub fn metrics(&self) -> &Font::MetricsMap {
        &self.metrics_set
    }

    /// Gets the data for the given font specification.
    ///
    /// # Errors
    ///
    /// Returns an error if the spec does not point to a font.
    pub fn data(&mut self, spec: &Spec<Font::Id, Fg::Id>) -> Result<&mut Load::Data> {
        let id = spec.id;
        let mut index: Index = *self.slot_mapping.get(id);
        if index.is_unset() {
            // This is where we're about to add a new index.
            index = Index(self.cache.len());

            let path = &self.font_set.get(id).texture_path();
            let tex = self.loader.load(path)?;
            self.cache.push(tex);

            // TODO(@MattWindsor91): index usize::MAX should be off limits; we should raise an error
            self.slot_mapping.set(id, index);
        }

        let tex = &mut self.cache[index.0];
        self.loader
            .colourise(tex, *self.colour_set.get(spec.colour));
        Ok(tex)
    }
}
