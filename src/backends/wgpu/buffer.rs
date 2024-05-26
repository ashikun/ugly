//! Code for setting up the buffers for the `wgpu` renderer.
use wgpu::util::DeviceExt;

use super::{
    instance::Instance,
    vertex::{Index, Vertex},
};

/// Creates the vertex buffer.
pub(super) fn create_vertex(device: &wgpu::Device) -> wgpu::Buffer {
    create(
        device,
        "Vertex Buffer",
        SizeFactor::<Vertex>::default(),
        wgpu::BufferUsages::VERTEX,
    )
}

/// Creates the index buffer.
pub(super) fn create_index(device: &wgpu::Device) -> wgpu::Buffer {
    create(
        device,
        "Index Buffer",
        SizeFactor::<Index>::default(),
        wgpu::BufferUsages::INDEX,
    )
}

/// Creates the instance buffer.
pub(super) fn create_instance(device: &wgpu::Device) -> wgpu::Buffer {
    create(
        device,
        "Instance Buffer",
        SizeFactor::<Instance>::default(),
        wgpu::BufferUsages::VERTEX,
    )
}

fn create<T>(
    device: &wgpu::Device,
    label: &str,
    size_factor: SizeFactor<T>,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let desc = wgpu::BufferDescriptor {
        label: Some(label),
        size: size_factor.buffer_size(),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&desc)
}

/// The initial buffer allocation, in multiples of `COPY_BUFFER_ALIGNMENT`.
const INITIAL_BUFFER_SIZE: wgpu::BufferAddress = 1024;

/// A sizing factor for constructing a buffer.
#[derive(Copy, Clone, Debug)]
pub struct SizeFactor<T> {
    factor: wgpu::BufferAddress,
    ty: std::marker::PhantomData<T>,
}

impl<T> SizeFactor<T> {
    const fn new(factor: wgpu::BufferAddress) -> Self {
        Self {
            factor,
            ty: std::marker::PhantomData,
        }
    }

    /// Calculates a safe buffer size holding `factor * COPY_BUFFER_ALIGNMENT` instances of `T`.
    const fn buffer_size(self) -> wgpu::BufferAddress {
        (std::mem::size_of::<T>() as wgpu::BufferAddress)
            * self.factor
            * wgpu::COPY_BUFFER_ALIGNMENT
    }
}

impl<T> Default for SizeFactor<T> {
    fn default() -> Self {
        Self::new(INITIAL_BUFFER_SIZE)
    }
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
    padding: u32,
    /// The current scale factor of the screen (not the textures).
    pub(super) scale_factor: f32,
}

impl Uniform {
    /// Updates the uniform buffer's screen size.
    pub(super) fn update_screen_size(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.screen_size[0] = size.width;
        self.screen_size[1] = size.height;
    }

    /// Updates the uniform buffer's scale factor.
    /// Ignores any obviously incorrect scale factors.
    pub(super) fn update_scale_factor(&mut self, scale_factor: f32) {
        if 0.0 < scale_factor {
            self.scale_factor = scale_factor;
        }
    }
}

/// A set of `wgpu` buffers.
pub(super) struct Set {
    pub(super) vertex: wgpu::Buffer,
    pub(super) index: wgpu::Buffer,
    pub(super) instance: wgpu::Buffer,
    pub(super) uniform: wgpu::Buffer,
}

impl Set {
    /// Creates a buffer set with the given device and initial uniform layout.
    pub(super) fn new(device: &wgpu::Device, uniform: Uniform) -> Self {
        Self {
            vertex: create_vertex(device),
            index: create_index(device),
            instance: create_instance(device),
            uniform: create_uniform(device, uniform),
        }
    }

    /// Populates the buffers from the given input.
    pub(super) fn populate(&self, queue: &wgpu::Queue, input: &Input) {
        queue.write_buffer(&self.vertex, 0, bytemuck::cast_slice(&input.vertices));
        queue.write_buffer(&self.index, 0, bytemuck::cast_slice(&input.indices));
        queue.write_buffer(&self.instance, 0, bytemuck::cast_slice(&input.instances));
    }
}

/// Inputs for populating a buffer set.
#[derive(Clone, Debug, Default)]
pub(super) struct Input {
    // TODO: is this inefficient?
    /// The list of vertices to push to the vertex buffer.
    pub(super) vertices: Vec<Vertex>,
    /// The list of indices to push to the index buffer.
    pub(super) indices: Vec<Index>,
    /// The list of instances to push to the instance buffer.
    pub(super) instances: Vec<Instance>,
}
