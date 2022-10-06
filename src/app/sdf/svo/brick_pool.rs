
use wgpu::util::DeviceExt;

use crate::app::gpu::GPUContext;

use super::Capacity;

/// A format of one voxel in brick pool texture.
/// - It determines how many bits are used for each voxel.
pub enum BrickVoxelFormat {
    Distance, DistanceColor
    // TODO: f16 | f32 | f16f16 | f32f32
}
impl BrickVoxelFormat {
    pub fn voxel_ints(&self) -> u32 {
        match self {
            BrickVoxelFormat::Distance => 1,
            BrickVoxelFormat::DistanceColor => 2,
        }
    }
}

/// A format of brick pool texture.
/// - Used to initialize brick pool texture.
/// - It determines total size of the texture by defining voxel format and padding around each brick.
pub struct BrickPoolFormat {
    /// What is stored in one voxel of the brick in brick pool.
    pub voxel_format: BrickVoxelFormat,
    
    /// amount of space in each direction (0 -> 8, 1 -> 10, 2 -> 12, ...)
    pub padding: u32,
}

impl Default for BrickPoolFormat {
    fn default() -> Self {
        BrickPoolFormat {
            voxel_format: BrickVoxelFormat::Distance,
            padding: 1,
        }
    }
}

impl BrickPoolFormat {
    pub fn voxel_format(&self) -> &BrickVoxelFormat {
        &self.voxel_format
    }
    pub fn padding(&self) -> u32 {
        self.padding
    }
    pub fn ints_per_brick_in_one_dimension(&self) -> u32 {
        2 * self.padding + self.voxel_format.voxel_ints()
    }
    pub fn bytes_per_brick_in_one_dimension(&self) -> u32 {
        (std::mem::size_of::<u32>() as u32) * self.ints_per_brick_in_one_dimension()
    }
}

/// A Brick Pool of the SVO residing on GPU.
pub struct BrickPool {
    
    /// A gpu texture that stores all the bricks.
    brick_atlas: wgpu::Texture,
    
    /// A Texture view for the brick atlas.
    brick_atlas_view: wgpu::TextureView,
    
    /// An amount of bricks that can be stored in this texture in each dimension.
    side_size: u32,
    
    // A uniform buffer holding value for dimensions of brick pool.
    side_size_buffer: wgpu::Buffer,
    
    /// A value into which a count information is read from count_buffer.
    /// - If None, `load_count` method must be call to populate this value.
    count: Option<u32>,
    
    /// In this buffer number of bricks in SVO is stored.
    /// - It is used for atomic increments in shaders
    count_buffer: wgpu::Buffer,
    
    /// A bind group of this particular node pool.
    /// - When accessed through a `bind_group` method it will bew created.
    bind_group: Option<wgpu::BindGroup>,
    
}

// getters
impl BrickPool {
    pub fn brick_atlas(&self) -> &wgpu::Texture {
        &self.brick_atlas
    }
    pub fn side_size(&self) -> &u32 {
        &self.side_size
    }
    pub fn side_size_buffer(&self) -> &wgpu::Buffer {
        &self.side_size_buffer
    }
    pub fn count(&self) -> Option<u32> {
        self.count
    }
    pub fn count_buffer(&self) -> &wgpu::Buffer {
        &self.count_buffer
    }
}

impl BrickPool {
    
    /// Creates empty brick pool texture.
    ///   `capacity` - Used to set minimal amount of bricks that can be stored in this texture.
    ///   `context`  - GPU context.
    #[profiler::function]
    pub fn new(gpu: &GPUContext, capacity: Capacity, format: BrickPoolFormat) -> Self {
        let side_size = Self::dimension_from_capacity(capacity.nodes());
        
        let brick_atlas = gpu.device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width:                 side_size,
                    height:                side_size,
                    depth_or_array_layers: side_size,
                },
                label:           None,
                mip_level_count: 1,
                sample_count:    1,
                dimension:       wgpu::TextureDimension::D3,
                format:          wgpu::TextureFormat::R32Float,
                usage:           wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
            }
        );
        
        let brick_atlas_view = brick_atlas.create_view(&wgpu::TextureViewDescriptor::default());
        
        let side_size_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[side_size]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        
        let count = 0u32;
        let count_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[count]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        Self {
            brick_atlas,
            brick_atlas_view,
            side_size,
            side_size_buffer,
            count: Some(count),
            count_buffer,
            bind_group: None,
        }
    }
    
    /// Calculates minimum number of bricks in one dimension of (cubical) brick pool which can contain given amount of bricks.
    /// `brick_count` - Amount of bricks that need to be stored in brick pool.
    pub fn dimension_from_capacity(brick_count: u32) -> u32 {
        let mut dimension: u32 = 0;
        loop {
            let capacity = dimension.pow(3);
            if capacity >= brick_count {
                break;
            }
            dimension += 1;
        }
        dimension
    }
    
}

// GPU binding
impl BrickPool {
    /// Returns existing bind group or creates a new one with given layout.
    #[profiler::function]
    pub fn bind_group(&mut self, gpu: &GPUContext, layout: &wgpu::BindGroupLayout) -> &wgpu::BindGroup {
        if self.bind_group.is_none() {
            self.bind_group = Some(gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("SVO Node Pool Bind Group"),
                layout: layout,
                entries: &[
                    // brick_atlas
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.brick_atlas_view),
                    },
                    // count_buffer
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.count_buffer().as_entire_binding(),
                    },
                    // side_size_buffer
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.side_size_buffer().as_entire_binding(),
                    },
                ],
            }));
        };
        self.bind_group.as_ref().unwrap()
    }
    
    /// Creates and returns a custom binding for the node pool.
    #[profiler::function]
    pub fn create_write_bind_group_layout(gpu: &GPUContext, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayout {
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SVO Node Pool Bind Group Layout"),
            entries: &[
                // brick_atlas
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    count: None,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rg32Float, // TODO: Use format of given BrickVoxelFormat
                        view_dimension: wgpu::TextureViewDimension::D3,
                    }
                },
                // count_buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                // side_size_buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
