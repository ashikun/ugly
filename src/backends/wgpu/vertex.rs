//! The definition of a (2D) vertex in the renderer, and shapes made out of them.
use crate::metrics::{Anchor, Rect};
use crate::{colour, metrics};
use itertools::Itertools;
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

/// A low level encoding of a shape:
/// a pre-calculated list of vertices and indices with a particular texture.
#[derive(Clone, Debug)]
pub(super) struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    texture: Rc<wgpu::Texture>,
}

impl Shape {
    /// Constructs a quad with the given first index, on-screen rectangle, and material.
    pub(super) fn quad(first_index: u16, screen_rect: Rect, material: Material<Rect>) -> Self {
        let anchors = [
            Anchor::BOTTOM_LEFT,
            Anchor::BOTTOM_RIGHT,
            Anchor::TOP_RIGHT,
            Anchor::TOP_LEFT,
        ];

        let vertices = anchors
            .into_iter()
            .map(|anchor| {
                Vertex::new(
                    screen_rect.anchor(anchor),
                    material.dimensions.anchor(anchor),
                    material.colour,
                )
            })
            .collect_vec();
        let indices = [0, 1, 2, 0, 2, 3]
            .iter()
            .map(|x| x + first_index)
            .collect_vec();

        Shape {
            vertices,
            indices,
            texture: material.texture,
        }
    }

    /// Gets the number of vertices referenced in this shape.
    pub(super) fn num_vertices(&self) -> Index {
        self.vertices.len() as Index
    }

    /// Gets the number of indices referenced in this shape.
    pub(super) fn num_indices(&self) -> u32 {
        self.indices.len() as u32
    }

    /// Gets an iterator over copies of all vertices in the shape.
    pub fn vertices<'a>(&'a self) -> impl Iterator<Item = Vertex> + 'a {
        self.vertices.iter().copied()
    }

    /// Gets an iterator over copies of all indices in the shape.
    pub fn indices<'a>(&'a self) -> impl Iterator<Item = Index> + 'a {
        self.indices.iter().copied()
    }

    /// Borrows the shape's texture.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

/// Type synonym for indices.
pub(crate) type Index = u16;

// TODO: make indices u32

/// A bundle of data about texturing and colouring for a [Shape].
pub(super) struct Material<D> {
    pub(super) colour: colour::Definition,
    pub(super) texture: Rc<wgpu::Texture>,
    pub(super) dimensions: D,
}
