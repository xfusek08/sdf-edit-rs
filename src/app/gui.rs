/// This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use std::sync::Arc;
use egui::Style;
use winit::event_loop::EventLoopWindowTarget;

use super::{
    updating::{InputUpdateResult, UpdateContext},
    application::ControlFlowResultAction,
    scene::Scene
};

pub struct Gui {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub textures_delta: Option<egui::TexturesDelta>,
    pub paint_jobs: Arc<Vec<egui::epaint::ClippedPrimitive>>, // this is shared with the renderer
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
            paint_jobs: Default::default(),
            textures_delta: Some(Default::default()),
        }
    }
    
    #[profiler::function]
    pub fn on_event(&mut self, event: &winit::event::WindowEvent<'_>) -> bool {
        self.egui_winit.on_event(&self.egui_ctx, event)
    }
    
    #[profiler::function]
    pub fn update(&mut self, scene: &mut Scene, context: &UpdateContext) -> InputUpdateResult {
        let raw_input = self.egui_winit.take_egui_input(context.window);
        
        // Run gui
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = self.egui_ctx.run(raw_input, |egui_ctx| gui(egui_ctx, scene));
        
        // Update window state (mainly to change cursors)
        self.egui_winit.handle_platform_output(context.window, &self.egui_ctx, platform_output);
        
        // Check if event should propagate to the scene
        let handled = self.egui_ctx.is_using_pointer() || self.egui_ctx.wants_keyboard_input();
        
        // Update output data which will be rendered next frame
        self.paint_jobs = Arc::new(self.egui_ctx.tessellate(shapes));
        
        // Update textures, if any. Renderer is responsible for taking (removing self.textures_delta) when it renders
        if !textures_delta.is_empty() {
            if let Some(old_textures_delta) = self.textures_delta.as_mut() {
                old_textures_delta.append(textures_delta);
            } else  {
                self.textures_delta = Some(textures_delta);
            }
        }
        
        // Gui might consume event stopping its propagation to the scene
        InputUpdateResult {
            handled,
            result: ControlFlowResultAction::None,
        }
    }
}

fn style_gui(mut style: Style) -> Style {
    // adjust intrusive window shadowing
    style.visuals.window_shadow = egui::epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::BLACK,
    };
    style
}

#[profiler::function]
fn gui(ctx: &egui::Context, scene: &mut Scene) {
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
