//! Backends for `ugly`.
//!
//! At the time of writing, only one backend exists: SDL2, enabled by the `sdl2` feature and residing in the `ugly::backends::sdl` module.

#[cfg(feature = "sdl2")]
pub mod sdl;
