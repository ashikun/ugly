//! The core of the `wgpu` rendering backend.
use crate::{
    backends::wgpu::{
        buffer, init, texture,
        vertex::{Shape, Vertex},
    },
    colour, Error, Result,
};
use itertools::Itertools;
use std::{path::Path, rc::Rc};

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

    size: winit::dpi::PhysicalSize<u32>,
    uniform: buffer::Uniform,
}

impl<'a> Core<'a> {
    /// Constructs a `wgpu` renderer over the given window.
    ///
    /// # Errors
    ///
    /// Fails if any part of the wgpu bring-up fails.
    pub async fn new(window: &'a winit::window::Window) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|e| Error::Backend(e.to_string()))?;

        let adapter = init::create_adapter(instance, &surface).await?;

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

        let config = init::create_surface_config(&surface, &adapter, size);
        surface.configure(&device, &config);

        let vertex_buffer = buffer::create_vertex(&device);
        let index_buffer = buffer::create_index(&device);

        let uniform = buffer::Uniform {
            screen_size: [size.width, size.height],
        };
        let uniform_buffer = buffer::create_uniform(&device, uniform);

        let uniform_bind_group_layout = init::create_uniform_bind_group_layout(&device);
        let uniform_bind_group =
            init::create_uniform_bind_group(&device, &uniform_buffer, &uniform_bind_group_layout);

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
        let pipeline = device.create_render_pipeline(&pipeline_desc);

        let null_texture = Rc::new(texture::create(
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
            uniform,
            uniform_buffer,
            uniform_bind_group,
            null_texture,
            size,
        })
    }

    /// Notifies the rendering core of a change to the window size.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.update_uniform();
    }

    fn update_uniform(&mut self) {
        self.uniform.update_screen_size(self.size);

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniform));
    }

    pub(super) fn null_texture(&self) -> Rc<wgpu::Texture> {
        self.null_texture.clone()
    }

    pub(super) fn load_image(&self, path: impl AsRef<Path>) -> crate::Result<Rc<wgpu::Texture>> {
        let tex = texture::load(&self.device, &self.queue, path)?;

        Ok(Rc::new(tex))
    }

    pub(super) fn render(&self, bg: colour::Definition, shapes: &[Shape]) {
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            let mut cur_index: u32 = 0;

            for shape in shapes {
                let next_index = cur_index + shape.num_indices() + 1;

                render_pass.draw_indexed(cur_index..next_index, 0, 0..1);

                cur_index = next_index - 1;
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    fn prepare_buffers(&self, shapes: &[Shape]) {
        let vertices = shapes.iter().flat_map(Shape::vertices).collect_vec();
        let indices = shapes.iter().flat_map(Shape::indices).collect_vec();

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
    }
}
