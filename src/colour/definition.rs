//! Colour definitions.

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::collections::HashMap;
use std::{fmt::Display, str::FromStr};

use super::error::{Error, Result};

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

/// EGA base palette without intensity.
pub struct EgaBase {
    pub black: Definition,
    pub blue: Definition,
    pub green: Definition,
    pub cyan: Definition,
    pub red: Definition,
    pub magenta: Definition,
    pub yellow: Definition,
    pub white: Definition,
}

/// EGA palette with intensity.
pub struct Ega {
    pub dark: EgaBase,
    pub bright: EgaBase,
}

/// The default EGA palette.
pub const EGA: Ega = Ega {
    dark: EgaBase {
        black: Definition::rgb(0x00, 0x00, 0x00),
        blue: Definition::rgb(0x00, 0x00, 0xAA),
        green: Definition::rgb(0x00, 0xAA, 0x00),
        cyan: Definition::rgb(0x00, 0xAA, 0xAA),
        red: Definition::rgb(0xAA, 0x00, 0x00),
        magenta: Definition::rgb(0xAA, 0x00, 0xAA),
        yellow: Definition::rgb(0xAA, 0x55, 0x00),
        white: Definition::rgb(0xAA, 0xAA, 0xAA),
    },
    bright: EgaBase {
        black: Definition::rgb(0x55, 0x55, 0x55),
        blue: Definition::rgb(0x55, 0x55, 0xFF),
        green: Definition::rgb(0x55, 0xFF, 0x55),
        cyan: Definition::rgb(0x55, 0xFF, 0xFF),
        red: Definition::rgb(0xFF, 0x55, 0x55),
        magenta: Definition::rgb(0xFF, 0x55, 0xFF),
        yellow: Definition::rgb(0xFF, 0xFF, 0x55),
        white: Definition::rgb(0xFF, 0xFF, 0xFF),
    },
};

/// Shorthand for hash-maps containing definitions.
pub type Map<Id> = HashMap<Id, Definition>;

/// Pair of foreground and background colour maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSet<Fg, Bg> {
    /// Foreground colour space.
    #[serde(bound(
        serialize = "Fg: super::id::Fg + Serialize",
        deserialize = "Fg: super::id::Fg + Deserialize<'de>"
    ))]
    pub fg: Map<Fg>,
    /// Background colour space.
    #[serde(bound(
        serialize = "Bg: super::id::Bg + Serialize",
        deserialize = "Bg: super::id::Bg + Deserialize<'de>"
    ))]
    pub bg: Map<Bg>,
}

impl<Fg: super::id::Fg, Bg: super::id::Bg> MapSet<Fg, Bg> {
    /// Gets the foreground at `id`, substituting white if `id` is not in the map.
    #[must_use]
    pub fn fg_or_white(&self, id: Fg) -> Definition {
        fg_or_white(&self.fg, id)
    }

    /// Gets the background at `id`, substituting black if `id` is not in the map.
    #[must_use]
    pub fn bg_or_black(&self, id: Bg) -> Definition {
        self.bg
            .get(&id)
            .copied()
            .unwrap_or(super::definition::EGA.dark.black)
    }
}

/// Gets the foreground at `id`, substituting white if `id` is not in the map.
#[must_use]
pub fn fg_or_white<Fg: super::id::Fg>(map: &Map<Fg>, id: Fg) -> Definition {
    map.get(&id)
        .copied()
        .unwrap_or(super::definition::EGA.bright.white)
}
