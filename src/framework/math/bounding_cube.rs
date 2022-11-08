
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundingCube {
    pub pos: glam::Vec3,
    pub size: f32,
}
