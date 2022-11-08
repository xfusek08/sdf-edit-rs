
use super::BoundingCube;

#[derive(Debug)]
pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    
    pub fn new(min: glam::Vec3, max: glam::Vec3) -> Self {
        Self { min, max }
    }
    
    pub fn from_bounding_cube(bounding_cube: &BoundingCube) -> Self {
        let half_size = bounding_cube.size * 0.5;
        let min = bounding_cube.pos - glam::Vec3::splat(half_size);
        let max = bounding_cube.pos + glam::Vec3::splat(half_size);
        Self { min, max }
    }
    
    pub fn bounding_cube(&self) -> BoundingCube {
        let size = (self.max - self.min).max_element();
        let pos = self.min + (self.max - self.min) * 0.5;
        BoundingCube { pos, size }
    }
    
}
