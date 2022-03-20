//! The SDL2 backend for `ugly`.
//!
//! SDL2 is the reference backend, and so virtually everything `ugly` exposes is supported by it.

pub mod font;
pub mod manager;
pub mod metrics;
pub mod render;

pub use manager::Manager;
pub use render::Renderer;
