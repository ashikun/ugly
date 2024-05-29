//! Font management.
//!
//! This part of the font subsystem deals with storing font texture data, predominantly.
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    path::Path,
};

use super::Result;

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

/// A backend-agnostic, cached font manager.
pub struct Manager<Font, Data>
where
    Font: super::Map,
{
    /// The cache of already-loaded fonts.
    cache: HashMap<Font::Id, Data>,

    /// The font path set.
    font_set: Font,
    /// The font metrics set.
    metrics_set: Font::MetricsMap,
}

impl<Font, Data> Manager<Font, Data>
where
    Font: super::Map,
    Font::Id: Eq + Hash,
{
    /// Creates a font manager with the given texture creator and config maps.
    #[must_use]
    pub fn new(font_set: Font, metrics_set: Font::MetricsMap) -> Self {
        Self {
            cache: HashMap::new(),
            font_set,
            metrics_set,
        }
    }

    /// Gets a reference to this font manager's metrics set.
    pub fn metrics(&self) -> &Font::MetricsMap {
        &self.metrics_set
    }

    /// Gets the data for the given font ID.
    ///
    /// If the font is not present, its texture will be loaded using `loader`.
    ///
    /// # Errors
    ///
    /// Returns an error if the spec does not point to a font.
    pub fn data(
        &mut self,
        id: Font::Id,
        mut loader: impl FnMut(&Path) -> Result<Data>,
    ) -> Result<&Data> {
        match self.cache.entry(id) {
            Entry::Occupied(slot) => Ok(slot.into_mut()),
            Entry::Vacant(slot) => {
                let path = &self.font_set.get(id).texture_path();
                let data = loader(path)?;

                Ok(slot.insert(data))
            }
        }
    }
}
