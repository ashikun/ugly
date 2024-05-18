//! Errors in the `wgpu` backend.
use thiserror::Error;

/// `wgpu` specific errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("surface creation error: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("image loading error: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no adapter available")]
    NoAdapterAvailable,
    #[error("device request error: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

/// Shorthand for `wgpu` results.
pub type Result<T> = std::result::Result<T, Error>;
