
use crate::{
    demo_app::scene::Scene,
    framework::{
        gui::GuiModule,
        camera::{
            OrbitCameraRig,
            FreeCameraRig, CameraRig
        },
    },
};

pub struct CameraGuiModule;

impl CameraGuiModule {
    
    fn create_free_camera_rig(&self, scene: &Scene) -> CameraRig {
        CameraRig::Free(FreeCameraRig::from_camera(
            scene.camera_rig.camera().clone(),
            0.2,
            1.0,
        ))
    }
    
    fn create_orbit_camera_rig(&self, scene: &Scene) -> CameraRig {
        CameraRig::Orbit(OrbitCameraRig::from_camera(
            scene.camera_rig.camera().clone(),
            glam::Vec3::ZERO,
            10.0,
        ))
    }
}

impl GuiModule<Scene> for CameraGuiModule {
    fn gui_window(&mut self, _: &mut Scene, _: &egui::Context) {}

    fn gui_section(&mut self, scene: &mut Scene, ui: &mut egui::Ui) {
        let mut is_free = match scene.camera_rig {
            CameraRig::Free(_) => true,
            _ => false,
        };
        let was_free = is_free;
        
        ui.horizontal(|ui| {
            ui.label("Camera");
            ui.checkbox(&mut is_free, "Free");
        });
        
        match &mut scene.camera_rig {
            CameraRig::Free(rig) => {
                ui.horizontal(|ui| {
                    ui.label("Look Speed");
                    ui.add(egui::Slider::new(&mut rig.look_speed, 0.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Move Speed");
                    ui.add(egui::Slider::new(&mut rig.move_speed, 0.0..=5.0));
                });
            },
            _ => {},
        }
        
        if was_free != is_free {
            scene.camera_rig = if is_free {
                self.create_free_camera_rig(scene)
            } else {
                self.create_orbit_camera_rig(scene)
            }
        }
    }
}
