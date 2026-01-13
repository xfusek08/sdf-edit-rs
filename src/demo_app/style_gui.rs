#[profiler::function]
pub fn style_gui(mut style: egui::Style) -> egui::Style {
    // adjust intrusive window shadowing
    style.visuals.window_shadow = egui::epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::BLACK,
    };
    style
}
