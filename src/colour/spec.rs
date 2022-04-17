//! Parseable representations of colours.
//!
//! This module primarily exposes [Spec], a representation of a colour that can be serialised and
//! deserialised with `serde`.
//!
//! [Spec] uses external libraries to support parsing of CSS4 colours; other forms of colour
//! input may appear in later versions of `ugly`.

use std::{fmt::Display, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{Error, Result};

/// A parseable representation of a colour.
#[derive(Copy, Clone, Debug, DeserializeFromStr, SerializeDisplay)]
pub struct Spec(css_color_parser::Color);

impl Display for Spec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Spec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Spec(s.parse()?))
    }
}

impl Spec {
    /// Converts a [Spec] into a definition.
    ///
    /// This conversion is lossy in that it truncates floating-point alpha values into a single
    /// integer byte.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ugly::colour::{Definition, Spec};
    ///
    /// let spec = Spec::from_str("transparent").unwrap();
    ///
    /// assert!(spec.into_definition().is_transparent());
    /// ```
    #[must_use]
    pub fn into_definition(self) -> super::Definition {
        super::Definition {
            r: self.0.r,
            g: self.0.g,
            b: self.0.b,
            a: convert_alpha(self.0.a),
        }
    }
}

/// Converts an alpha float (0.0..1.0) into an integer.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
#[must_use]
fn convert_alpha(a: f32) -> u8 {
    (255.0 * a).round() as u8
}
