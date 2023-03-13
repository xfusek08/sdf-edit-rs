
use crate::{
    framework::gui::GuiModule,
    demo_app::scene::Scene
};

pub struct CountersGui;

impl GuiModule<Scene> for CountersGui {
    fn gui_window(&mut self, _: &mut Scene, _: &egui::Context) {}

    fn gui_section(&mut self, _: &mut Scene, ui: &mut egui::Ui) {
        counters::with_counters!(|counters| {
            if let Some(fc) = counters.get("frame_counter") {
                ui.label(format!("Frames: {:.0}", fc.total));
                ui.label(format!("FPS: {:.3}", fc.sum_past_values_second()));
                ui.label(format!("Last Frame time: {:.3} ms", fc.duration_of_last_sample().as_secs_f64() * 1000.0));
                ui.label(format!("Frame Time Average: {:.3} ms", fc.average_duration_past(100).as_secs_f64() * 1000.0));
            }
            ui.separator();
            if let Some(fc) = counters.get("update_counter") {
                ui.label(format!("UPS: {:.3}", fc.sum_past_values_second()));
                ui.label(format!("Last Update time: {:.3} ms", fc.duration_of_last_sample().as_secs_f64() * 1000.0));
                ui.label(format!("Update Time Average: {:.3} ms", fc.average_duration_past(100).as_secs_f64() * 1000.0));
            }
            ui.separator();
            if let Some(fc) = counters.get("event_counter") {
                ui.label(format!("Events: {:.0}", fc.total));
                ui.label(format!("EPS: {:.3}", fc.sum_past_values_second()));
            }
            ui.separator();
            let object_total = counters.get_latest_value("object_instance_counter");
            let object_selected = counters.get_latest_value("object_selected_counter");
            let brick_selected = counters.get_latest_value("brick_selected_counter");
            ui.label(format!("Object: {:.0}/{:.0}", object_selected, object_total));
            ui.label(format!("Bricks: {:.0}", brick_selected));
        });
    }
}
