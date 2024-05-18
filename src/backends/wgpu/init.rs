//! High-level initialisation functions for `wgpu`.
//!
//! Initialisation functions for buffers belong in the `buffer` module.
use super::{Error, Result};

/// Creates a `wgpu` adapter.
pub(super) async fn create_adapter<'w>(
    instance: wgpu::Instance,
    surface: &wgpu::Surface<'w>,
) -> Result<wgpu::Adapter> {
    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    };
    let adapter = instance
        .request_adapter(&adapter_options)
        .await
        .ok_or_else(|| Error::NoAdapterAvailable)?;

    Ok(adapter)
}

/// Creates configuration suitable for the given surface, adapter, and screen size.
pub(super) fn create_surface_config(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .unwrap_or(surface_caps.formats[0]);
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

/// Creates the bind group layout for the renderer's uniform buffer.
pub(super) fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let desc = wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    };
    device.create_bind_group_layout(&desc)
}

/// Creates the bind group layout for the renderer's uniform buffer.
pub(super) fn create_uniform_bind_group(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    let uniform_bind_group_desc = wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("uniform_bind_group"),
    };
    device.create_bind_group(&uniform_bind_group_desc)
}
