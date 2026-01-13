#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundingCube {
    pub pos: glam::Vec3,
    pub size: f32,
}

impl BoundingCube {
    pub const UNIT: Self = Self {
        pos: glam::Vec3::ZERO,
        size: 1.0,
    };
}

impl Default for BoundingCube {
    fn default() -> Self {
        Self::UNIT
    }
}
