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
    MetricsParse(#[from] ron::de::Error),

    /// Error loading a texture file.
    #[error("Error loading font texture")]
    TextureLoad(String),

    /// We tried to configure or use a font using a nonexistent ID.
    #[error("Font ID not recognised: {0}")]
    UnknownFont(String),

    /// We tried to use a width override to make a character larger than its bounding box.
    #[error("Can't override a char to be larger than its grid ({grid_width} < {override_width})")]
    OverlyLargeOverride {
        grid_width: crate::metrics::Length,
        override_width: crate::metrics::Length,
    },
}

impl Error {
    /// Constructs an unknown font error over `id`.
    #[must_use]
    pub fn unknown_font(id: impl super::Id) -> Self {
        // font IDs are debuggable, so we use that in the representation
        Self::UnknownFont(format!("{id:?}"))
    }
}

/// Shorthand for a result using [Error].
pub type Result<T> = std::result::Result<T, Error>;
