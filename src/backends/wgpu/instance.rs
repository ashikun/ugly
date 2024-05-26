/// The layout of one element in the instance buffer.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(in crate::backends::wgpu) struct Instance {
    /// The position delta (X and Y).
    pub(in crate::backends::wgpu) delta: [i32; 2],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![3 => Sint32x2];

    /// Gets the vertex buffer layout of an instance.
    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
