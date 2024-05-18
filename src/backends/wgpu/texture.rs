//! Texture creation and bookkeeping facilities.
use super::{Error, Result};

/// Loads an image as a texture.
///
/// # Errors
///
/// Fails if the image cannot be loaded or decoded.
///
/// # Panics
///
/// May panic if something fails at the GPU level.
pub(super) fn load(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: impl AsRef<std::path::Path>,
) -> Result<wgpu::Texture> {
    let reader = image::io::Reader::open(path)?;
    let image = reader.decode()?;
    let rgba = image.to_rgba8();

    let (width, height) = rgba.dimensions();

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = create(device, size);

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    Ok(texture)
}

/// Creates a texture on the given device with the given extends and sensible settings.
pub(super) fn create(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("image_texture"),
        view_formats: &[],
    };
    device.create_texture(&desc)
}
