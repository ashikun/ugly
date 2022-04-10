//! Colour definitions.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::{
    super::resource,
    ega::EGA,
    error::{Error, Result},
};

/// A true-colour definition.
#[derive(Copy, Clone, Debug, DeserializeFromStr, SerializeDisplay)]
pub struct Definition(css_color_parser::Color);

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Definition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Definition(s.parse()?))
    }
}

impl Definition {
    /// Constructs a colour using bytes for red, green, and blue components (and full alpha).
    ///
    /// ```
    /// use ugly::colour::Definition;
    ///
    /// let col = Definition::rgb(12, 34, 56);
    /// assert_eq!(12, col.red_byte());
    /// assert_eq!(34, col.green_byte());
    /// assert_eq!(56, col.blue_byte());
    /// assert_eq!(255, col.alpha_byte());
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(css_color_parser::Color { r, g, b, a: 1.0 })
    }

    /// Gets the red component of this colour as a byte.
    #[must_use]
    pub const fn red_byte(&self) -> u8 {
        self.0.r
    }

    /// Gets the green component of this colour as a byte.
    #[must_use]
    pub const fn green_byte(&self) -> u8 {
        self.0.g
    }

    /// Gets the blue component of this colour as a byte.
    #[must_use]
    pub const fn blue_byte(&self) -> u8 {
        self.0.b
    }

    /// Gets the alpha component of this colour as a byte.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[must_use]
    pub fn alpha_byte(&self) -> u8 {
        (255.0 * self.0.a).round() as u8
    }
}

/// Pair of foreground and background colour maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSet<Fg, Bg> {
    /// Foreground colour space.
    pub fg: Fg,
    /// Background colour space.
    pub bg: Bg,
}

impl<Fg: resource::Map<Definition>, Bg: resource::Map<Option<Definition>>> MapSet<Fg, Bg> {
    /// Gets the background at `id`, substituting black if `id` is not in the map.
    #[must_use]
    pub fn bg_or_black(&self, id: Bg::Id) -> Definition {
        self.bg.get(id).unwrap_or(EGA.dark.black)
    }
}
