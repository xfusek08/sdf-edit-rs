
use crate::{
    sdf::geometry::{Geometry, self},
    demo_app::modules::svo_sdf_brick,
};

use super::scene::Scene;

#[profiler::function]
pub fn style_gui(mut style: egui::Style) -> egui::Style {
    // adjust intrusive window shadowing
    style.visuals.window_shadow = egui::epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::BLACK,
    };
    style
}

#[profiler::function]
pub fn draw_gui(ctx: &egui::Context, scene: &mut Scene) {
    scene.counters.gui_updates += 1;
    draw_main_window(ctx, scene);
    
        
    // if stats feature is enabled, show stats window
    #[cfg(feature = "stats")]
    draw_stats_window(ctx, scene);
}

// Private functions
// -----------------

fn draw_main_window(ctx: &egui::Context, scene: &mut Scene) {
    egui::Window::new("Apps")
        .default_pos((10.0, 10.0))
        .show(ctx, |ui| {
            
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
                    ui.add(
                        egui::Slider::new(&mut scene.tmp_evaluator_config.min_voxel_size, Geometry::VOXEL_SIZE_RANGE)
                            .step_by(0.001)
                            .clamp_to_range(true)
                    );
                    
                });
            
            ui.separator();
            
            egui::Grid::new("grid_2")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Camera fov:");
                    ui.add(
                        egui::Slider::new(&mut scene.camera.fov, 10.0..=150.0)
                            // .step_by(step as f64)
                            .clamp_to_range(true)
                    );
                    ui.end_row();
            });
            
            ui.separator();
            
            let mut render_level = scene.tmp_evaluator_config.render_level;
            
            ui.label("TMP SVO Stats");
            for (geometry_id, geometry) in scene.geometry_pool.iter() {
                let id = format!("{:?}", geometry_id);
                ui.label(format!("Geometry: {}", id));
                ui.label("SVO:");
                if let Some(svo) = geometry.svo.as_ref() {
                    egui::Grid::new(&id).num_columns(2).show(ui, |ui| {
                        ui.label("Render Svo Level");
                        
                        let levels = svo.levels.len() as u32;
                        if levels > 0 {
                            ui.add(egui::Slider::new(
                                &mut render_level,
                                0..=levels - 1,
                            ).step_by(1.0).clamp_to_range(true));
                        }
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
                } else {
                    ui.label("None");
                }
                
                scene.tmp_evaluator_config.render_level = render_level;
                
                ui.spacing();
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
                    
                    checkbox!(svo_sdf_brick::DisplayOptions::DEPTH,        "Depth");
                    checkbox!(svo_sdf_brick::DisplayOptions::NORMALS,      "Normals");
                    checkbox!(svo_sdf_brick::DisplayOptions::SOLID,        "Solid");
                    checkbox!(svo_sdf_brick::DisplayOptions::STEP_COUNT,   "Step Count");
                });
            });
        });
}

#[cfg(feature = "stats")]
fn draw_stats_window(ctx: &egui::Context, _: &mut Scene) {
    use egui_extras::{TableBuilder, Column};

    egui::Window::new("Stats")
        .default_pos((10.0, 10.0))
        .show(ctx, |ui| {
            
            let mut statistics_guard = profiler::STATISTICS.lock();
            let Some(statistics) = statistics_guard.as_mut() else { return; };
            
            // searchbar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut statistics.filter);
                if ui.button("✖").clicked() {
                    statistics.filter.clear();
                }
            });
            
            // to pin stat names:
            let mut to_pin: Option<&'static str> = None;
            let mut to_unpin: Option<&'static str> = None;
            
            let table = TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto()) // pin button
                .column(Column::auto().resizable(true).clip(true)) // name
                .column(Column::initial(50.0)) // last value
                .column(Column::initial(50.0)) // average
                .column(Column::initial(50.0)) // max
                .column(Column::initial(50.0)) // min
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            
            table
                .header(20.0, |mut header| {
                    header.col(|_| { });
                    header.col(|ui| { ui.strong("Name"); });
                    header.col(|ui| { ui.strong("Latest"); });
                    header.col(|ui| { ui.strong("Average"); });
                    header.col(|ui| { ui.strong("Max"); });
                    header.col(|ui| { ui.strong("Min"); });
                })
                .body(|mut body| {
                    // display pinned
                    for (name, stats) in statistics.pinned() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                // an pinned icon button
                                if ui.button("★").clicked() {
                                    to_unpin = Some(name);
                                }
                             });
                            row.col(|ui| { ui.label(name); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.latest())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.average())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.max_time)); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.min_time)); });
                        });
                    }
                    // display separator blank row
                    body.row(25.0, |mut row| {
                        row.col(|_| { });
                        row.col(|ui| { ui.strong("Unpinned:"); });
                    });
                    
                    // display unpinned
                    for (name, stats) in statistics.unpinned() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                // an unpinned icon button
                                if ui.button("☆").clicked() {
                                    to_pin = Some(name);
                                }
                             });
                            row.col(|ui| { ui.label(name); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.latest())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.average())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.max_time)); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.min_time)); });
                        });
                    }
                });
                
            if let Some(name) = to_pin {
                statistics.pin(name);
            }
            if let Some(name) = to_unpin {
                statistics.unpin(name);
            }
        });
}
