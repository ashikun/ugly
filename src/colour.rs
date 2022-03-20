//! Notions of colour, colour maps, and so on.
//!
//! Currently, `ugly` supports only true-colour definitions presented as CSS colour
//! specifications.  Eventually, as `ugly` learns how to do text user interfaces, it might support
//! things like EGA colours more directly.

pub mod definition;
pub mod error;
pub mod id;

pub use definition::{Definition, Map, MapSet};
pub use error::{Error, Result};
