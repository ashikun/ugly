//! Rendering using `wgpu`.
use itertools::Itertools;
use std::path::Path;
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{IndexFormat, Instance, PipelineCompilationOptions};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::font::Spec;
use crate::metrics::{Anchor, Point, Rect};
use crate::{colour, font, resource, Error, Result};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    /// The position, in screen coordinates.
    screen_xy: [i32; 2],
    /// The texture coordinates, in terms of the texture itself.
    texture_xy: [i32; 2],
    /// The colour, as (0-255) linear RGBA.
    colour: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Sint32x2, 1 => Sint32x2, 2 => Float32x4];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    fn new(screen_xy: Point, texture_xy: Point, colour: colour::Definition) -> Self {
        Self {
            screen_xy: [screen_xy.x, screen_xy.y],
            texture_xy: [texture_xy.x, texture_xy.y],
            colour: [
                colour.r as f32,
                colour.g as f32,
                colour.b as f32,
                colour.a as f32,
            ],
        }
    }
}

/// The layout of the uniform buffer.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    /// The current screen size, in pixels.
    /// Used to convert screen coordinates to clip-space coordinates.
    screen_size: [u32; 2],
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

    current_index: u16,
    shapes: Vec<Shape>,
}

/// A low level encoding of a shape:
/// a pre-calculated list of vertices and indices with a particular texture.
#[derive(Clone, Debug)]
struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    texture: Rc<wgpu::Texture>,
}

impl Shape {
    fn quad(first_index: u16, screen_rect: Rect, material: Material<Rect>) -> Self {
        let anchors = [
            Anchor::BOTTOM_LEFT,
            Anchor::BOTTOM_RIGHT,
            Anchor::TOP_RIGHT,
            Anchor::TOP_LEFT,
        ];

        let vertices: Vec<Vertex> = anchors
            .into_iter()
            .map(|anchor| {
                Vertex::new(
                    screen_rect.anchor(anchor),
                    material.dimensions.anchor(anchor),
                    material.colour,
                )
            })
            .collect();
        let indices: Vec<u16> = [0, 1, 2, 0, 2, 3].iter().map(|x| x + first_index).collect();

        Shape {
            vertices,
            indices,
            texture: material.texture,
        }
    }
}

struct Material<D> {
    colour: colour::Definition,
    texture: Rc<wgpu::Texture>,
    dimensions: D,
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
        // Make a texture rect whose coordinates will always be negative
        let tex_rect = Rect::new(-2, -2, 1, 1);

        let material = Material {
            colour: self.lookup_bg(colour),
            texture: self.core.null_texture(),
            dimensions: tex_rect,
        };

        self.push_shape(|i| Shape::quad(i, rect, material));

        Ok(())
    }

    fn clear(&mut self, colour: Bg::Id) -> Result<()> {
        /* We clear at the beginning of every rendering cycle anyway, so
         * 'clear' is tantamount to changing the colour we clear to.
         */
        self.bg = self.lookup_bg(colour);

        Ok(())
    }

    fn present(&mut self) {
        let shapes = std::mem::take(&mut self.shapes);
        self.current_index = 0;

        self.core.render(self.bg, &shapes);
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
            bg: colour::Definition::default(),
            fonts,
            metrics,
            palette,
            core,
            current_index: 0,
            shapes: vec![],
        };

        Ok(result)
    }

    fn push_shape(&mut self, shape_fn: impl FnOnce(u16) -> Shape) {
        let shape = shape_fn(self.current_index);

        self.current_index += shape.vertices.len() as u16;
        self.shapes.push(shape);
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
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    null_texture: Rc<wgpu::Texture>,

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

        let adapter = create_adapter(instance, &surface).await?;

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

        let config = create_surface_config(&surface, &adapter, size);
        surface.configure(&device, &config);

        let vertex_buffer = create_vertex_buffer(&device);
        let index_buffer = create_index_buffer(&device);

        let uniform = Uniform {
            screen_size: [size.width, size.height],
        };
        let uniform_buffer = create_uniform_buffer(&device, uniform);

        let uniform_bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
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
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&uniform_bind_group_layout_desc);

        let uniform_bind_group_desc = wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        };
        let uniform_bind_group = device.create_bind_group(&uniform_bind_group_desc);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
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

        let null_texture = Rc::new(create_texture(
            &device,
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        ));

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            null_texture,
            size,
        })
    }

    pub fn null_texture(&self) -> Rc<wgpu::Texture> {
        self.null_texture.clone()
    }

    pub fn load_image(&self, path: impl AsRef<Path>) -> Result<Rc<wgpu::Texture>> {
        let reader = image::io::Reader::open(path).map_err(|e| Error::Backend(e.to_string()))?;
        let image = reader.decode().map_err(|e| Error::Backend(e.to_string()))?;
        let rgba = image.to_rgba8();

        let (width, height) = rgba.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = create_texture(&self.device, size);

        self.queue.write_texture(
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

        Ok(Rc::new(texture))
    }

    pub fn render(&self, bg: colour::Definition, shapes: &[Shape]) {
        self.prepare_buffers(shapes);

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
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            let mut cur_index: u32 = 0;

            for shape in shapes {
                let next_index = cur_index + (shape.indices.len() as u32) + 1;

                render_pass.draw_indexed(cur_index..next_index, 0, 0..1);

                cur_index = next_index - 1;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    fn prepare_buffers(&self, shapes: &[Shape]) {
        let vertices = shapes
            .iter()
            .flat_map(|s| s.vertices.iter().copied())
            .collect_vec();
        let indices = shapes
            .iter()
            .flat_map(|s| s.indices.iter().copied())
            .collect_vec();

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        let uniform = Uniform {
            screen_size: [self.size.width, self.size.height],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

async fn create_adapter<'w>(
    instance: Instance,
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
        .ok_or_else(|| Error::Backend("no adapter available".to_string()))?;

    Ok(adapter)
}

fn create_surface_config(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    size: PhysicalSize<u32>,
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

fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let desc = wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: (std::mem::size_of::<Vertex>() as wgpu::BufferAddress)
            * 256
            * wgpu::COPY_BUFFER_ALIGNMENT,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&desc)
}

fn create_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let desc = wgpu::BufferDescriptor {
        label: Some("Index Buffer"),
        size: (std::mem::size_of::<u16>() as wgpu::BufferAddress)
            * 256
            * wgpu::COPY_BUFFER_ALIGNMENT,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&desc)
}

fn create_uniform_buffer(device: &wgpu::Device, initial: Uniform) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&initial),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&desc)
}

fn create_texture(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
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
