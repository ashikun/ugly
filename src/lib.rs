/*! The Undead Graphics Library.

`ugly` is a work-in-progress pile of abstractions over, primarily, creating proportional pixel font
based user interfaces.  It doesn't do much yet, but we're working on that.
*/

#![warn(clippy::all, clippy::pedantic)]

pub mod backends;
pub mod colour;
pub mod error;
pub mod font;
pub mod metrics;
pub mod render;
pub mod text;

pub use error::{Error, Result};
