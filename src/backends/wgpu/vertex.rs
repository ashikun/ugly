//! The definition of a (2D) vertex in the renderer, and shapes made out of them.
use super::texture::Texture;
use crate::{
    colour,
    metrics::{self},
};
use std::rc::Rc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
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

    /// Gets the vertex buffer layout of a vertex.
    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }

    /// Constructs a vertex given its screen and texture coordinates as well as a colour definition.
    pub(super) fn new(
        screen_xy: metrics::Point,
        texture_xy: metrics::Point,
        colour: colour::Definition,
    ) -> Self {
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

/// Type synonym for indices.
pub(crate) type Index = u16;

// TODO: make indices u32

/// A bundle of data about texturing and colouring for a [Shape].
pub(super) struct Material<D> {
    pub(super) colour: colour::Definition,
    pub(super) texture: Rc<Texture>,
    pub(super) dimensions: D,
}
