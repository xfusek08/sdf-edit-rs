use glam::Vec4;

use crate::app::gpu::GPUContext;


// Sparse Voxel Octree
// -------------------

pub struct SVOctree {
    
    /// A header value of svo root node.
    /// bit 0 "is subdivided flag":
    ///   - 0: There are no children to this node. SVO does not need the node pool, thus it might not be even allocated.
    ///   - 1: There is a first tile in node pool holding first level of octree
    /// bit 1 "has brick flag":
    ///   - 0: There is no brick for this node, payload carries a solid color.
    ///        If root is not subdivided (see bit 0), SVO does not need the brick pool, thus it might not be even allocated.
    ///   - 1: There is a brick in this node, payload carries a brick index.
    ///        NOTE: if root is not subdivided (see bit 0), Brick pool can be shirked to 1 brick to save memory.
    pub root_header: u32,
    
    /// A payload value of svo root node.
    /// If root has a brick (see bit 1), this is the coordinates of the brick in the brick pool.
    ///     In format: 2b padding | 10b x | 10b y | 10b z
    /// If root does not have a brick, this is the color of the node.
    ///     In format: 8b r | 8b g | 8b b | 8b a
    pub root_payload: u32,
    
    /// A node pool of the SVO on GPU.
    pub node_pool: Option<SVONodePool>,
    
    /// A 3D texture atlas on GPU holding an NxNxN array of SDF bricks with padding.
    pub brick_pool: Option<BrickPool>,
    
}

impl SVOctree {
    /// A color of default SVO root node (solid color of 1m x 1m x 1m filled cube).
    ///   see: https://coolors.co/ff9933
    const DEFAULT_COLOR: image::Rgba<u8> = image::Rgba([255, 153, 51, 255]);
}

impl Default for SVOctree {
    
    /// A solid color 1x1x1 filled cube
    fn default() -> Self {
        
        // static cast from color RGBA8 to u32 type
        let color_u32 = bytemuck::from_bytes::<u32>(&SVOctree::DEFAULT_COLOR.0).clone();
        
        Self {
            root_header: 0, // no subdivision, no brick
            root_payload: color_u32,
            node_pool: None,
            brick_pool: None,
        }
    }
}


// SVO Node Pool Capacity
// ----------------------


/// A helper struct to express desired octree capacity
pub enum SVONodePoolCapacity {
    /// Reserve capacity for the exact number of nodes in the octree.
    Nodes(u32),
    
    /// Reserve capacity for fully subdivided octree up to the given depth.
    Depth(u32),
}

impl SVONodePoolCapacity {
    
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


// SVO Noe Pool
// ------------


pub struct SVONodePool {
    /// A total number on nodes for this octree.
    ///   This value should be updated when octree is evaluated (nodes in gpu memory changed).
    count: u32,
    
    /// A total number of nodes that can be stored currently in the buffers.
    capacity: u32,
    
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
    
}

// getters
impl SVONodePool {
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
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
}

impl SVONodePool {
    /// Creates empty GPU octree by allocating buffers on GPU.
    pub fn new(capacity: SVONodePoolCapacity, context: &GPUContext) -> Self {
        let count = 0;
        
        let capacity = capacity.nodes();
        let capacity64 = capacity as u64;
        
        let header_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: capacity64 * std::mem::size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let payload_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: capacity64 * std::mem::size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        let vertex_buffer = context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: capacity64 * std::mem::size_of::<Vec4>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        
        Self {
            count,
            capacity,
            header_buffer,
            payload_buffer,
            vertex_buffer,
        }
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

pub struct BrickPool {
    
    /// An amount of bricks that can be stored in this texture in each dimension.
    dimensions: glam::UVec3,
    
    /// A gpu texture that stores all the bricks.
    atlas: wgpu::Texture,
}

impl BrickPool {
    
    /// Creates empty brick pool texture.
    ///   `capacity` - Used to set minimal amount of bricks that can be stored in this texture.
    ///   `context`  - GPU context.
    pub fn new(format: BrickPoolFormat, capacity: SVONodePoolCapacity, context: &GPUContext) -> Self {
        let dimension = Self::dimension_from_capacity(capacity.nodes());
        
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
        
        Self {
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
