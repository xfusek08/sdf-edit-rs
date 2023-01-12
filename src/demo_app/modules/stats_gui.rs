use egui::{Layout, Align};
use egui_extras::{TableBuilder, Column};

use crate::{
    framework::gui::GuiModule,
    demo_app::scene::Scene
};

pub struct StatsGui;

impl GuiModule<Scene> for StatsGui {
    fn gui(&mut self, scene: &mut Scene, egui_ctx: &egui::Context) {
        egui::Window::new("Statistics").show(egui_ctx, |ui| {
            let mut statistics_guard = profiler::STATISTICS.lock();
            let Some(statistics) = statistics_guard.as_mut() else { return; };
            
            // searchbar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut statistics.filter);
                if ui.button("✖").clicked() {
                    statistics.filter.clear();
                }
            });
            
            // to pin stat names:
            let mut to_pin: Option<&'static str> = None;
            let mut to_unpin: Option<&'static str> = None;
            
            let table = TableBuilder::new(ui)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto()) // pin button
                .column(Column::auto().resizable(true).clip(true)) // name
                .column(Column::initial(50.0)) // last value
                .column(Column::initial(50.0)) // average
                .column(Column::initial(50.0)) // max
                .column(Column::initial(50.0)) // min
                .column(Column::remainder())
                .min_scrolled_height(0.0);
            
            table
                .header(20.0, |mut header| {
                    header.col(|_| { });
                    header.col(|ui| { ui.strong("Name"); });
                    header.col(|ui| { ui.strong("Latest"); });
                    header.col(|ui| { ui.strong("Average"); });
                    header.col(|ui| { ui.strong("Max"); });
                    header.col(|ui| { ui.strong("Min"); });
                })
                .body(|mut body| {
                    // display pinned
                    for (name, stats) in statistics.pinned() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                // an pinned icon button
                                if ui.button("★").clicked() {
                                    to_unpin = Some(name);
                                }
                                });
                            row.col(|ui| { ui.label(name); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.latest())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.average())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.max_time)); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.min_time)); });
                        });
                    }
                    // display separator blank row
                    body.row(25.0, |mut row| {
                        row.col(|_| { });
                        row.col(|ui| { ui.strong("Unpinned:"); });
                    });
                    
                    // display unpinned
                    for (name, stats) in statistics.unpinned() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                // an unpinned icon button
                                if ui.button("☆").clicked() {
                                    to_pin = Some(name);
                                }
                                });
                            row.col(|ui| { ui.label(name); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.latest())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.average())); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.max_time)); });
                            row.col(|ui| { ui.label(format!("{:?}", stats.min_time)); });
                        });
                    }
                });
                
            if let Some(name) = to_pin {
                statistics.pin(name);
            }
            if let Some(name) = to_unpin {
                statistics.unpin(name);
            }
        });
    }
}
