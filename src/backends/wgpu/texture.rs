//! Texture creation and bookkeeping facilities.
use super::{init, Result};
use std::collections::HashMap;
use std::rc::Rc;

/// A texture and its attached view.
#[derive(Debug)]
pub(super) struct Texture {
    pub(super) texture: wgpu::Texture,
    pub(super) view: wgpu::TextureView,
}

impl Texture {
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
    ) -> Result<Self> {
        let reader = image::io::Reader::open(path)?;
        let image = reader.decode()?;
        let rgba = image.to_rgba8();

        let (width, height) = rgba.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = Self::create(device, size);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
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
    pub(super) fn create(device: &wgpu::Device, size: wgpu::Extent3d) -> Self {
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
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}

/// A texture manager.
pub(super) struct Manager {
    pub(super) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) texture_bind_groups: HashMap<wgpu::Id<wgpu::Texture>, wgpu::BindGroup>,
    pub(super) sampler: wgpu::Sampler,
    pub(super) null_texture: Rc<Texture>,
}

impl Manager {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let sampler = create_sampler(&device);
        let texture_bind_group_layout = init::create_texture_bind_group_layout(&device);
        let null_texture = Rc::new(Texture::create(
            &device,
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        ));

        let mut texture_bind_groups = HashMap::new();
        texture_bind_groups.insert(
            null_texture.texture.global_id(),
            create_texture_bind_group(
                device,
                &null_texture.view,
                &sampler,
                &texture_bind_group_layout,
            ),
        );

        Self {
            sampler,
            texture_bind_group_layout,
            texture_bind_groups,
            null_texture,
        }
    }

    /// Registers a bind group for `texture`.
    pub(super) fn register_bind_group(&mut self, device: &wgpu::Device, texture: &Texture) {
        let id = texture.texture.global_id();

        if self.texture_bind_groups.contains_key(&id) {
            return;
        }

        self.texture_bind_groups.insert(
            id,
            create_texture_bind_group(
                device,
                &texture.view,
                &self.sampler,
                &self.texture_bind_group_layout,
            ),
        );
    }

    /// Gets the bind group previously registered for `texture`.
    pub(super) fn get_bind_group(&self, texture: &Texture) -> Option<&wgpu::BindGroup> {
        self.texture_bind_groups.get(&texture.texture.global_id())
    }
}

pub(super) fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    let desc = wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    };

    device.create_sampler(&desc)
}

pub(super) fn create_texture_bind_group(
    device: &wgpu::Device,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let desc = wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("texture_bind_group"),
    };
    device.create_bind_group(&desc)
}
