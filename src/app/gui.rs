/// This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use egui::{Style, epaint::ClippedShape};
use winit::event_loop::EventLoopWindowTarget;

use super::state::Scene;

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
