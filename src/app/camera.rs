use bevy_ecs::prelude::Component;
use dolly::{rig::CameraRig, prelude::{Arm, YawPitch, Smooth, Position}};
use glam::{Vec3, Mat4};


#[derive(Component)]
pub struct Camera {
    pub rig: dolly::rig::CameraRig,
    // up: Vec3,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

// builder
impl Camera {
    pub fn new() -> Self {
        Self {
            rig: CameraRig::builder().build(),
            // up: Vec3::Y,
            aspect_ratio: 1.0,
            fov: 45.0,
            near: 0.1,
            far: 100.0
        }
    }
    
    pub fn orbit(mut self, center: Vec3, distance: f32) -> Self {
        self.rig = CameraRig::builder()
            .with(Position::new(center))
            .with(YawPitch::new())
            .with(Smooth::new_rotation(1.5))
            .with(Arm::new(Vec3::NEG_Z * distance))
            .build();
        self
    }
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        glam::Mat4::from_rotation_translation(
            self.rig.final_transform.rotation,
            self.rig.final_transform.position
        )
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        glam::Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }
    
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
