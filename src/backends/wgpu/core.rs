//! The core of the `wgpu` rendering backend.
use std::sync::Arc;
use std::{path::Path, rc::Rc};
use wgpu::{CommandEncoder, RenderPass, TextureView};

use crate::colour;

use super::{
    buffer, init, shape,
    texture::{self, Texture},
    Result,
};

/// The core of the `wgpu` renderer.
pub struct Core {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,

    buffers: buffer::Set,
    uniform_bind_group: wgpu::BindGroup,

    textures: texture::Manager,

    uniform: buffer::Uniform,
}

impl Core {
    /// Constructs a `wgpu` renderer over the given window.
    ///
    /// # Errors
    ///
    /// Fails if any part of the wgpu bring-up fails.
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;

        let adapter = init::create_adapter(instance, &surface).await?;

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        };
        let (device, queue) = adapter.request_device(&device_desc, None).await?;

        let size = window.inner_size();

        let config = init::create_surface_config(&surface, &adapter, size);
        surface.configure(&device, &config);

        let mut uniform = buffer::Uniform::default();
        uniform.update_screen_size(size);
        uniform.update_scale_factor(window.scale_factor() as f32);

        let buffers = buffer::Set::new(&device, uniform);

        let uniform_bind_group_layout = init::create_uniform_bind_group_layout(&device);
        let uniform_bind_group =
            init::create_uniform_bind_group(&device, &buffers.uniform, &uniform_bind_group_layout);

        let textures = texture::Manager::new(&device);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &textures.texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);
        let pipeline = init::create_pipeline(&device, &pipeline_layout, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            buffers,
            uniform_bind_group,
            textures,
            uniform,
        })
    }

    /// Notifies the rendering core of a change to the window size.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.uniform.update_screen_size(new_size);
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        self.update_uniform();
    }

    /// Notifies the rendering core of a change to the window scale factor.
    pub fn rescale(&mut self, new_scale: f32) {
        self.uniform.update_scale_factor(new_scale);

        self.update_uniform();
    }

    fn update_uniform(&mut self) {
        self.queue
            .write_buffer(&self.buffers.uniform, 0, bytemuck::bytes_of(&self.uniform));
    }

    pub(super) fn null_texture(&self) -> Rc<Texture> {
        self.textures.null_texture.clone()
    }

    pub(super) fn load_image(&mut self, path: impl AsRef<Path>) -> Result<Rc<Texture>> {
        let tex = Texture::load(&self.device, &self.queue, path)?;

        self.textures.register_bind_group(&self.device, &tex);

        Ok(Rc::new(tex))
    }

    pub(super) fn render(
        &self,
        bg: colour::Definition,
        buffers: &buffer::Input,
        manifests: Vec<shape::Manifest>,
    ) {
        self.buffers.populate(&self.queue, buffers);

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
            let mut render_pass = self.create_render_pass(bg, &view, &mut encoder);

            let mut cur_texture_id: Option<wgpu::Id<wgpu::Texture>> = None;

            for manifest in manifests {
                let new_texture = manifest.texture;
                let new_texture_id = new_texture.contents.global_id();
                let old_texture_id = cur_texture_id.replace(new_texture_id);
                if old_texture_id != cur_texture_id {
                    // The texture has changed since the last shape.

                    let texture_bind_group = self.textures.get_bind_group(&new_texture).unwrap();
                    render_pass.set_bind_group(1, texture_bind_group, &[]);
                }

                render_pass.draw_indexed(
                    manifest.indices,
                    manifest.base_vertex,
                    manifest.instances,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    fn create_render_pass<'b>(
        &'b self,
        bg: colour::Definition,
        view: &'b TextureView,
        encoder: &'b mut CommandEncoder,
    ) -> RenderPass<'b> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
        render_pass.set_index_buffer(self.buffers.index.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
        render_pass.set_vertex_buffer(1, self.buffers.instance.slice(..));
        render_pass
    }
}
