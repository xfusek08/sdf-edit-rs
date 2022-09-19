use glam::{Vec3, Mat4};
use dolly::{
    rig::CameraRig,
    prelude::{Arm, YawPitch, Smooth, LookAt}
};

pub struct CameraProperties {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Default for CameraProperties {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            fov: 90.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

pub struct Camera {
    pub rig: dolly::rig::CameraRig,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

// builder
impl Camera {
    
    pub fn new(properties: CameraProperties) -> Self {
        Self {
            rig: CameraRig::builder().build(),
            aspect_ratio: properties.aspect_ratio,
            fov: properties.fov,
            near: properties.near,
            far: properties.far,
        }
    }
    
    pub fn orbit(mut self, center: Vec3, distance: f32) -> Self {
        self.rig = CameraRig::builder()
            .with(YawPitch::new())
            .with(Arm::new((Vec3::Z * distance) - center))
            .with(Smooth::new_rotation(1.1))
            .with(Smooth::new_position(1.1))
            .with(LookAt::new(center))
            .build();
        self
    }
    
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        glam::Mat4::from_rotation_translation(
            self.rig.final_transform.rotation,
            self.rig.final_transform.position
        ).inverse()
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near,
            self.far
        )
    }
    
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
