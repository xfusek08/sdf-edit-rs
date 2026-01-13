use crate::{framework::gpu, warn};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrickInstance {
    /// Id of a brick(node) in a particular SVO
    brick_id: u32,
    /// Id of a instance of particular SVO
    instance_id: u32,
}

#[derive(Debug)]
pub struct BrickInstances {
    pub count: Option<u32>,
    pub buffer: gpu::Buffer<BrickInstance>,
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
                    | wgpu::BufferUsages::STORAGE,
            )
            .with_grow_rate(1.5),
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

    #[profiler::function]
    pub fn clear(&mut self, gpu: &gpu::Context) {
        self.count = None;
        self.count_buffer.queue_update(gpu, &[0]);
    }

    /// Returns true if any of the buffers was recreated.
    #[profiler::function]
    pub fn clear_resize(&mut self, gpu: &gpu::Context, capacity: usize) -> bool {
        self.clear(gpu);
        self.buffer.resize(gpu, capacity)
    }

    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn create_bind_group(
        &self,
        gpu: &gpu::Context,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let max_binding_size_bytes = gpu.device.limits().max_storage_buffer_binding_size as usize;
        let capacity_bytes = self.buffer.capacity_bytes();

        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Brick instance buffer bind group"),
            layout: layout,
            entries: &[
                // Buffer with brick instances
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffer.buffer,
                        offset: 0,
                        size: if max_binding_size_bytes > capacity_bytes {
                            None
                        } else {
                            warn!("Brick instances buffer size is too big to be bound: {}. Limiting to {}", capacity_bytes, max_binding_size_bytes);
                            wgpu::BufferSize::new(max_binding_size_bytes as u64)
                        },
                    }),
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
    pub fn create_bind_group_layout(
        gpu: &gpu::Context,
        visibility: wgpu::ShaderStages,
        read_only: bool,
    ) -> wgpu::BindGroupLayout {
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
