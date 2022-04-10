//! EGA palettes.
//!
//! This is both an example of how one might implement a colour resource map, and also a useful
//! set of defaults for loading into your own colour resource maps.
use super::{super::resource, Definition};

/// EGA base palette without intensity.
pub struct Base {
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
    pub dark: Base,
    pub bright: Base,
}

/// The default EGA palette.
pub const EGA: Ega = Ega {
    dark: Base {
        black: Definition::rgb(0x00, 0x00, 0x00),
        blue: Definition::rgb(0x00, 0x00, 0xAA),
        green: Definition::rgb(0x00, 0xAA, 0x00),
        cyan: Definition::rgb(0x00, 0xAA, 0xAA),
        red: Definition::rgb(0xAA, 0x00, 0x00),
        magenta: Definition::rgb(0xAA, 0x00, 0xAA),
        yellow: Definition::rgb(0xAA, 0x55, 0x00),
        white: Definition::rgb(0xAA, 0xAA, 0xAA),
    },
    bright: Base {
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

/// The base identifier set for EGA.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BaseId {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Yellow,
    White,
}

/// The bright/dark identifier set for EGA.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Bright(BaseId),
    Dark(BaseId),
}

/// The default base EGA identifier is white, as we assume that we're targeting foreground.
impl Default for BaseId {
    fn default() -> Self {
        BaseId::White
    }
}

/// The default EGA identifier is bright white, as we assume that we're targeting foreground.
impl Default for Id {
    fn default() -> Self {
        Self::Bright(BaseId::White)
    }
}

/// We can use [Base] as a foreground colour map.
impl resource::Map<Definition> for Base {
    type Id = BaseId;

    fn get(&self, k: Self::Id) -> &Definition {
        match k {
            BaseId::Black => &self.black,
            BaseId::Blue => &self.blue,
            BaseId::Green => &self.green,
            BaseId::Cyan => &self.cyan,
            BaseId::Red => &self.red,
            BaseId::Magenta => &self.magenta,
            BaseId::Yellow => &self.yellow,
            BaseId::White => &self.white,
        }
    }
}

/// We can use [Ega] as a foreground colour map.
impl resource::Map<Definition> for Ega {
    type Id = Id;

    fn get(&self, k: Self::Id) -> &Definition {
        match k {
            Id::Bright(k) => self.bright.get(k),
            Id::Dark(k) => self.dark.get(k),
        }
    }
}
