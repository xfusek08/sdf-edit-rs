
use egui_glium::EguiGlium;
use glium::glutin::dpi::PhysicalSize;
use winit_input_helper::WinitInputHelper;

use crate::framework::{Application, UpdateResult, Context, clock};

pub struct DemoApp {
    updates: u64,
    renders: u64,
    inputs: u64,
}

impl DemoApp {
    pub fn new(context: &Context) -> Self {
        Self {
            renders: 0,
            updates: 0,
            inputs: 0,
        }
    }
}

impl Application for DemoApp {
    #[profiler::function]
    fn style_gui(&mut self, mut style: egui::Style) -> egui::Style {
        style.visuals.window_shadow = egui::epaint::Shadow {
            extrusion: 0.0,
            color: egui::Color32::BLACK,
        };
        style
    }
    
    #[profiler::function]
    fn gui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Apps")
            .default_pos((10.0, 10.0))
            .show(ctx, |ui| {
                egui::Grid::new("grid_1")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Renders:");
                        ui.label(format!("{}", self.renders));
                        ui.end_row();
                        ui.label("Updates:");
                        ui.label(format!("{}", self.updates));
                        ui.end_row();
                        ui.label("Inputs:");
                        ui.label(format!("{}", self.inputs));
                    })
            });
    }
    
    #[profiler::function]
    fn input(&mut self, input: &WinitInputHelper, tick: &clock::Tick) -> UpdateResult {
        self.inputs += 1;
        UpdateResult::default()
    }
    
    #[profiler::function]
    fn update(&mut self, input: &WinitInputHelper, tick: &clock::Tick) -> UpdateResult {
        self.updates += 1;
        UpdateResult::default()
    }
    
    #[profiler::function]
    fn render(&mut self, display: &glium::Display, gui: &mut EguiGlium) {
        self.renders += 1;
        use glium::Surface as _; // imports Surface trait implementation for target
        let mut target = profiler::call!(display.draw());
        
        profiler::call!(target.clear_color(0.5, 0.6, 0.7, 1.0));
        
        // run rendering
        
        
        profiler::call!(gui.paint(&display, &mut target));
        profiler::call!(target.finish().unwrap());
    }
    
    #[profiler::function]
    fn resize(&mut self, size: PhysicalSize<u32>, scaling_factor: f64) {
    }
    
    #[profiler::function]
    fn exit(&mut self) {
    }
}
