use std::marker::PhantomData;
use winit_input_helper::WinitInputHelper;

use dolly::{
    driver::RigDriver,
    prelude::{Handedness, Position, RightHanded, Smooth, YawPitch},
    rig::RigUpdateParams,
};

use super::Camera;
use crate::framework::math::Transform;

pub struct OrbitCameraRig {
    rig: dolly::rig::CameraRig,
    camera: Camera,
}

impl OrbitCameraRig {
    pub fn from_camera(camera: Camera, target: glam::Vec3) -> Self {
        let mut yaw_pitch = YawPitch::new();
        let current_distance_to_target = glam::Vec3::distance(camera.position, target);
        yaw_pitch.set_rotation_quat(camera.rotation);
        let mut rig = dolly::rig::CameraRig::builder()
            .with(yaw_pitch)
            .with(Smooth::new_rotation(0.8))
            .with(Position::new(camera.position))
            .with(SmoothZoom::new(current_distance_to_target, 0.8))
            .build();

        // rig.driver_mut::<SmoothZoom<RightHanded>>().zoom(distance - current_distance_to_target);
        rig.driver_mut::<Position>().position = target.into();

        Self { rig, camera }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera.fov = camera.fov;
        self.camera.aspect_ratio = camera.aspect_ratio;
        self.camera.near = camera.near;
        self.camera.far = camera.far;
    }

    pub fn on_input(&mut self, input: &WinitInputHelper) {
        let (dx, dy) = input.mouse_diff();
        if (dx != 0.0 || dy != 0.0) && input.mouse_held(0) {
            self.rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(-dx * 0.7, -dy * 0.7);
        }
        let scroll = input.scroll_diff();
        if scroll != 0.0 {
            self.rig
                .driver_mut::<SmoothZoom<RightHanded>>()
                .zoom(-scroll);
        }
    }

    pub fn update(&mut self, delta_time_seconds: f32, _: &WinitInputHelper) -> Transform {
        let res = self.rig.update(delta_time_seconds);
        self.camera.position = res.position.into();
        self.camera.rotation = res.rotation.into();
        Transform {
            position: res.position.into(),
            rotation: res.rotation.into(),
            ..Default::default()
        }
    }
}

/// This is a custom dolly rig driver that behaves similarly to Arm driver but uses just distance which can be smoothly changed.
/// Implementation based on example: https://github.com/h3r2tic/dolly/blob/main/examples/nested_driver.rs
#[derive(Debug)]
pub struct SmoothZoom<H: Handedness> {
    rig: dolly::rig::CameraRig<H>,
}

impl<H: Handedness> SmoothZoom<H> {
    pub fn new(distance: f32, smoothness: f32) -> Self {
        Self {
            rig: dolly::rig::CameraRig::builder()
                .with(Position::new(glam::vec3(0.0, 0.0, distance)))
                .with(Smooth::new_position(smoothness))
                .build(),
        }
    }

    pub fn zoom(&mut self, zoom: f32) {
        let p = self.rig.driver_mut::<Position>();
        let scale = 1.0 - zoom * -0.3;
        p.position.z = (p.position.z * scale).max(0.1);
    }
}

impl<H: Handedness> RigDriver<H> for SmoothZoom<H> {
    fn update(&mut self, params: RigUpdateParams<H>) -> dolly::transform::Transform<H> {
        let t = self.rig.update(params.delta_time_seconds);

        let parent_position: glam::Vec3 = params.parent.position.into();
        let parent_rotation: glam::Quat = params.parent.rotation.into();
        let position: glam::Vec3 = t.position.into();

        let final_position = parent_position + parent_rotation * position;
        dolly::transform::Transform {
            rotation: params.parent.rotation,
            position: final_position.into(),
            phantom: PhantomData,
        }
    }
}
