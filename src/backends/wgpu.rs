use crate::colour;

mod buffer;
mod core;
mod error;
mod font;
mod init;
mod instance;
mod render;
mod shape;
mod texture;
mod vertex;
pub mod winit;

pub use {
    core::Core,
    error::{Error, Result},
    render::Renderer,
};

/// Losslessly converts an `ugly` colour to a (linear) `wgpu` one.
impl From<colour::Definition> for wgpu::Color {
    fn from(value: colour::Definition) -> Self {
        Self {
            r: f64::from(value.r) / 255.0,
            g: f64::from(value.g) / 255.0,
            b: f64::from(value.b) / 255.0,
            a: f64::from(value.a) / 255.0,
        }
    }
}
