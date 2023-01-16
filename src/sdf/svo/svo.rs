
use crate::framework::{gpu, math::BoundingCube};
use super::{NodePool, BrickPool, BrickPoolFormat, BrickVoxelFormat};

/// A helper struct to express desired octree capacity
#[derive(Debug, Clone)]
pub enum Capacity {
    /// Reserve capacity for the exact number of nodes in the octree.
    Nodes(u32),
    
    /// Reserve capacity for fully subdivided octree up to the given depth.
    Depth(u32),
    
    /// Reserve capacity based on brick pool size
    BrickPoolSide(u32),
}

impl Capacity {
    
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
            
            Self::BrickPoolSide(size) => size * size * size
        }
    }
    
}

#[derive(Clone, Debug, Copy)]
pub struct Level {
    pub start_index: u32,
    pub node_count: u32,
}

/// A Sparse Voxel Octree residing on GPU.
#[derive(Debug)]
pub struct Svo {
    
    /// A node pool of the SVO on GPU.
    pub node_pool: NodePool,
    
    /// A 3D texture atlas on GPU holding an NxNxN array of SDF bricks with padding.
    pub brick_pool: BrickPool,
    
    /// A list of SVO levels.
    pub levels: Vec<Level>,
    
    // A bounding cube of the SVO.
    pub domain: BoundingCube,
}

impl Svo {
    #[profiler::function]
    pub fn new(gpu: &gpu::Context, initial_capacity: Capacity) -> Self {
        Self {
            node_pool: NodePool::new(gpu, initial_capacity.clone()),
            brick_pool: BrickPool::new(
                gpu,
                initial_capacity.clone(),
                BrickPoolFormat {
                    voxel_format: BrickVoxelFormat::Distance,
                    padding: 1,
                }
            ),
            domain: BoundingCube::UNIT,
            levels: vec![]
        }
    }
    
    // #[profiler::function]
    // pub fn trim_overflowing_levels(&mut self, gpu: &gpu::Context) {
    //     let mut final_node_count = 0;
    //     for level in self.levels.iter().rev() {
    //         let level_node_count = level.start_index + level.node_count;
    //         if level_node_count >= self.node_pool.capacity() {
                
    //         } else {
    //             final_node_count = level_node_count;
    //             break;
    //         }
    //     }
    // }
    
}
