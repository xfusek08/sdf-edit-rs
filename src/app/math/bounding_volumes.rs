
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundingCube {
    pub pos: glam::Vec3,
    pub size: f32,
}

pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    pub fn bounding_cube(&self) -> BoundingCube {
        let size = (self.max - self.min).max_element();
        let pos = self.min + (self.max - self.min) * 0.5;
        BoundingCube { pos, size }
    }
}
