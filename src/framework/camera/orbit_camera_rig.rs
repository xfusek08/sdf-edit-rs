
use std::marker::PhantomData;
use winit_input_helper::WinitInputHelper;

use dolly::{
    driver::RigDriver,
    rig::RigUpdateParams,
    prelude::{
        YawPitch,
        Smooth,
        RightHanded,
        Handedness,
        Position
    },
};

use crate::framework::math::Transform;
use super::{Camera, CameraRig};

pub struct OrbitCameraRig {
    rig:    dolly::rig::CameraRig,
    camera: Camera,
}

impl OrbitCameraRig {
    
    pub fn from_camera(camera: Camera, target: glam::Vec3, distance: f32) -> Self {
        let mut yaw_pitch = YawPitch::new();
        yaw_pitch.set_rotation_quat(camera.rotation);
        let rig = dolly::rig::CameraRig::builder()
            .with(yaw_pitch)
            .with(Smooth::new_rotation(0.8))
            .with(Position::new(target))
            .with(SmoothZoom::new(distance, 0.8))
            .build();
        Self { rig, camera }
    }
}

impl CameraRig for OrbitCameraRig {
    fn camera(&self) -> &Camera {
        &self.camera
    }
    
    fn set_camera(&mut self, camera: Camera) {
        self.camera.fov = camera.fov;
        self.camera.aspect_ratio = camera.aspect_ratio;
        self.camera.near = camera.near;
        self.camera.far = camera.far;
    }

    fn on_input(&mut self, input: &WinitInputHelper) {
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
    
    fn update(&mut self, delta_time_seconds: f32, _: &WinitInputHelper) -> Transform {
        let res = self.rig.update(delta_time_seconds);
        self.camera.position = res.position;
        self.camera.rotation = res.rotation;
        Transform {
            position: res.position,
            rotation: res.rotation,
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
                .with(Position::new((0.0, 0.0, distance).into()))
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
        dolly::transform::Transform {
            rotation: params.parent.rotation,
            position: params.parent.position + params.parent.rotation * t.position,
            phantom: PhantomData,
        }
    }
}
