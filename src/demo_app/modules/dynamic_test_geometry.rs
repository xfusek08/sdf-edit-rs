
use crate::{
    shape_builder::Shape,
    demo_app::scene::Scene,
    sdf::geometry::{EvaluationStatus, self},
    framework::{math::Transform, gui::GuiModule},
};

#[derive(Clone, PartialEq)]
struct GeometryDynamicData {
    operation: geometry::Operation,
    transform: Transform,
    radius: f32,
    blending: f32,
}

/// This module controls first geometry in geometry pool
/// It allow to add and control spheres in the geometry
pub struct DynamicTestGeometry {
    geometry_dynamic_data: Vec<GeometryDynamicData>,
}

impl DynamicTestGeometry {
    pub fn new() -> Self {
        Self {
            geometry_dynamic_data: vec![],
        }
    }

    fn update_geometry(&self, scene: &mut Scene) {
        let Some((_, geometry)) = scene.geometry_pool.iter_mut().next() else {
            return;
        };
        
        let mut shape = Shape::empty();
        
        for geometry_dynamic_data in self.geometry_dynamic_data.iter() {
            shape = match geometry_dynamic_data.operation {
                geometry::Operation::Add      => shape.add(Shape::sphere(geometry_dynamic_data.radius), geometry_dynamic_data.transform.clone(), geometry_dynamic_data.blending),
                geometry::Operation::Subtract => shape.subtract(Shape::sphere(geometry_dynamic_data.radius), geometry_dynamic_data.transform.clone(), geometry_dynamic_data.blending),
                _ => shape,
            }
        }
        
        geometry.edits = shape.build();
        geometry.evaluation_status = EvaluationStatus::NeedsEvaluation;
    }
}

impl GuiModule<Scene> for DynamicTestGeometry {
    fn gui(&mut self, scene: &mut Scene, egui_ctx: &egui::Context) {
        egui::Window::new("Dynamic geometry").show(egui_ctx, |ui| {
            let mut changed = false;
            let mut to_delete_indices: Vec<usize> = vec![];
            let mut to_add: Vec<GeometryDynamicData> = vec![];
            
            egui::CollapsingHeader::new("Dynamic Test Geometry")
                .default_open(true)
                .show(ui, |ui| {
                    for (i, geometry_dynamic_data) in self.geometry_dynamic_data.iter_mut().enumerate() {
                        let mut new_data = geometry_dynamic_data.clone();
                        
                        ui.horizontal(|ui| {
                            // operation
                            ui.label("op:");
                            egui::ComboBox::from_id_source(format!("{i}_op"))
                                .width(20.0)
                                .selected_text(match new_data.operation {
                                    geometry::Operation::Add => "+".to_owned(),
                                    geometry::Operation::Subtract => "-".to_owned(),
                                    _ => "??".to_owned(),
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut new_data.operation, geometry::Operation::Add, "+");
                                    ui.selectable_value(&mut new_data.operation, geometry::Operation::Subtract, "-");
                                });
                            
                            // x integer input
                            ui.label("position:");
                            ui.add(egui::DragValue::new(&mut new_data.transform.position.x).speed(0.01));
                            ui.add(egui::DragValue::new(&mut new_data.transform.position.y).speed(0.01));
                            ui.add(egui::DragValue::new(&mut new_data.transform.position.z).speed(0.01));
                            
                            // radius
                            ui.label("radius:");
                            ui.add(egui::DragValue::new(&mut new_data.radius).speed(0.01).clamp_range(0.01..=1.0));
                            
                            // blending
                            ui.label("blending:");
                            ui.add(egui::DragValue::new(&mut new_data.blending).speed(0.01).clamp_range(0.0..=1.0));
                            
                            // delete button with gray x emoji
                            if ui.button("✖").clicked() {
                                to_delete_indices.push(i);
                            }
                        });
                        
                        if new_data != *geometry_dynamic_data {
                            *geometry_dynamic_data = new_data;
                            changed = true;
                        }
                    }
                    // add button with plus emoji
                    if ui.button("➕").clicked() {
                        to_add.push(GeometryDynamicData {
                            operation: geometry::Operation::Add,
                            transform: Transform::default(),
                            radius: 0.2,
                            blending: 0.0,
                        });
                    }
                });
            
            for i in to_delete_indices.drain(..) {
                self.geometry_dynamic_data.remove(i);
                changed = true;
            }
            
            for geometry_dynamic_data in to_add.drain(..) {
                self.geometry_dynamic_data.push(geometry_dynamic_data);
                changed = true;
            }
            
            if changed {
                self.update_geometry(scene);
            }
        });
    }
}
