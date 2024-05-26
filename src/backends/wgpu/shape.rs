use std::{ops::Range, rc::Rc};

use itertools::Itertools;

use crate::metrics::{Anchor, Rect};

use super::{
    buffer,
    instance::Instance,
    texture::Texture,
    vertex::{Index, Material, Vertex},
};

/// A queue of shapes.
#[derive(Clone, Debug, Default)]
pub(super) struct Queue {
    /// The buffer inputs to send to the buffers.
    buffer_inputs: buffer::Input,

    /// The manifests to convert to draw commands.
    manifests: Vec<Manifest>,
}

/// A manifest for converting a shape directly into a draw command.
#[derive(Clone, Debug)]
pub(super) struct Manifest {
    /// The base vertex for this draw.
    pub(super) base_vertex: i32,
    /// The texture to load.
    pub(super) texture: Rc<Texture>,
    /// The index range to use.
    pub(super) indices: Range<u32>,
    /// The instance range to use.
    pub(super) instances: Range<u32>,
}

impl Queue {
    /// Pushes a shape onto the shape queue.
    pub(super) fn push(&mut self, mut shape: Shape) {
        // TODO: compress similar data (i.e. same instance, same mesh, etc)
        // also compress like shapes into one shape

        // Make sure there's at least one instance.
        if shape.instances.is_empty() {
            shape.instances.push(Instance::default());
        }

        let base_vertex = self.buffer_inputs.vertices.len() as i32;
        let base_index = self.buffer_inputs.indices.len() as u32;
        let base_instance = self.buffer_inputs.instances.len() as u32;

        // TODO: maybe just send these straight to the buffer?
        self.buffer_inputs.vertices.extend(shape.vertices);
        self.buffer_inputs.indices.extend(shape.indices);
        self.buffer_inputs.instances.extend(shape.instances);

        let next_base_index = self.buffer_inputs.indices.len() as u32;
        let next_base_instance = self.buffer_inputs.instances.len() as u32;

        let manifest = Manifest {
            base_vertex,
            texture: shape.texture,
            indices: (base_index..next_base_index),
            instances: (base_instance..next_base_instance),
        };

        self.manifests.push(manifest);
    }

    /// Clears the queue and returns the enqueued data.
    pub(super) fn take(&mut self) -> (buffer::Input, Vec<Manifest>) {
        let buffer_inputs = std::mem::take(&mut self.buffer_inputs);
        let manifests = std::mem::take(&mut self.manifests);

        (buffer_inputs, manifests)
    }
}

/// A low level encoding of a shape:
/// a pre-calculated list of vertices and indices with a particular texture.
#[derive(Clone, Debug)]
pub(super) struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    texture: Rc<Texture>,
    instances: Vec<Instance>,
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
            instances: vec![],
        }
    }

    /// Adds instances to this shape.
    pub(super) fn instanced(mut self, instances: Vec<Instance>) -> Self {
        self.instances = instances;
        self
    }
}
