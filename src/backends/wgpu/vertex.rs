//! The definition of a (2D) vertex in the renderer, and shapes made out of them.
use super::texture::Texture;
use crate::{
    colour,
    metrics::{self, Anchor, Rect},
};
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

/// A queue of shapes.
#[derive(Clone, Debug, Default)]
pub(super) struct ShapeQueue {
    /// The number of vertices pushed in this queue, used to determine base vertices.
    current_base: Index,

    /// The contents of the queue.
    contents: Vec<(Index, Shape)>,
}

impl ShapeQueue {
    /// Pushes a shape onto the shape queue.
    pub(super) fn push(&mut self, shape: Shape) {
        let next_base = self.current_base + (shape.vertices.len() as Index);
        self.contents.push((self.current_base, shape));
        self.current_base = next_base;
    }

    /// Clears the queue and returns the enqueued data.
    pub(super) fn take(&mut self) -> Vec<(Index, Shape)> {
        self.current_base = 0;
        std::mem::take(&mut self.contents)
    }
}

/// A low level encoding of a shape:
/// a pre-calculated list of vertices and indices with a particular texture.
#[derive(Clone, Debug)]
pub(super) struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    texture: Rc<Texture>,
}

impl Shape {
    /// Constructs a quad with the given on-screen rectangle and material.
    pub(super) fn quad(screen_rect: Rect, material: Material<Rect>) -> Self {
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
        let indices = vec![0, 1, 2, 0, 2, 3];

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
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

/// Type synonym for indices.
pub(crate) type Index = u16;

// TODO: make indices u32

/// A bundle of data about texturing and colouring for a [Shape].
pub struct Material<D> {
    pub colour: colour::Definition,
    pub texture: Rc<Texture>,
    pub dimensions: D,
}
