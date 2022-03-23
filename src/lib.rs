//! The Undead Graphics Library.
//!
//! `ugly` is a work-in-progress pile of abstractions over, primarily, creating proportional pixel
//! font based user interfaces.  It doesn't do much yet, but we're working on that.

#![warn(clippy::all, clippy::pedantic)]

pub mod backends;
pub mod colour;
pub mod error;
pub mod font;
pub mod metrics;
pub mod render;
pub mod text;

// Generally, we re-export anything where the name would stutter; these tend to be the most
// important type in the respective module anyway.

pub use error::{Error, Result};
pub use font::Font;
pub use render::Renderer;
