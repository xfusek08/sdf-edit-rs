
use crate::app::{
    gui::{self, GuiDataToRender},
    application::ControlFlowResultAction,
    updating::{UpdateContext, UpdaterModule, InputUpdateResult, ResizeContext}, state::State,
};

pub struct GuiUpdater;

impl UpdaterModule for GuiUpdater {
    
    fn input(&mut self, context: &mut UpdateContext) -> InputUpdateResult {
        InputUpdateResult::default()
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
        
        let State { gui, scene, ..} = context.state;
        
        let raw_input = gui.egui_winit.take_egui_input(context.window);
        
        // Run gui
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = gui.egui_ctx.run(raw_input, |egui_ctx| gui::gui(egui_ctx, scene));
        
        // Update window state (mainly to change cursors)
        gui.egui_winit.handle_platform_output(context.window, &gui.egui_ctx, platform_output);
        
        // Check if event should propagate to the scene
        let handled = gui.egui_ctx.is_using_pointer() || gui.egui_ctx.wants_keyboard_input();
        
        // Update textures, if any. Renderer is responsible for taking (removing gui.textures_delta) when it renders
        let textures_delta = if !textures_delta.is_empty() {
            if let Some(mut data) = gui.data_to_render.take() {
                data.textures_delta.append(textures_delta);
                data.textures_delta
            } else  { textures_delta }
        } else { textures_delta };
        
        gui.data_to_render = Some(GuiDataToRender { textures_delta, shapes });
        
        // Gui might consume event stopping its propagation to the scene
        InputUpdateResult {
            handled,
            result: ControlFlowResultAction::None,
        }
    }
    
}
