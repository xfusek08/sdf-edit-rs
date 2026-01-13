use super::Capacity;
use crate::framework::gpu;
use wgpu::util::DeviceExt;

#[cfg(debug_assertions)]
#[derive(Debug)]
struct ResourceLabels {
    count_buffer: String,
    header_buffer: String,
    payload_buffer: String,
    vertex_buffer: String,
    capacity_buffer: String,
}

/// A Node Pool of the SVO residing on GPU.
#[derive(Debug)]
pub struct NodePool {
    /// A label of the Node Pool
    svo_name: String,

    #[cfg(debug_assertions)]
    resource_labels: ResourceLabels,

    /// A total number of nodes that can be stored currently in the buffers.
    capacity: u32,

    /// In this buffer number of nodes in SVO is stored.
    /// - It is used for atomic increments in shaders
    count_buffer: wgpu::Buffer,

    /// Buffer of first 32 bytes of nodes, where each integer contains:
    ///   - 1 bit - is subdivided flag
    ///   - 1 bit - is has brick linked flag
    ///   - 30 bits - child node tile index
    header_buffer: wgpu::Buffer,

    /// Buffer of second 32 bytes of nodes, where each integer contains:
    ///   ether: (when node has linked brick)
    ///     - 2 bit padding
    ///     - 10x10x10 (x,y,z) of 10 bit indices into brick bool
    ///   or: (if node does not have linked brick)
    ///     - 8x8x8x8 of rgba bytes
    payload_buffer: wgpu::Buffer,

    /// Buffer of node positions, where each node has its own vec4 (4xf32) vertex value:
    ///   - 3x f32 - position
    ///   - 1x f32 - size
    vertex_buffer: wgpu::Buffer,

    /// A uniform buffer holding value for capacity of node pool.
    capacity_buffer: wgpu::Buffer,

    /// A value into which a count information is read from count_buffer.
    /// - If None, `load_count` method must be call to populate this value.
    count: Option<u32>,
}

// getters
impl NodePool {
    pub fn count(&self) -> Option<u32> {
        self.count
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
    pub fn count_buffer(&self) -> &wgpu::Buffer {
        &self.count_buffer
    }
    pub fn header_buffer(&self) -> &wgpu::Buffer {
        &self.header_buffer
    }
    pub fn payload_buffer(&self) -> &wgpu::Buffer {
        &self.payload_buffer
    }
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }
    pub fn capacity_buffer(&self) -> &wgpu::Buffer {
        &self.capacity_buffer
    }
    pub fn buffers_changed(&mut self) {
        self.count = None;
    }
}

impl NodePool {
    /// Creates empty GPU octree by allocating buffers on GPU.
    #[profiler::function]
    pub fn new(svo_name: String, gpu: &gpu::Context, capacity: Capacity) -> Self {
        #[cfg(debug_assertions)]
        let resource_labels = ResourceLabels {
            count_buffer: format!("{} - Node Pool Count Buffer", svo_name),
            header_buffer: format!("{} - Node Pool Header Buffer", svo_name),
            payload_buffer: format!("{} - Node Pool Payload Buffer", svo_name),
            vertex_buffer: format!("{} - Node Pool Vertex Buffer", svo_name),
            capacity_buffer: format!("{} - Node Pool Capacity Buffer", svo_name),
        };

        let capacity = capacity.nodes();
        let capacity64 = capacity as u64;

        let count = 0u32;
        let count_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                #[cfg(debug_assertions)]
                label: Some(&resource_labels.count_buffer),
                #[cfg(not(debug_assertions))]
                label: None,
                contents: bytemuck::cast_slice(&[count]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            });

        let header_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            #[cfg(debug_assertions)]
            label: Some(&resource_labels.header_buffer),
            #[cfg(not(debug_assertions))]
            label: None,
            size: capacity64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let payload_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            #[cfg(debug_assertions)]
            label: Some(&resource_labels.payload_buffer),
            #[cfg(not(debug_assertions))]
            label: None,
            size: capacity64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let vertex_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            #[cfg(debug_assertions)]
            label: Some(&resource_labels.vertex_buffer),
            #[cfg(not(debug_assertions))]
            label: None,
            size: capacity64 * std::mem::size_of::<glam::Vec4>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let capacity_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                #[cfg(debug_assertions)]
                label: Some(&resource_labels.capacity_buffer),
                #[cfg(not(debug_assertions))]
                label: None,
                contents: bytemuck::cast_slice(&[capacity]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        Self {
            svo_name,
            capacity,
            count_buffer,
            header_buffer,
            payload_buffer,
            vertex_buffer,
            capacity_buffer,
            count: Some(count),
            #[cfg(debug_assertions)]
            resource_labels,
        }
    }

    /// Reads value from count buffer on GPU into internal `count` property and returns its value.
    #[profiler::function]
    pub fn load_count(&mut self, gpu: &gpu::Context) -> u32 {
        self.count
            .get_or_insert_with(|| gpu::Buffer::<u32>::static_read(&self.count_buffer, gpu)[0])
            .clone()
    }

    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn create_bind_group(
        &self,
        gpu: &gpu::Context,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SVO Node Pool Bind Group"),
            layout: layout,
            entries: &[
                // node_count
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.count_buffer().as_entire_binding(),
                },
                // node_headers
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.header_buffer().as_entire_binding(),
                },
                // node_payload
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.payload_buffer().as_entire_binding(),
                },
                // node_vertices
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.vertex_buffer().as_entire_binding(),
                },
                // node_pool_capacity
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.capacity_buffer().as_entire_binding(),
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
                label: Some("SVO Node Pool Bind Group Layout"),
                entries: &[
                    // node_count
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // node_headers
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // node_payload
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // node_vertices
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                    // node_pool_capacity
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility,
                        count: None,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                ],
            })
    }
}
