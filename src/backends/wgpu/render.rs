//! Rendering using `wgpu`.
use std::mem::size_of;
use wgpu::core::resource::StagingBuffer;
use wgpu::util::DeviceExt;
use wgpu::{BufferAddress, Color, IndexFormat, PipelineCompilationOptions, COPY_BUFFER_ALIGNMENT};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::font::Spec;
use crate::metrics::{Anchor, Point, Rect};
use crate::{colour, font, resource, Error, Result};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    colour: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Renderer<
    'a,
    Font: font::Map,
    Fg: resource::Map<colour::Definition>,
    Bg: resource::Map<colour::Definition>,
> {
    bg: colour::Definition,
    fonts: Font,
    metrics: Font::MetricsMap,
    palette: colour::Palette<Fg, Bg>,

    pub core: Core<'a>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl<
        'a,
        Font: font::Map,
        Fg: resource::Map<colour::Definition>,
        Bg: resource::Map<colour::Definition>,
    > crate::Renderer<'a, Font, Fg, Bg> for Renderer<'a, Font, Fg, Bg>
{
    fn font_metrics(&self) -> &Font::MetricsMap {
        &self.metrics
    }

    fn write(&mut self, font: Spec<Font::Id, Fg::Id>, str: &font::layout::String) -> Result<()> {
        Ok(())
    }

    fn fill(&mut self, rect: Rect, colour: Bg::Id) -> Result<()> {
        let colour = self.lookup_bg(colour);

        let Point { x: x1, y: y1 } = rect.anchor(Anchor::TOP_LEFT);
        let Point { x: x2, y: y2 } = rect.anchor(Anchor::BOTTOM_RIGHT);

        // TODO: safely cast or drop length to 16
        let x1 = self.convert_x(x1);
        let x2 = self.convert_x(x2);
        let y1 = self.convert_y(y1);
        let y2 = self.convert_y(y2);

        // TODO: sRGB conversion
        let colour = [
            f32::from(colour.r),
            f32::from(colour.g),
            f32::from(colour.b),
            f32::from(colour.a),
        ];

        let base = self.vertices.len() as u16;
        self.vertices.extend([
            Vertex {
                colour,
                position: [x1, y1],
            },
            Vertex {
                colour,
                position: [x2, y1],
            },
            Vertex {
                colour,
                position: [x2, y2],
            },
            Vertex {
                colour,
                position: [x1, y2],
            },
        ]);

        self.indices
            .extend([0, 1, 2, 0, 2, 3].iter().map(|x| x + base));

        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        self.bg = self.lookup_bg(colour);
        Ok(())
    }

    fn present(&mut self) {
        let vertices = std::mem::take(&mut self.vertices);
        let indices = std::mem::take(&mut self.indices);

        self.core.render(self.bg, &vertices, &indices);
    }
}

impl<
        'a,
        Font: font::Map,
        Fg: resource::Map<colour::Definition>,
        Bg: resource::Map<colour::Definition>,
    > Renderer<'a, Font, Fg, Bg>
{
    /// Constructs a new `wgpu` renderer.
    ///
    /// # Errors
    ///
    /// Fails if we can't load font metrics.
    pub fn new(fonts: Font, palette: colour::Palette<Fg, Bg>, core: Core<'a>) -> Result<Self> {
        let metrics = fonts.load_metrics()?;

        let result = Self {
            bg: Default::default(),
            fonts,
            metrics,
            palette,
            core,
            vertices: vec![],
            indices: vec![],
        };

        Ok(result)
    }

    fn convert_x(&self, x: crate::metrics::Length) -> f32 {
        // TODO: move this to the shader
        let w = f64::from(self.core.size.width);
        let x = f64::from(x);

        let x = (x / (w * 0.5)) - 1.0;

        x as f32
    }

    fn convert_y(&self, y: crate::metrics::Length) -> f32 {
        // TODO: move this to the shader
        let h = f64::from(self.core.size.height);
        let y = f64::from(y);

        let y = (y / (h * 0.5)) - 1.0;

        y as f32
    }

    /// Looks up a background colour.
    fn lookup_bg(&self, id: Bg::Id) -> colour::Definition {
        *self.palette.bg.get(id)
    }

    /// Looks up a foreground colour.
    fn lookup_fg(&self, id: Fg::Id) -> colour::Definition {
        *self.palette.fg.get(id)
    }
}

/// The core of the `wgpu` renderer.
pub struct Core<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    size: PhysicalSize<u32>,
}

impl<'a> Core<'a> {
    /// Constructs a `wgpu` renderer over the given window.
    ///
    /// # Errors
    ///
    /// Fails if any part of the wgpu bring-up fails.
    pub async fn new(window: &'a Window) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|e| Error::Backend(e.to_string()))?;

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .ok_or_else(|| Error::Backend("no adapter available".to_string()))?;

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        };
        let (device, queue) = adapter
            .request_device(&device_desc, None)
            .await
            .map_err(|e| Error::Backend(e.to_string()))?;

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let vertex_buffer_desc = wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (size_of::<Vertex>() as BufferAddress) * 256 * COPY_BUFFER_ALIGNMENT,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let vertex_buffer = device.create_buffer(&vertex_buffer_desc);

        let index_buffer_desc = wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (size_of::<u16>() as BufferAddress) * 256 * COPY_BUFFER_ALIGNMENT,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let index_buffer = device.create_buffer(&index_buffer_desc);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);
        let fragment_state_targets = [Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &fragment_state_targets,
                compilation_options: PipelineCompilationOptions::default(),
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
        let pipeline = device.create_render_pipeline(&pipeline_desc);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            size,
        })
    }

    pub fn render(&self, bg: colour::Definition, vertices: &[Vertex], indices: &[u16]) {
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw_indexed(0..((indices.len() as u32) + 1), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
