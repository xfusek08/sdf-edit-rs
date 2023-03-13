
use crate::{
    demo_app::scene::Scene,
    framework::{
        gui::GuiModule,
        camera::{
            CameraRig,
            OrbitCameraRig,
            FreeCameraRig
        },
    },
};

pub struct CameraGuiModule {
    is_free: bool,
    speed:  f32,
}

impl CameraGuiModule {
    pub fn new(speed: f32) -> Self {
        Self {
            is_free: false,
            speed,
        }
    }
    
    fn create_free_camera_rig(&self, scene: &Scene) -> Box<dyn CameraRig> {
        Box::new(FreeCameraRig::from_camera(
            scene.camera_rig.camera().clone(),
            self.speed
        ))
    }
    
    fn create_orbit_camera_rig(&self, scene: &Scene) -> Box<dyn CameraRig> {
        Box::new(OrbitCameraRig::from_camera(
            scene.camera_rig.camera().clone(),
            glam::Vec3::ZERO,
            10.0,
        ))
    }
}

impl GuiModule<Scene> for CameraGuiModule {
    fn gui_window(&mut self, _: &mut Scene, _: &egui::Context) {}

    fn gui_section(&mut self, scene: &mut Scene, ui: &mut egui::Ui) {
        let was_free = self.is_free;
        ui.horizontal(|ui| {
            ui.label("Camera");
            ui.checkbox(&mut self.is_free, "Free");
        });
        if was_free != self.is_free {
            scene.camera_rig = if self.is_free {
                self.create_free_camera_rig(scene)
            } else {
                self.create_orbit_camera_rig(scene)
            }
        }
    }
}
