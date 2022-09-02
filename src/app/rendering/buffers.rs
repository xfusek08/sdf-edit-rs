use wgpu::util::DeviceExt;

use super::{RenderContext, vertices::Vertex};

// TODO Generalize Buffer for different usage types

/// Creates new vertex buffer on GPU from vertex data.
fn init_vertex_buffer<V: Vertex>(label: Option<&'static str>, vertices: &[V], context: &RenderContext) -> wgpu::Buffer {
    context.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(vertices), // <- vertex buffer casted as array of bytes
            usage: wgpu::BufferUsages::VERTEX, // <- mark this buffer to be used as vertex buffer
        }
    )
}

pub struct VertexBuffer {
    /// Label of buffer on GPU.
    pub label: Option<&'static str>,
    /// Vertex buffer on GPU.
    pub buffer: wgpu::Buffer,
    /// The number of vertices in the buffer.
    pub size: usize,
    /// Capacity of the buffer (how many vertices it can hold).
    pub capacity: usize,
}

impl VertexBuffer {
    /// Create a new vertex buffer.
    #[profiler::function]
    pub fn new<V: Vertex>(label: Option<&'static str>, vertices: &[V], context: &RenderContext) -> Self {
        Self {
            label,
            buffer: init_vertex_buffer(label, vertices, context),
            size: vertices.len(),
            capacity: vertices.len(),
        }
    }
    
    /// Update the buffer with new data.
    ///  - After update old buffer reference does not make sense, hence self is moved into this method.
    #[profiler::function]
    pub fn update<V: Vertex>(&mut self, context: &RenderContext, vertices: &[V]) {
        if vertices.len() > self.capacity {
            self.buffer = init_vertex_buffer(self.label, vertices, context);
            self.capacity = vertices.len();
        } else {
            context.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(vertices)
            );
        }
        self.size = vertices.len();
    }
    
}
