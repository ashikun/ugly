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
