//! Notions of colour, colour maps, and so on.
//!
//! Currently, `ugly` supports only true-colour definitions presented as CSS colour
//! specifications.  Eventually, as `ugly` learns how to do text user interfaces, it might support
//! things like EGA colours more directly.

pub mod definition;
pub mod ega;
pub mod error;
pub mod spec;

use serde::{Deserialize, Serialize};

pub use definition::Definition;
pub use ega::{Ega, EGA};
pub use error::{Error, Result};
pub use spec::Spec;

/// A full colour palette, consisting of foreground and background colour maps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Palette<Fg, Bg> {
    /// Foreground colour space.
    pub fg: Fg,
    /// Background colour space.
    pub bg: Bg,
}
