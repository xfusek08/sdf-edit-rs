
use crate::framework::math::Transform;

#[derive(Debug, Clone)]
pub struct Camera {
    pub aspect_ratio: f32,
    pub fov:          f32,
    pub near:         f32,
    pub far:          f32,
    pub position:     glam::Vec3,
    pub rotation:     glam::Quat,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            fov:          90.0,
            near:         0.1,
            far:          100.0,
            position:     glam::Vec3::ZERO,
            rotation:     glam::Quat::IDENTITY,
        }
    }
}

impl Camera {
    
    pub fn look_at(mut self, target: glam::Vec3) -> Self {
        let look_at_matrix = glam::Mat4::look_at_rh(self.position, target, glam::Vec3::Y);
        self.rotation = glam::Quat::from_mat4(&look_at_matrix);
        self
    }
    
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }
    
    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near,
            self.far
        )
    }
    
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    pub fn transform(&self) -> Transform {
        Transform {
            position: self.position,
            rotation: self.rotation,
            ..Default::default()
        }
    }
    
    pub fn focal_length(&self) -> f32 {
        1.0 / (self.fov.to_radians() * 0.5).tan()
    }
}
