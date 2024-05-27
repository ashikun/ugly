//! Fonts, their metrics, and ways of loading and referring to them.
//!
//! The main type is [Font], which refers to a directory filled with font files.  Of these, one
//! file is a serialisation of the font's [Metrics]; another is its texture (a PNG).
//!
//! Consumers of `ugly` supply two sets of identifiers that describe a particular font and its
//! intended foreground colour.  To refer to such a [Font] in the `ugly` system, we have two main
//! types.  A [Spec] refers to a combination of font and colour IDs, and is parametric.  An
//! [Index] represents a cached, resolved index into a rendering backend's own font tables, and is
//! just a fancy `usize`.  Backends will supply a means for loading [Spec]s (producing [Index]es)
//! and rendering text using [Index]es.

pub mod error;
pub mod layout;
pub mod manager;
pub mod metrics;

use std::path::PathBuf;

pub use error::{Error, Result};
pub use manager::{Index, Manager};
pub use metrics::Metrics;

/// A font.
///
/// In `ugly`, a font is a directory containing two items: a texture file (PNG), and a metrics file
/// (RON).
#[derive(Clone, Debug)]
pub struct Font(PathBuf);

impl Font {
    /// Creates a font that refers to the contents of a directory at `path`.
    #[must_use]
    pub fn from_dir(path: impl AsRef<std::path::Path>) -> Self {
        Self(path.as_ref().to_path_buf())
    }

    /// Constructs the path to the font's texture (a PNG).
    ///
    /// Different backends use the texture in different ways, and it is often easier for them to
    /// load the file directly rather than take in bytes, so we don't expose high-level 'load
    /// this texture' functionality here.
    ///
    /// # Example
    ///
    /// ```
    /// let path = ugly::font::Font::from_dir("test").texture_path();
    /// assert_eq!(["test", "font.png"].iter().collect::<std::path::PathBuf>(), path);
    /// ```
    #[must_use]
    pub fn texture_path(&self) -> PathBuf {
        self.0.join(TEXTURE_FILE)
    }

    /// Resolves the path to the font's metrics file and tries to load it.
    ///
    /// # Errors
    ///
    /// Returns an error if the font metrics file is unreachable or unparseable as RON.
    pub fn metrics(&self) -> Result<Metrics> {
        let str = std::fs::read_to_string(self.0.join(METRICS_FILE))?;
        let spec: metrics::Spec = ron::from_str(&str)?;
        spec.into_metrics()
    }
}

/// Trait for font resource maps.
pub trait Map: super::resource::Map<Font> {
    /// The type of metrics maps produced by following this map.
    type MetricsMap: super::resource::Map<Metrics, Id = Self::Id>;

    /// The type of font index maps produced by following this map.

    type IndexMap: super::resource::MutableMap<Index, Id = Self::Id> + Default;

    /// Loads metrics for all fonts in the map.
    ///
    /// # Errors
    ///
    /// Fails if any of the font metrics files is missing.
    fn load_metrics(&self) -> Result<Self::MetricsMap>;
}

impl<K: Copy + Clone + Default + std::hash::Hash + Eq> Map
    for super::resource::DefaultingHashMap<K, Font>
{
    type MetricsMap = super::resource::DefaultingHashMap<K, Metrics>;
    type IndexMap = super::resource::DefaultingHashMap<K, Index>;

    fn load_metrics(&self) -> Result<Self::MetricsMap> {
        let map = self
            .iter()
            .map(|(k, v)| Ok((*k, v.metrics()?)))
            .collect::<Result<std::collections::HashMap<_, _>>>()?;
        Ok(super::resource::DefaultingHashMap::new(
            map,
            Metrics::default(),
        ))
    }
}

/// The metrics filename.
const METRICS_FILE: &str = "metrics.ron";
/// The texture filename.
const TEXTURE_FILE: &str = "font.png";
