//! Errors raised by the font subsystem.

use thiserror::Error;

/// A font error.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred while loading a font file.
    #[error("IO error reading font file")]
    Io(#[from] std::io::Error),

    /// An error occurred while loading a metrics file.
    #[error("Error parsing metrics file")]
    MetricsParse(#[from] ron::de::SpannedError),

    /// Error loading a texture file.
    #[error("Error loading font texture")]
    TextureLoad(String),

    /// We tried to use a width override to make a character larger than its bounding box.
    #[error("Can't override a char to be larger than its grid ({grid_width} < {override_width})")]
    OverlyLargeOverride {
        grid_width: crate::metrics::Length,
        override_width: crate::metrics::Length,
    },

    #[error("Problem compiling kerning tables for font")]
    Kerning(#[from] super::metrics::kerning::Error),

    #[error("Tried to use invalid font handle")]
    BadHandle(super::Index),
}

/// Shorthand for a result using [enum@Error].
pub type Result<T> = std::result::Result<T, Error>;
