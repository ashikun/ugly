//! Fonts, their metrics, and ways of loading and referring to them.

pub mod error;
pub mod metrics;
pub mod spec;

use std::{collections::HashMap, path::PathBuf};

pub use error::{Error, Result};
pub use metrics::Metrics;
pub use spec::{Id, Spec};

/// A font.
///
/// In `ugly`, a font is a directory containing two items: a texture file (PNG), and a metrics file
/// (RON).
#[derive(Clone, Debug)]
pub struct Font(std::path::PathBuf);

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

/// Shorthand for font maps.
pub type Map<FId> = HashMap<FId, Font>;

/// The metrics filename.
const METRICS_FILE: &str = "metrics.ron";
/// The texture filename.
const TEXTURE_FILE: &str = "font.png";
