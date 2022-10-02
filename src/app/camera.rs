use std::marker::PhantomData;

use glam::{Vec3, Mat4};
use dolly::{
    prelude::{YawPitch, Smooth, Handedness, Position},
    rig::{CameraRig, RigUpdateParams},
    driver::RigDriver,
    transform::Transform as DollyTransform,
};
use super::transform::Transform;

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
            .with(Smooth::new_rotation(0.8))
            // .with(Smooth::new_rotation(0.3).predictive(true))
            .with(SmoothZoomArm::new((Vec3::Z * distance) - center, 0.8))
            .build();
        self
    }
    
}

impl Camera {
    
    #[profiler::function]
    pub fn view_matrix(&self) -> Mat4 {
        glam::Mat4::from_rotation_translation(
            self.rig.final_transform.rotation,
            self.rig.final_transform.position
        ).inverse()
    }
    
    #[profiler::function]
    pub fn projection_matrix(&self) -> Mat4 {
        glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect_ratio,
            self.near,
            self.far
        )
    }
    
    #[profiler::function]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    pub fn transform(&self) -> Transform {
        Transform {
            position: self.rig.final_transform.position,
            rotation: self.rig.final_transform.rotation,
            ..Default::default()
        }
    }
}

/// This is a custom dolly rig driver that behaves just like Arm but smooths a offset vale
/// Implementation based on example: https://github.com/h3r2tic/dolly/blob/main/examples/nested_driver.rs
/// Offsets the camera along a vector, in the coordinate space of the parent.
#[derive(Debug)]
pub struct SmoothZoomArm<H: Handedness> {
    direction: Vec3,
    smooth_rig: CameraRig<H>,
}

impl<H: Handedness> SmoothZoomArm<H> {
    pub fn new(offset: Vec3, smoothness: f32) -> Self {
        let magnitude = offset.length();
        Self {
            direction: offset.normalize(),
            smooth_rig: CameraRig::builder()
                .with(Position::new(Vec3::new(magnitude, 0.0, 0.0)))
                .with(Smooth::new_position(smoothness))
                .build(),
        }
    }
    
    pub fn scale_distance(&mut self, scale: f32) {
        let p = self.smooth_rig.driver_mut::<Position>();
        p.position.x = p.position.x * scale;
    }
}

impl<H: Handedness> RigDriver<H> for SmoothZoomArm<H> {
    fn update(&mut self, params: RigUpdateParams<H>) -> DollyTransform<H> {
        let t = self.smooth_rig.update(params.delta_time_seconds);
        DollyTransform {
            rotation: params.parent.rotation,
            position: params.parent.position + params.parent.rotation * (t.position.x * self.direction),
            phantom: PhantomData,
        }
    }
}
