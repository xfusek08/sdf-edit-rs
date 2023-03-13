
use winit_input_helper::WinitInputHelper;

use dolly::{
    prelude::{
        YawPitch,
        Smooth,
        Position
    },
};

use crate::framework::math::Transform;
use super::{Camera, CameraRig};

pub struct FreeCameraRig {
    rig:    dolly::rig::CameraRig,
    camera: Camera,
    speed:  f32,
}

impl FreeCameraRig {
    pub fn from_camera(camera: Camera, speed: f32) -> Self {
        let mut yaw_pitch = YawPitch::new();
        yaw_pitch.set_rotation_quat(camera.rotation);
        let rig = dolly::rig::CameraRig::builder()
            .with(yaw_pitch)
            .with(Smooth::new_rotation(1.0))
            .with(Position::new(camera.position))
            .with(Smooth::new_position(1.0))
            .build();
        Self { rig, camera, speed }
    }
}

impl CameraRig for FreeCameraRig {
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
                .rotate_yaw_pitch(-dx * self.speed, -dy * self.speed);
        }
    }
    
    fn update(&mut self, delta_time_seconds: f32, input: &WinitInputHelper) -> Transform {
        let forward = input.key_held(winit::event::VirtualKeyCode::W);
        let backward = input.key_held(winit::event::VirtualKeyCode::S);
        let left = input.key_held(winit::event::VirtualKeyCode::A);
        let right = input.key_held(winit::event::VirtualKeyCode::D);
        let up = input.key_held(winit::event::VirtualKeyCode::E);
        let down = input.key_held(winit::event::VirtualKeyCode::Q);
        
        let mut move_vector = glam::Vec3::ZERO;
        
        if forward != backward {
            let dir = self.rig.final_transform.forward();
            move_vector += if forward { dir } else { -dir };
        }
        if left != right {
            let dir = self.rig.final_transform.right();
            move_vector += if left { -dir } else { dir };
        }
        if up != down {
            let dir = self.rig.final_transform.up();
            move_vector += if up { -dir } else { dir };
        }
        
        if move_vector != glam::Vec3::ZERO {
            self.rig.driver_mut::<Position>().position += move_vector * self.speed * 2.0;
        }
        
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
