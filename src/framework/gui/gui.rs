use egui_winit::EventResponse;
use winit::{event::WindowEvent, event_loop::EventLoopWindowTarget};

pub struct Gui {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub data_to_render: Option<GuiDataToRender>,
}

pub struct GuiDataToRender {
    pub textures_delta: egui::TexturesDelta,
    pub shapes: Vec<egui::epaint::ClippedShape>,
}

impl Gui {
    #[profiler::function]
    pub fn new<T, F>(event_loop: &EventLoopWindowTarget<T>, style_gui: F) -> Self
    where
        F: FnOnce(egui::Style) -> egui::Style,
    {
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
    pub fn on_event(&mut self, event: &winit::event::WindowEvent<'_>) -> EventResponse {
        self.egui_winit.on_event(&self.egui_ctx, event)
    }
}
