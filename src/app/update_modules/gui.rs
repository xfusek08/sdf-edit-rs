
use std::sync::Arc;

use crate::app::{
    gui,
    application::ControlFlowResultAction,
    updating::{UpdateContext, UpdaterModule, InputUpdateResult, ResizeContext},
};

pub struct GuiUpdater;

impl UpdaterModule for GuiUpdater {
    
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
        self.update_internal(context)
    }
    
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        self.update_internal(context).result
    }
    
    fn resize(&mut self, _: &mut ResizeContext) -> ControlFlowResultAction {
        ControlFlowResultAction::None
    }
    
}

impl GuiUpdater {
    
    #[profiler::function]
    fn update_internal(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
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
    
}
