
use std::{
    marker::PhantomData,
    fmt::Debug
};
use wgpu::util::DeviceExt;
use crate::{info, warn};
use super::{
    Context,
    vertices::Vertex,
};

pub trait BufferItem:  {}

#[derive(Debug)]
pub struct Buffer<I: Debug + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable> {
    /// Label of buffer on GPU.
    pub label: Option<&'static str>,
    /// Vertex buffer on GPU.self.
    pub buffer: wgpu::Buffer,
    /// The number of items in the buffer.
    pub size: usize,
    /// Capacity of the buffer (how many items it can hold).
    pub capacity: usize,
    /// TODO: delete after wgpu 0.14
    pub usage: wgpu::BufferUsages,
    /// The rate at which the buffer will grow when it is full.
    pub grow_rate: f32,
    /// The type of the buffer item data.
    _phantom: PhantomData<I>,
}

// Statics (Helpers, Constructors)
impl<I: Debug + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable> Buffer<I> {
    
    /// Create a new buffer on the GPU.
    #[profiler::function]
    pub fn new(
        gpu: &Context,
        label: Option<&'static str>,
        data: &[I],
        usage: wgpu::BufferUsages,
    ) -> Buffer<I> {
        let size = data.len();
        let buffer = gpu.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label,
                usage,
                contents:
                bytemuck::cast_slice(data)
            }
        );
        Buffer {
            label,
            buffer,
            size,
            capacity: size,
            usage,
            grow_rate: 1.0,
            _phantom: PhantomData,
        }
    }
    
    /// Create a new buffer on the GPU with a given capacity without initializing it.
    #[profiler::function]
    pub fn new_empty(
        gpu: &Context,
        label: Option<&'static str>,
        capacity: usize,
        usage: wgpu::BufferUsages,
    ) -> Buffer<I> {
        let buffer = gpu.device.create_buffer(
            &wgpu::BufferDescriptor {
                label,
                size: Self::padded_bytes_for_item_count(capacity) as u64,
                usage,
                mapped_at_creation: false,
            }
        );
        Buffer {
            size: 0,
            buffer,
            label,
            capacity,
            usage,
            grow_rate: 1.0,
            _phantom: PhantomData,
        }
    }
    
    pub fn with_grow_rate(mut self, grow_rate: f32) -> Self {
        self.grow_rate = grow_rate;
        self
    }
    
    /// Helper function to compute how many bytes will occupy given number of items in this buffer
    #[inline]
    pub fn bytes_for_item_count(count: usize) -> usize {
        (count * std::mem::size_of::<I>()) as usize
    }
    
    /// Helper function to compute real required size of buffer for given number of items
    ///   - For more info why it is done this way see `wgpu::DeviceExt::create_buffer_init`.
    #[inline]
    pub fn padded_bytes_for_item_count(count: usize) -> usize {
        let unpadded_size = Self::bytes_for_item_count(count) as u64;
        let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
        ((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT) as usize
    }
    
    /// Returns how many items at most can be stored in this buffer.
    pub fn max_capacity(gpu: &Context, usage: wgpu::BufferUsages) -> usize {
        let max_capacity_bytes = if usage.contains(wgpu::BufferUsages::STORAGE) {
            gpu.device.limits().max_storage_buffer_binding_size
        } else if usage.contains(wgpu::BufferUsages::UNIFORM) {
            gpu.device.limits().max_uniform_buffer_binding_size
        } else {
            0
        } as usize;
        max_capacity_bytes / std::mem::size_of::<I>()
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
    
    fn calculate_resize_capacity(&self, gpu: &Context, new_size: usize) -> usize {
        let max_capacity = Self::max_capacity(gpu, self.usage);
        
        let new_capacity = if self.grow_rate > 0.0 && self.grow_rate != 1.0 {
            (new_size as f32 * self.grow_rate) as usize
        } else {
            new_size
        };
        
        if new_capacity > max_capacity {
            warn!("Buffer: \"{}\": Cannot grow buffer to {} items, because it exceeds the maximum capacity of {} items - trimming", self.label(), new_capacity, max_capacity);
            max_capacity
        } else {
            new_capacity
        }
    }
    
    fn label(&self) -> &str {
        self.label.unwrap_or("Unnamed Buffer")
    }
    
    /// Returns allocated number of bytes (on GPU) for this buffer
    #[inline]
    pub fn byte_size(&self) -> usize {
        Self::bytes_for_item_count(self.size)
    }
    
    #[inline]
    pub fn capacity_bytes(&self) -> usize {
        Self::bytes_for_item_count(self.capacity)
    }
    
    /// Update the buffer on the GPU using wgpu queue with the given data.
    /// - If the buffer is not large enough, it will be reallocated with the new size.
    /// - Returns true if the buffer was resized and thus the old bindings is invalid.
    #[profiler::function]
    pub fn queue_update(&mut self, gpu: &Context, new_data: &[I]) -> bool {
        info!("Buffer \"{}\": Updating data: {}/{} -> {}/{}", self.label(), self.size, self.capacity, new_data.len(), self.capacity);
        
        // Reallocate if too small
        if new_data.len() > self.capacity {
            profiler::scope!("Updating Buffer with reallocation");
            let new_capacity = self.calculate_resize_capacity(gpu, new_data.len());
            let data_slice: &[u8] = bytemuck::cast_slice(new_data);
            info!("Buffer: \"{}\": Reallocating with new capacity and data {}/{} -> {}/{}", self.label(), self.size, self.capacity, new_data.len(), new_capacity);
            self.buffer = self.new_buffer(gpu, new_capacity, true);
            self.buffer.slice(..)
                .get_mapped_range_mut()[..data_slice.len()]
                .copy_from_slice(data_slice);
            self.buffer.unmap();
            self.size = new_data.len();
            self.capacity = new_capacity;
            return true;
        }
        
        profiler::scope!("Updating Buffer without reallocation");
        gpu.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(new_data));
        self.size = new_data.len();
        false
    }
    
    /// If new capacity is larger than current capacity, resize the buffer.
    /// - This operation does not copy the old data to the new buffer.
    /// - Returns true if the buffer was resized and thus the old data is invalid.
    #[profiler::function]
    pub fn resize(&mut self, gpu: &Context, new_capacity: usize) -> bool {
        
        // Reallocate if too small
        if new_capacity > self.capacity {
            let recalculated_capacity = self.calculate_resize_capacity(gpu, new_capacity);
            info!("Buffer \"{}\": Reallocating with new capacity {}/{} -> {}/{} (asked for {})", self.label(), self.size, self.capacity, self.size, recalculated_capacity, new_capacity);
            self.buffer = self.new_buffer(gpu, recalculated_capacity, false);
            self.capacity = new_capacity;
            return true;
        }
        false
    }
    
    fn new_buffer(&mut self, gpu: &Context, capacity: usize, mapped: bool) -> wgpu::Buffer {
        gpu.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: self.label,
                size: Self::padded_bytes_for_item_count(capacity) as u64,
                usage: self.usage,
                mapped_at_creation: mapped,
            }
        )
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
