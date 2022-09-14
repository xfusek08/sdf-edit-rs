/// This file is inspired by: https://github.com/hasenbanck/egui_example/blob/master/src/main.rs

use std::sync::Arc;
use winit::event_loop::EventLoopWindowTarget;

use super::{
    updating::{InputUpdateResult, UpdateContext},
    application::ControlFlowResultAction,
    scene::Scene
};

pub struct Gui {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub textures_delta: egui::TexturesDelta,
    pub paint_jobs: Arc<Vec<egui::epaint::ClippedPrimitive>>, // this is shared with the renderer
}

impl Gui {
    #[profiler::function]
    pub fn new<T>(event_loop: &EventLoopWindowTarget<T>) -> Self {
        Self {
            egui_ctx: Default::default(),
            egui_winit: egui_winit::State::new(event_loop),
            paint_jobs: Default::default(),
            textures_delta: Default::default(),
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
        self.textures_delta.append(textures_delta);
        
        InputUpdateResult {
            handled,
            result: if repaint_after.is_zero() {
                    ControlFlowResultAction::Redraw
                } else {
                    ControlFlowResultAction::None
                }
        }
    }
}

#[profiler::function]
fn gui(ctx: &egui::Context, scene: &mut Scene) {
    egui::Window::new("asd")
        .show(ctx, |ui| {
            ui.label("Hello World!");
        });
}
