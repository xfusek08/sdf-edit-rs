
use std::{marker::PhantomData, fmt::Debug};

use wgpu::util::DeviceExt;

use crate::warn;

use super::{
    Context,
    vertices::Vertex,
};

pub trait BufferItem:  {}

#[derive(Debug)]
pub struct Buffer<I: Debug + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable> {
    /// Label of buffer on GPU.
    pub label: Option<&'static str>,
    /// Vertex buffer on GPU.
    pub buffer: wgpu::Buffer,
    /// The number of items in the buffer.
    pub size: usize,
    /// Capacity of the buffer (how many items it can hold).
    pub capacity: usize,
    /// TODO: delete after wgpu 0.14
    pub usage: wgpu::BufferUsages,
    /// The type of the buffer item data.
    _phantom: PhantomData<I>,
}

// Statics (Helpers, Constructors)
impl<I: Debug + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable> Buffer<I> {
    /// Create a new buffer on the GPU.
    #[profiler::function]
    pub fn new(gpu: &Context, label: Option<&'static str>, data: &[I], usage: wgpu::BufferUsages) -> Buffer<I> {
        let size = data.len();
        let buffer = gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label, usage, contents: bytemuck::cast_slice(data) }
        );
        
        Buffer { label, buffer, size, capacity: size, usage, _phantom: PhantomData }
    }
    
    /// Create a new buffer on the GPU with a given capacity without initializing it.
    #[profiler::function]
    pub fn new_empty(gpu: &Context, label: Option<&'static str>, capacity: usize, usage: wgpu::BufferUsages) -> Buffer<I> {
        let size = 0;
        let buffer = gpu.device.create_buffer(
            &wgpu::BufferDescriptor {
                label,
                size: Self::bytes_for_item_count(capacity) as u64,
                usage,
                mapped_at_creation: false,
            }
        );
        Buffer { label, buffer, size, capacity, usage, _phantom: PhantomData }
    }
    
    /// Helper function to compute how many bytes will occupy given number of items in this buffer
    pub fn bytes_for_item_count(count: usize) -> usize {
        (count * std::mem::size_of::<I>()) as usize
    }
    
    /// Be ware that this panics when MAP_READ is not valid usage for the buffer.
    #[profiler::function]
    pub fn static_read(buffer: &wgpu::Buffer, gpu: &Context) -> Vec<I> {
        let data = {
            let buffer_slice = buffer.slice(..);
            profiler::call!(buffer_slice.map_async(wgpu::MapMode::Read, move |_| ()));
            profiler::call!(gpu.device.poll(wgpu::Maintain::Wait));
            let data = profiler::call!(buffer_slice.get_mapped_range());
            bytemuck::cast_slice(&data).to_vec()
        };
        profiler::call!(buffer.unmap());
        data
    }
}

// Instance methods
impl<I: Debug + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable> Buffer<I> {
    
    /// Returns allocated number of bytes (on GPU) for this buffer
    pub fn byte_size(&self) -> usize {
        // self.buffer.size() as usize // TODO: wgpu 0.14
        self.size * std::mem::size_of::<I>()
    }
    
    /// Returns the usage with which this buffer was created
    pub fn usage(&self) -> wgpu::BufferUsages {
        // self.buffer.usage() // TODO: wgpu 0.14
        self.usage
    }
    
    /// Update the buffer on the GPU using wgpu queue with the given data.
    /// - If the buffer is not large enough, it will be reallocated with the new size.
    /// - Returns true if the buffer was resized and thus the old bindings is invalid.
    #[profiler::function]
    pub fn queue_update(&mut self, gpu: &Context, new_data: &[I]) -> bool {
        self.size = new_data.len();
        if new_data.len() > self.capacity {
            // TODO: For some reason this might not work ... Assignment buffer is empty when first update went through this branch
            profiler::scope!("Updating Buffer with reallocation");
            warn!("Updating Buffer with reallocation {} -> {}", self.capacity, new_data.len());
            self.buffer = gpu.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: self.label,
                    usage: self.usage(),
                    contents: bytemuck::cast_slice(new_data),
                }
            );
            self.capacity = new_data.len();
            self.size     = new_data.len();
            return true;
        }
        profiler::scope!("Updating Buffer without reallocation");
        gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(new_data));
        false
    }
    
    /// If new capacity is larger than current capacity, resize the buffer.
    /// - This operation does not copy the old data to the new buffer.
    /// - Returns true if the buffer was resized and thus the old data is invalid.
    #[profiler::function]
    pub fn resize(&mut self, gpu: &Context, new_capacity: usize) -> bool {
        if new_capacity > self.capacity {
            self.buffer = gpu.device.create_buffer(
                &wgpu::BufferDescriptor {
                    label: self.label,
                    size:  Self::bytes_for_item_count(new_capacity) as u64,
                    usage: self.usage(),
                    mapped_at_creation: false,
                }
            );
            self.capacity = new_capacity;
            return true;
        }
        false
    }
    
    /// Be ware that this panics when MAP_READ is not valid usage for the buffer.
    pub fn read(&self, gpu: &Context) -> Vec<I> {
        Self::static_read(&self.buffer, gpu)
    }
}

// TODO Generalize Buffer for different usage types

/// Creates new vertex buffer on GPU from vertex data.
fn init_vertex_buffer<V: Vertex>(label: Option<&'static str>, vertices: &[V], context: &Context) -> wgpu::Buffer {
    context.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(vertices), // <- vertex buffer casted as array of bytes
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // <- mark this buffer to be used as vertex buffer and make it updatable
        }
    )
}

#[derive(Debug)]
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
    pub fn new<V: Vertex>(label: Option<&'static str>, vertices: &[V], context: &Context) -> Self {
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
    pub fn update<V: Vertex>(&mut self, context: &Context, vertices: &[V]) {
        dbg!("update vertex buffer");
        if vertices.len() > self.capacity {
            dbg!("update vertex buffer: resize");
            self.buffer = init_vertex_buffer(self.label, vertices, context);
            self.capacity = vertices.len();
        } else {
            dbg!("Updating vertex buffer");
            profiler::call!(
                context.queue.write_buffer(
                    &self.buffer,
                    0,
                    bytemuck::cast_slice(vertices)
                )
            );
        }
        self.size = vertices.len();
    }
    
}
