use crate::{framework::gui::GuiModule, demo_app::{scene::Scene, modules::svo_sdf_brick}, sdf::geometry::Geometry};


pub struct LegacyAppsGui;

impl GuiModule<Scene> for LegacyAppsGui {
    fn gui(&mut self, scene: &mut Scene, egui_ctx: &egui::Context) {
        egui::Window::new("App").show(egui_ctx, |ui| {
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
                        ui.add(egui::Slider::new(&mut scene.camera.fov, 10.0..=150.0)
                            // .step_by(step as f64)
                            .clamp_to_range(true)
                        );
                        ui.end_row();
                        ui.label("Brick Level Break Size:");
                        ui.add(egui::Slider::new(&mut scene.brick_level_break_size, 0.0..=5.0)
                            .step_by(0.001)
                            .clamp_to_range(true)
                        );
                });
                
                ui.separator();
                
                ui.label("TMP SVO Stats");
                for (geometry_id, geometry) in scene.geometry_pool.iter() {
                    let id = format!("{:?}", geometry_id);
                    ui.label(format!("Geometry: {}", id));
                    ui.label("SVO:");
                    if let Some(svo) = geometry.svo.as_ref() {
                        egui::Grid::new(&id)
                            .num_columns(2)
                            .show(ui, |ui| {
                                let levels = svo.levels.len() as u32;
                                ui.label(format!("/ {}", levels - 1));
                                ui.end_row();
                                ui.label("Svo Node Count:");
                                ui.label(format!("{:?}", svo.node_pool.count()));
                                ui.end_row();
                                ui.label("Svo Level Count:");
                                ui.label(format!("{}", svo.levels.len()));
                                ui.end_row();
                                ui.label("Svo Capacity:");
                                ui.label(format!("{:?}", svo.node_pool.capacity()));
                            });
                    }
                }
                
                egui::CollapsingHeader::new("Display Toggles").show(ui, |ui| {
                    ui.checkbox(&mut scene.display_toggles.show_axes, "Show Axes");
                    ui.checkbox(&mut scene.display_toggles.show_voxel_size_reference, "Show Voxel Size Reference");
                    ui.checkbox(&mut scene.display_toggles.show_wireframe, "Show Wireframe");
                    
                    egui::CollapsingHeader::new("Brick display options").show(ui, |ui| {
                        
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
                        
                        checkbox!(svo_sdf_brick::DisplayOptions::DEPTH,      "Depth");
                        checkbox!(svo_sdf_brick::DisplayOptions::NORMALS,    "Normals");
                        checkbox!(svo_sdf_brick::DisplayOptions::SOLID,      "Solid");
                        checkbox!(svo_sdf_brick::DisplayOptions::STEP_COUNT, "Step Count");
                    });
                
            });
        });
    }
}