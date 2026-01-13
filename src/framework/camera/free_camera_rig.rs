use winit_input_helper::WinitInputHelper;

use dolly::prelude::{Position, Smooth, YawPitch};

use crate::framework::math::Transform;

use super::Camera;

pub struct FreeCameraRig {
    rig: dolly::rig::CameraRig,
    camera: Camera,
    pub look_speed: f32,
    pub move_speed: f32,
}

impl FreeCameraRig {
    pub fn from_camera(camera: Camera, look_speed: f32, move_speed: f32) -> Self {
        let mut yaw_pitch = YawPitch::new();
        yaw_pitch.set_rotation_quat(camera.rotation);
        let rig = dolly::rig::CameraRig::builder()
            .with(yaw_pitch)
            .with(Smooth::new_rotation(0.8))
            .with(Position::new(camera.position))
            .with(Smooth::new_position(0.5))
            .build();

        Self {
            rig,
            camera,
            look_speed,
            move_speed,
        }
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
                .rotate_yaw_pitch(-dx * self.look_speed, -dy * self.look_speed);
        }
    }

    pub fn update(&mut self, delta_time_seconds: f32, input: &WinitInputHelper) -> Transform {
        let forward = input.key_held(winit::event::VirtualKeyCode::W);
        let backward = input.key_held(winit::event::VirtualKeyCode::S);
        let left = input.key_held(winit::event::VirtualKeyCode::A);
        let right = input.key_held(winit::event::VirtualKeyCode::D);
        let up = input.key_held(winit::event::VirtualKeyCode::Space);
        let down = input.key_held(winit::event::VirtualKeyCode::LControl);

        let mut move_vector = glam::Vec3::ZERO;

        if forward != backward {
            let dir: glam::Vec3 = self.rig.final_transform.forward();
            move_vector += if forward { dir } else { -dir };
        }
        if left != right {
            let dir: glam::Vec3 = self.rig.final_transform.right();
            move_vector += if left { -dir } else { dir };
        }
        if up != down {
            let dir = glam::Vec3::Y;
            move_vector += if up { dir } else { -dir };
        }

        if move_vector != glam::Vec3::ZERO {
            self.rig
                .driver_mut::<Position>()
                .translate(move_vector * self.move_speed);
        }

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
