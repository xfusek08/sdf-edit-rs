
use winit_input_helper::WinitInputHelper;

use super::math::Transform;

pub trait CameraRig {
    fn camera(&self) -> &Camera;
    fn set_camera(&mut self, camera: Camera);
    fn on_input(&mut self, input: &WinitInputHelper);
    fn update(&mut self, delta_time_seconds: f32, input: &WinitInputHelper) -> Transform;
}

pub trait SceneWithCamera {
    fn get_camera_rig(&self) -> &dyn CameraRig;
    fn get_camera_mut(&mut self) -> &mut dyn CameraRig;
}


mod camera_updater;
pub use camera_updater::*;

mod camera;
pub use camera::*;

mod orbit_camera_rig;
pub use orbit_camera_rig::*;

mod free_camera_rig;
pub use free_camera_rig::*;
