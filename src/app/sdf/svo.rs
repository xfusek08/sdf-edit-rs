use glam::Vec4;

use crate::app::gpu::GPUContext;


/// sparse voxel octree on GPU
pub struct SVOctree {
    
    /// A total number on nodes for this octree.
    ///   This value should be updated when octree is evaluated (nodes in gpu memory changed).
    node_count: u32,
    
    /// A value indicating how many times this octree has been evaluated (changed).
    ///   This value is used by renderer to help it determine whether it needs to update
    ///   its gpu resources needed for rendering of this octree.
    generation: u32,
    
    /// A total number of nodes that can be stored currently in the buffers.
    node_capacity: u32,
    
    /// Buffer of first 32 bytes of nodes, where each integer contains:
    ///   - 1 bit - is subdivided flag
    ///   - 1 bit - is has brick linked flag
    ///   - 30 bits - child node tile index
    node_header_buffer: wgpu::Buffer,
    
    /// Buffer of second 32 bytes of nodes, where each integer contains:
    ///   ether: (when node has linked brick)
    ///     - 2 bit padding
    ///     - 10x10x10 (x,y,z) of 10 bit indices into brick bool
    ///   or: (if node does not have linked brick)
    ///     - 8x8x8x8 of rgba bytes
    node_payload_buffer: wgpu::Buffer,
    
    /// Buffer of node positions, where each node has its own vec4 (4xf32) vertex value:
    ///   - 3x f32 - position
    ///   - 1x f32 - size
    node_vertex_buffer: wgpu::Buffer,
    
    
    /// A 3D texture atlas on GPU holding an NxNxN array of SDF bricks with padding.
    brick_pool_texture: BrickPoolTexture,
    
    
}

// getters
impl SVOctree {
    pub fn node_count(&self) -> u32 {
        self.node_count
    }
    pub fn node_capacity(&self) -> u32 {
        self.node_capacity
    }
    pub fn node_payload_buffer(&self) -> &wgpu::Buffer {
        &self.node_payload_buffer
    }
    pub fn node_header_buffer(&self) -> &wgpu::Buffer {
        &self.node_header_buffer
    }
    pub fn node_vertex_buffer(&self) -> &wgpu::Buffer {
        &self.node_vertex_buffer
    }
    pub fn brick_pool_texture(&self) -> &BrickPoolTexture {
        &self.brick_pool_texture
    }
}

impl SVOctree {
    
    /// Updates the node count ang increments the generation counter.
    ///   Use this method after evaluation finishes to update the node count.
    pub fn update(&mut self, node_count: u32) {
        self.node_count = node_count;
        self.generation += 1;
    }
    
}

impl SVOctree {
    
    /// Creates empty GPU octree by allocating buffers on GPU.
    pub fn new(capacity: SVOctreeCapacity, context: &GPUContext) -> SVOctree {
        let node_count = 0;
        let generation = 0;
        let node_capacity = capacity.nodes();
        
        let node_header_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let node_payload_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let node_vertex_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Vec4>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let brick_pool_texture = BrickPoolTexture::new(
            BrickPoolFormat::default(),
            capacity,
            context
        );
        
        SVOctree {
            node_count,
            generation,
            node_capacity,
            node_header_buffer,
            node_payload_buffer,
            node_vertex_buffer,
            brick_pool_texture,
        }
    }
    
    /// This function set this octree to contain exactly one root node. with a sphere SDF written into it.
    pub fn init_root_sphere(&mut self, context: &GPUContext) {
        
    }
    
}


// Brick Pool
// ----------


pub enum BrickVoxelFormat {
    Distance, DistanceColor
}
impl BrickVoxelFormat {
    pub fn voxel_ints(&self) -> u32 {
        match self {
            BrickVoxelFormat::Distance => 1,
            BrickVoxelFormat::DistanceColor => 2,
        }
    }
}

pub struct BrickPoolFormat {
    /// What is stored in one voxel of the brick in brick pool.
    voxel_format: BrickVoxelFormat,
    
    /// amount of space in each direction (0 -> 8, 1 -> 10, 2 -> 12, ...)
    padding: u32,
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

impl Default for BrickPoolFormat {
    fn default() -> Self {
        BrickPoolFormat {
            voxel_format: BrickVoxelFormat::Distance,
            padding: 1,
        }
    }
}

pub struct BrickPoolTexture {
    
    /// An amount of bricks that can be stored in this texture in each dimension.
    dimensions: glam::UVec3,
    
    /// A gpu texture that stores all the bricks.
    atlas: wgpu::Texture,
}

impl BrickPoolTexture {
    
    /// Creates empty brick pool texture.
    ///   `capacity` - Used to set minimal amount of bricks that can be stored in this texture.
    ///   `context`  - GPU context.
    pub fn new(format: BrickPoolFormat, capacity: SVOctreeCapacity, context: &GPUContext) -> BrickPoolTexture {
        let dimension = BrickPoolTexture::dimension_from_capacity(capacity.nodes());
        
        let atlas = context.device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width:                 dimension,
                    height:                dimension,
                    depth_or_array_layers: dimension,
                },
                label:           None,
                mip_level_count: 1,
                sample_count:    1,
                dimension:       wgpu::TextureDimension::D3,
                format:          wgpu::TextureFormat::R32Float,
                usage:           wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
            }
        );
        
        BrickPoolTexture {
            dimensions: (dimension, dimension, dimension).into(),
            atlas,
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


// SVOctreeCapacity
// ----------------


/// A helper struct to express desired octree capacity
pub enum SVOctreeCapacity {
    /// Reserve capacity for the exact number of nodes in the octree.
    Nodes(u32),
    
    /// Reserve capacity for fully subdivided octree up to the given depth.
    Depth(u32),
}

impl SVOctreeCapacity {
    
    /// Returns the number of nodes that should be reserved for the octree.
    pub fn nodes(&self) -> u32 {
        match self {
            Self::Nodes(n) => *n,
            
            // Calculate the number of nodes in a fully subdivided octree of the given depth.
            // The intuitive formula:
            //     1 + 8 + 8^2 + ... + 8^depth
            // Can be expressed as:
            //     (8^(depth + 1) - 1) / 7
            //     see: https://www.wolframalpha.com/input?i2d=true&i=f%5C%2840%29d%5C%2841%29%3DSum%5BPower%5B8%2C%5C%2840%29d-n%5C%2841%29%5D%2C%7Bn%2C0%2Cd%7D%5D
            // Also power of 8 can be expressed by bit shifting:
            //     8^n = 1 << (3 * n)
            Self::Depth(d) => (1 << (3 * (d + 1))) / 7,
        }
    }
    
}