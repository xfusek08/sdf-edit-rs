
use crate::{
    shape_builder::Shape,
    demo_app::scene::Scene,
    sdf::geometry::EvaluationStatus,
    framework::{math::Transform, gui::GuiModule},
};

#[derive(PartialEq)]
struct GeometryDynamicData {
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
            shape = shape.add(
                Shape::sphere(geometry_dynamic_data.radius),
                geometry_dynamic_data.transform.clone(),
                geometry_dynamic_data.blending
            );
        }
        
        geometry.edits = shape.build();
        geometry.evaluation_status = EvaluationStatus::NeedsEvaluation;
    }
}

impl GuiModule<Scene> for DynamicTestGeometry {
    fn gui(&mut self, scene: &mut Scene, ui: &mut egui::Ui) {
        let mut changed = false;
        let mut to_delete_indices: Vec<usize> = vec![];
        let mut to_add: Vec<GeometryDynamicData> = vec![];
        
        egui::CollapsingHeader::new("Dynamic Test Geometry")
            .default_open(true)
            .show(ui, |ui| {
                for (i, geometry_dynamic_data) in self.geometry_dynamic_data.iter_mut().enumerate() {
                    let mut transform = geometry_dynamic_data.transform.clone();
                    let mut radius = geometry_dynamic_data.radius.clone();
                    let mut blending = geometry_dynamic_data.blending.clone();
                    
                    ui.horizontal(|ui| {
                        // x integer input
                        ui.label("position:");
                        ui.add(egui::DragValue::new(&mut transform.position.x).speed(0.01));
                        ui.add(egui::DragValue::new(&mut transform.position.y).speed(0.01));
                        ui.add(egui::DragValue::new(&mut transform.position.z).speed(0.01));
                        
                        // radius
                        ui.label("radius:");
                        ui.add(egui::DragValue::new(&mut radius).speed(0.01).clamp_range(0.01..=1.0));
                        
                        // blending
                        ui.label("blending:");
                        ui.add(egui::DragValue::new(&mut blending).speed(0.01).clamp_range(0.0..=1.0));
                        
                        // delete button with gray x emoji
                        if ui.button("✖").clicked() {
                            to_delete_indices.push(i);
                        }
                    });
                    
                    let new_data = GeometryDynamicData {
                        transform,
                        radius,
                        blending,
                    };
                    
                    if new_data != *geometry_dynamic_data {
                        *geometry_dynamic_data = new_data;
                        changed = true;
                    }
                }
                // add button with plus emoji
                if ui.button("➕").clicked() {
                    to_add.push(GeometryDynamicData {
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
    }
}
