
use crate::{
    sdf::geometry::Geometry,
    framework::{
        gui::GuiModule,
        camera::Camera
    },
    demo_app::{
        svo_sdf_brick::DisplayOptions,
        scene::Scene,
        components::{
            AxisMesh,
            Active
        },
    },
};


pub struct LegacyAppsGui;

impl GuiModule<Scene> for LegacyAppsGui {
    fn gui_window(&mut self, _: &mut Scene, _: &egui::Context) {}
    
    fn gui_section(&mut self, scene: &mut Scene, ui: &mut egui::Ui) {
        egui::Grid::new("grid_1")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("gui updates:");
                ui.label(format!("{}", scene.counters.gui_updates));
                ui.end_row();
                ui.label("renders:");
                ui.label(format!("{}", scene.counters.renders));
                ui.end_row();
                ui.label("Min voxel Size:");
                ui.add(egui::Slider::new(&mut scene.tmp_evaluator_config.min_voxel_size, Geometry::VOXEL_SIZE_RANGE)
                    .step_by(0.001)
                    .clamp_to_range(true)
                );
            });
        
        ui.separator();
        
        egui::Grid::new("grid_2")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Camera fov:");
                let mut fov = scene.camera_rig.camera().fov;
                ui.add(egui::Slider::new(&mut fov, 10.0..=150.0).clamp_to_range(true));
                scene.camera_rig.set_camera(Camera {
                    fov,
                    ..*scene.camera_rig.camera()
                });
                ui.end_row();
                
                ui.label("Brick Level Break Size (/10):");
                let mut break_size = scene.brick_level_break_size * 10.0;
                ui.add(egui::Slider::new(&mut break_size, 0.01..=3.0)
                    .step_by(0.01)
                    .clamp_to_range(true)
                );
                scene.brick_level_break_size = break_size / 10.0;
                ui.end_row();
                
                ui.label("Hit Distance:");
                ui.add(egui::Slider::new(&mut scene.hit_distance, 0.00001..=0.1)
                    .step_by(0.00001)
                    .clamp_to_range(true)
                );
                ui.end_row();
                
                ui.label("Max Step Count:");
                ui.add(egui::Slider::new(&mut scene.max_step_count, 3..=300).clamp_to_range(true));
        });
        
        ui.separator();
        
        ui.label("TMP SVO Stats");
        for (geometry_id, geometry) in scene.geometry_pool.iter() {
            let id = format!("{:?}", geometry_id);
            egui::CollapsingHeader::new(format!("Geometry: {}", id)).show(ui, |ui| {
                if let Some(svo) = geometry.svo.as_ref() {
                    egui::Grid::new(&id)
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label(format!("Levels:"));
                            ui.label(format!("{}", svo.levels.len()));
                            ui.end_row();
                            ui.label(format!("Node Count:"));
                            ui.label(format!("{}", svo.node_pool.count().unwrap_or(0)));
                            ui.end_row();
                            ui.label(format!("Level Count:"));
                            ui.label(format!("{}", svo.levels.len()));
                            ui.end_row();
                            ui.label(format!("Capacity:"));
                            ui.label(format!("{}", svo.node_pool.capacity()));
                            ui.end_row();
                        });
                    
                    egui::CollapsingHeader::new("Levels").show(ui, |ui| {
                        egui::Grid::new(&id)
                            .num_columns(4)
                            .show(ui, |ui| {
                                ui.label(format!("Level"));
                                ui.label(format!("Start"));
                                ui.label(format!("Count"));
                                ui.label(format!("Total count"));
                                ui.end_row();
                                for (level_index, level) in svo.levels.iter().enumerate() {
                                    ui.label(format!("{}", level_index));
                                    ui.label(format!("{}", level.start_index));
                                    ui.label(format!("{}", level.node_count));
                                    ui.label(format!("{}", level.start_index + level.node_count));
                                    ui.end_row();
                                }
                            });
                    });
                }
            });
        }
        
        egui::CollapsingHeader::new("Display Toggles").show(ui, |ui| {
            // disable axes rendering
            let mut show_axes = scene.display_toggles.show_axes;
            ui.checkbox(&mut show_axes, "Show Axes");
            if show_axes != scene.display_toggles.show_axes {
                for (_, (_, active)) in scene.world.query::<(&AxisMesh, &mut Active)>().iter() {
                    *active = Active(show_axes);
                }
                scene.display_toggles.show_axes = show_axes;
            }
            
            // ui.checkbox(&mut scene.display_toggles.show_voxel_size_reference, "Show Voxel Size Reference");
            ui.checkbox(&mut scene.display_toggles.show_wireframe, "Show Wireframe");
            
            // Macro implementing checkbox for given bit flag, for flag it creates local mutable bool variable
            macro_rules! checkbox {
                ($flag:expr, $name:expr) => {
                    let mut checked = scene.display_toggles.brick_display_options.contains($flag);
                    ui.checkbox(&mut checked, $name);
                    if checked {
                        scene.display_toggles.brick_display_options.insert($flag);
                    } else {
                        scene.display_toggles.brick_display_options.remove($flag);
                    }
                };
            }
            
            checkbox!(DisplayOptions::DEPTH,      "Depth");
            checkbox!(DisplayOptions::NORMALS,    "Normals");
            checkbox!(DisplayOptions::SOLID,      "Solid");
            checkbox!(DisplayOptions::STEP_COUNT, "Step Count");
        });
    }
}
