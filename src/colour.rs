//! Notions of colour, colour maps, and so on.
//!
//! Currently, `ugly` supports only true-colour definitions presented as CSS colour
//! specifications.  Eventually, as `ugly` learns how to do text user interfaces, it might support
//! things like EGA colours more directly.

pub mod definition;
pub mod ega;
pub mod error;

pub use definition::{Definition, MapSet};
pub use ega::{Ega, EGA};
pub use error::{Error, Result};
