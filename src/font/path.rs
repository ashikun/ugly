//! Pathing logic for font directories.
//!
//! In `ugly`, a font is a directory containing two items: a texture file (PNG), and a metrics file
//! (TOML).  This module contains the [Path] type, which wraps path buffers with functionality on
//! how to load these files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{metrics::Metrics, Result};

/// A font directory path (wrapping a `PathBuf`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Path(pub std::path::PathBuf);

impl Path {
    /// Constructs the path to the font's texture.
    #[must_use]
    pub fn texture_path(&self) -> PathBuf {
        self.0.join(TEXTURE_FILE)
    }

    /// Resolves the path to the font's metrics file and tries to load it.
    ///
    /// # Errors
    ///
    /// Returns an error if the font metrics file is unreachable or unparseable
    /// as TOML.
    pub fn metrics(&self) -> Result<Metrics> {
        let str = std::fs::read_to_string(self.0.join(METRICS_FILE))?;
        Ok(toml::from_str(&str)?)
    }
}

/// Shorthand for path maps.
pub type Map<FId> = HashMap<FId, Path>;

/// The metrics filename.
const METRICS_FILE: &str = "metrics.toml";
/// The texture filename.
const TEXTURE_FILE: &str = "font.png";
