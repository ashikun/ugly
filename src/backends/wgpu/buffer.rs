//! Code for setting up the buffers for the `wgpu` renderer.
use super::vertex::Vertex;
use wgpu::util::DeviceExt;

/// Creates the vertex buffer.
pub(super) fn create_vertex(device: &wgpu::Device) -> wgpu::Buffer {
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

/// Creates the index buffer.
pub(super) fn create_index(device: &wgpu::Device) -> wgpu::Buffer {
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

/// Creates the uniform buffer using the initial data from `initial`.
pub(super) fn create_uniform(device: &wgpu::Device, initial: Uniform) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::bytes_of(&initial),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&desc)
}

/// The layout of the uniform buffer.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Uniform {
    /// The current screen size, in pixels.
    /// Used to convert screen coordinates to clip-space coordinates.
    pub(super) screen_size: [u32; 2],
}

impl Uniform {
    /// Updates the uniform buffer
    pub(super) fn update_screen_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.screen_size[0] = size.width;
        self.screen_size[1] = size.height;
    }
}
