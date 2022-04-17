//! Colour definitions.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// A true-colour definition.
///
/// The default colour is transparent black.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Definition {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Display for Definition {
    /// Formats a colour in #rrggbbaa format.
    ///
    /// ```
    /// use ugly::colour;
    ///
    /// assert_eq!("#aa5500ff", colour::EGA.dark.yellow.to_string())
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl Definition {
    /// Convenience constructor for RGBA colours.
    ///
    /// ```
    /// use ugly::colour::Definition;
    ///
    /// let col = Definition::rgba(12, 34, 56, 127);
    /// assert_eq!(12, col.r);
    /// assert_eq!(34, col.g);
    /// assert_eq!(56, col.b);
    /// assert_eq!(127, col.a);
    /// ```
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Constructs a colour using bytes for red, green, and blue components (and full alpha).
    ///
    /// ```
    /// use ugly::colour::Definition;
    ///
    /// let col = Definition::rgb(12, 34, 56);
    /// assert_eq!(12, col.r);
    /// assert_eq!(34, col.g);
    /// assert_eq!(56, col.b);
    /// assert_eq!(255, col.a);
    /// ```
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Gets whether this colour is transparent.
    ///
    /// ```
    /// assert!(ugly::colour::definition::TRANSPARENT.is_transparent());
    /// ```
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

/// A transparent black colour.
///
/// This is effectively a `const` synonym for `default`.
///
/// ```
/// use ugly::colour::definition;
///
/// let col = definition::TRANSPARENT;
/// assert_eq!(0, col.r);
/// assert_eq!(0, col.g);
/// assert_eq!(0, col.b);
/// assert_eq!(0, col.a);
///
/// assert_eq!(definition::Definition::default(), col);
/// ```
pub const TRANSPARENT: Definition = Definition::rgba(0, 0, 0, 0);

/// Pair of foreground and background colour maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSet<Fg, Bg> {
    /// Foreground colour space.
    pub fg: Fg,
    /// Background colour space.
    pub bg: Bg,
}
