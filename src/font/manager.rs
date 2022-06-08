//! Font management.
//!
//! This part of the font subsystem deals with storing font texture data, predominantly.
use std::{collections::HashMap, hash::Hash, path::Path, rc::Rc};

use super::{
    super::{colour, resource::Map},
    Error, Result, Spec,
};

/// An index for a loaded font in a [Manager].
///
/// The index may be arbitrary, as long as it can be traded in for a valid font.
#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct Index(pub usize);

/// Trait of abstract font managers.
///
/// This trait deliberately does not leak the actual font image data stored inside the manager.
///
/// The lifetime `'f` captures the lifetime requirements on fetching a font (for instance, the
/// loader might need to last within a larger lifetime).
pub trait Manager<'f, Font: super::Map, Fg: Map<colour::Definition>> {
    /// Asks the manager to fetch a font by specification, returning an index that can be used for
    /// fast access to font particulars later.
    ///
    /// This might re-use the same font if it has already been loaded.
    ///
    /// # Errors
    ///
    /// Fails if the font cannot be loaded.
    fn fetch(&'f mut self, font: Spec<Font::Id, Fg::Id>) -> Result<Index>;

    /// Gets the metrics of a font.
    fn metrics(&self) -> &Font::MetricsMap;
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
    fn colourise(&self, data: Self::Data, fg: colour::Definition) -> Self::Data;
}

/// A backend-agnostic, cached font manager.
pub struct Cached<'a, Font, Fg, Load>
where
    Font: super::Map,
    Load: Loader<'a>,
    Fg: Map<colour::Definition>,
{
    /// Mapping from specs to already-cached font indices.
    slot_mapping: HashMap<Spec<Font::Id, Fg::Id>, Index>,
    /// The cache of already-loaded fonts.
    cache: Vec<Rc<Load::Data>>,

    /// Loader for font data from PNG files.
    loader: Load,

    /// The font path set.
    font_set: &'a Font,
    /// The font metrics set.
    metrics_set: Font::MetricsMap,
    /// The foreground colour set, used for setting up font colours.
    colour_set: &'a Fg,
}

impl<'a, Font, Fg, Load> Manager<'a, Font, Fg> for Cached<'a, Font, Fg, Load>
where
    Font: super::Map,
    Fg: Map<colour::Definition>,
    Load: Loader<'a>,
    Font::Id: Eq + Hash,
    Fg::Id: Eq + Hash,
{
    fn fetch(&'a mut self, spec: Spec<Font::Id, Fg::Id>) -> Result<Index> {
        // This font might have already been loaded.
        self.slot_mapping
            .get(&spec)
            .copied()
            .map_or_else(|| self.fetch_uncached(&spec), Ok)
    }

    fn metrics(&self) -> &Font::MetricsMap {
        &self.metrics_set
    }
}

impl<'a, Font, Fg, Load> Cached<'a, Font, Fg, Load>
where
    Font: super::Map,
    Fg: Map<colour::Definition>,
    Load: Loader<'a>,
    Fg::Id: Eq + Hash,
    Font::Id: Eq + Hash,
{
    fn fetch_uncached(&'a mut self, spec: &Spec<Font::Id, Fg::Id>) -> Result<Index> {
        let id = spec.id;
        let path = &self.font_set.get(id).texture_path();

        let tex = self.loader.load(path)?;
        let col_tex = self
            .loader
            .colourise(tex, *self.colour_set.get(spec.colour));

        self.cache.push(Rc::new(col_tex));

        let index = Index(self.cache.len() - 1);
        self.slot_mapping.insert(*spec, index);

        Ok(index)
    }
}

impl<'a, Font, Fg, Load> Cached<'a, Font, Fg, Load>
where
    Font: super::Map,
    Fg: Map<colour::Definition>,
    Load: Loader<'a>,
    Font::Id: Eq + Hash,
    Fg::Id: Eq + Hash,
{
    /// Creates a font manager with the given texture creator and config maps.
    #[must_use]
    pub fn new(
        loader: Load,
        font_set: &'a Font,
        metrics_set: Font::MetricsMap,
        colour_set: &'a Fg,
    ) -> Self {
        Self {
            loader,
            cache: Vec::new(),
            slot_mapping: HashMap::new(),
            font_set,
            metrics_set,
            colour_set,
        }
    }

    /// Gets the data for the given font handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the handle does not point to a font.
    pub fn data(&mut self, handle: Index) -> Result<Rc<Load::Data>> {
        self.cache
            .get(handle.0)
            .cloned()
            .ok_or(Error::BadHandle(handle))
    }
}
