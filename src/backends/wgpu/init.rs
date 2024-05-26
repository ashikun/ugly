//! High-level initialisation functions for `wgpu`.
//!
//! Initialisation functions for buffers belong in the `buffer` module.
use super::{instance::Instance, vertex::Vertex, Error, Result};

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

pub(super) fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let desc = wgpu::BindGroupLayoutDescriptor {
        entries: &[
            TEXTURE_BIND_GROUP_LAYOUT_ENTRY,
            SAMPLER_BIND_GROUP_LAYOUT_ENTRY,
        ],
        label: Some("texture_bind_group_layout"),
    };
    device.create_bind_group_layout(&desc)
}

const TEXTURE_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
    binding: 0,
    // Needs to be visible from the vertex shader too, so we can translate coordinates
    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
    ty: wgpu::BindingType::Texture {
        multisampled: false,
        view_dimension: wgpu::TextureViewDimension::D2,
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
    },
    count: None,
};

const SAMPLER_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
    binding: 1,
    visibility: wgpu::ShaderStages::FRAGMENT,
    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    count: None,
};

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

pub(super) fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    surface_config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let fragment_state_targets = [Some(wgpu::ColorTargetState {
        format: surface_config.format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
    let pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc(), Instance::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &fragment_state_targets,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    };
    device.create_render_pipeline(&pipeline_desc)
}
