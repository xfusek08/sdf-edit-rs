/// This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use egui::{Style, epaint::ClippedShape};
use winit::event_loop::EventLoopWindowTarget;

use super::state::Scene;
use crate::app::sdf::geometry::Geometry;

#[profiler::function]
pub fn style_gui(mut style: Style) -> Style {
    // adjust intrusive window shadowing
    style.visuals.window_shadow = egui::epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::BLACK,
    };
    style
}

#[profiler::function]
pub fn gui(ctx: &egui::Context, scene: &mut Scene) {
    scene.counters.gui_updates += 1;
    
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
                    ui.label("Render Svo Level Begin:");
                    ui.add(egui::DragValue::new(&mut scene.tmp_evaluator_config.render_svo_level_begin).speed(1).clamp_range(0..=scene.tmp_evaluator_config.render_svo_level_end - 1));
                    ui.end_row();
                    ui.label("Render Svo Level End:");
                    ui.add(egui::DragValue::new(&mut scene.tmp_evaluator_config.render_svo_level_end).speed(1).clamp_range(scene.tmp_evaluator_config.render_svo_level_begin + 1..=20));
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
            
            ui.label("TMP SVO Stats");
            for (geometry_id, geometry) in scene.geometry_pool.iter() {
                let id = format!("{:?}", geometry_id);
                ui.label(format!("Geometry: {}", id));
                ui.label("SVO:");
                if let Some(svo) = geometry.svo.as_ref() {
                    egui::Grid::new(&id).num_columns(2).show(ui, |ui| {
                        ui.label("Svo Node Count:");
                        ui.label(format!("{:?}", svo.node_pool.count()));
                        ui.end_row();
                        ui.label("Svo Level Count:");
                        ui.label(format!("{}", svo.levels.len()));
                        ui.end_row();
                    });
                } else {
                    ui.label("None");
                }
                
                ui.spacing();
            }
        });
}

pub struct Gui {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub data_to_render: Option<GuiDataToRender>,
}

pub struct GuiDataToRender {
    pub textures_delta: egui::TexturesDelta,
    pub shapes: Vec<ClippedShape>,
}

impl Gui {
    
    #[profiler::function]
    pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
        let egui_ctx = egui::Context::default();
        
        // set global egui styling
        egui_ctx.set_style(style_gui((*egui_ctx.style()).clone()));
        
        Self {
            egui_ctx,
            egui_winit: egui_winit::State::new(event_loop),
            data_to_render: None,
        }
    }
    
    #[profiler::function]
    pub fn on_event(&mut self, event: &winit::event::WindowEvent<'_>) -> bool {
        self.egui_winit.on_event(&self.egui_ctx, event)
    }
    
}
