//! Backends for `ugly`.
//!
//! At the time of writing, only one backend exists: SDL2.

#[cfg(feature = "backend_sdl")]
pub mod sdl;
