
use winit_input_helper::WinitInputHelper;

use super::math::Transform;

mod camera_updater;
pub use camera_updater::*;

mod camera;
pub use camera::*;


mod orbit_camera_rig;
pub use orbit_camera_rig::*;

mod free_camera_rig;
pub use free_camera_rig::*;

pub enum CameraRig {
    Orbit(OrbitCameraRig),
    Free(FreeCameraRig),
}

impl CameraRig {
    pub fn camera(&self) -> &Camera {
        match self {
            CameraRig::Orbit(rig) => rig.camera(),
            CameraRig::Free(rig) => rig.camera(),
        }
    }
    
    pub fn set_camera(&mut self, camera: Camera) {
        match self {
            CameraRig::Orbit(rig) => rig.set_camera(camera),
            CameraRig::Free(rig) => rig.set_camera(camera),
        }
    }
    
    pub fn on_input(&mut self, input: &WinitInputHelper) {
        match self {
            CameraRig::Orbit(rig) => rig.on_input(input),
            CameraRig::Free(rig) => rig.on_input(input),
        }
    }
    
    pub fn update(&mut self, delta_time_seconds: f32, input: &WinitInputHelper) -> Transform {
        match self {
            CameraRig::Orbit(rig) => rig.update(delta_time_seconds, input),
            CameraRig::Free(rig) => rig.update(delta_time_seconds, input),
        }
    }
}

pub trait SceneWithCamera {
    fn get_camera_rig(&self) -> &CameraRig;
    fn get_camera_mut(&mut self) -> &mut CameraRig;
}
