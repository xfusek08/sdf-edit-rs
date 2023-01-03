
use crate::framework::gpu;

#[derive(Debug)]
pub struct BrickInstances {
    count: Option<u32>,
    pub buffer: gpu::Buffer<u32>,
    pub count_buffer: gpu::Buffer<u32>,
}

impl BrickInstances {
    pub fn new(gpu: &gpu::Context, capacity: usize) -> Self {
        Self {
            count: Some(0),
            buffer: gpu::Buffer::new_empty(
                gpu,
                Some("Brick instances buffer"),
                capacity,
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE
                    
            ),
            count_buffer: gpu::Buffer::new(
                &gpu,
                Some("Brick instances counter buffer"),
                &[0],
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::MAP_READ, // TODO: this is needed for load_count to work. Will not be needed, when we will have indirect draws.
            ),
        }
    }
    
    pub fn count(&self) -> Option<u32> {
        self.count
    }
    
    pub fn clear(&mut self, gpu: &gpu::Context) {
        self.count = None;
        self.count_buffer.queue_update(gpu, &[0]);
    }
    
    pub fn clear_resize(&mut self, gpu: &gpu::Context, capacity: usize) {
        self.buffer.resize(gpu, capacity);
        self.clear(gpu);
    }
    
    pub fn load_count(&mut self, gpu: &gpu::Context) -> u32 {
        self.count.get_or_insert_with(|| {
            self.count_buffer.read(gpu)[0]
        }).clone()
    }
    
    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn create_bind_group(&self, gpu: &gpu::Context, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Brick instance buffer bind group"),
            layout: layout,
            entries: &[
                // Buffer with brick instances
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer.buffer.as_entire_binding(),
                },
                // Buffer with brick instances count
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.count_buffer.buffer.as_entire_binding(),
                },
            ],
        })
    }
    
    /// Creates and returns a custom binding for the node pool.
    #[profiler::function]
    pub fn create_bind_group_layout(gpu: &gpu::Context, visibility: wgpu::ShaderStages, read_only: bool) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Brick instance buffer bind group layout"),
            entries: &[
                // Buffer with brick instances
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Buffer with brick instances count
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}
