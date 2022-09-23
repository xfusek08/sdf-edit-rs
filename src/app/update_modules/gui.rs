
use std::{sync::Arc, time::Duration};

use crate::app::{
    updating::{UpdateContext, UpdaterModule, InputUpdateResult, ResizeContext},
    application::ControlFlowResultAction,
    gui,
};


#[derive(Default)]
pub struct GuiUpdater;

impl UpdaterModule for GuiUpdater {
    
    #[profiler::function]
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
        let raw_input = context.gui.egui_winit.take_egui_input(context.window);
        
        // Run gui
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = context.gui.egui_ctx.run(raw_input, |egui_ctx| gui::gui(egui_ctx, context.scene));
        
        // Update window state (mainly to change cursors)
        context.gui.egui_winit.handle_platform_output(context.window, &context.gui.egui_ctx, platform_output);
        
        // Check if event should propagate to the scene
        let handled = context.gui.egui_ctx.is_using_pointer() || context.gui.egui_ctx.wants_keyboard_input();
        
        // Update output data which will be rendered next frame
        context.gui.paint_jobs = Arc::new(context.gui.egui_ctx.tessellate(shapes));
        
        // Update textures, if any. Renderer is responsible for taking (removing context.gui.textures_delta) when it renders
        if !textures_delta.is_empty() {
            if let Some(old_textures_delta) = context.gui.textures_delta.as_mut() {
                old_textures_delta.append(textures_delta);
            } else  {
                context.gui.textures_delta = Some(textures_delta);
            }
        }
        
        // Gui might consume event stopping its propagation to the scene
        InputUpdateResult {
            handled,
            result: ControlFlowResultAction::None,
        }
    }
    
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        ControlFlowResultAction::None
    }
    
    fn resize(&mut self, context: &mut ResizeContext) -> ControlFlowResultAction {
        ControlFlowResultAction::None
    }
    
}
