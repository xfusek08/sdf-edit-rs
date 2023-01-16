
use egui::{Layout, Align};
use egui_extras::{TableBuilder, Column};
use strum::IntoEnumIterator;

use crate::{
    shape_builder::{Shape, ShapeRecord},
    demo_app::scene::Scene,
    sdf::geometry::{EvaluationStatus, Primitive, PrimitiveType, Operation},
    framework::{math::Transform, gui::GuiModule}, warn,
};

/// This module controls first geometry in geometry pool
/// It allow to add and control spheres in the geometry
#[derive(Default)]
pub struct DynamicTestGeometry {
    shape: Option<Shape>,
}

impl DynamicTestGeometry {
    
    pub fn new() -> Self {
        Self {
            shape: Some(Shape::default())
        }
    }
    
    fn update_geometry(&self, scene: &mut Scene) {
        // Replace first unit geometry in scene
        let Some((_, geometry)) = scene.geometry_pool.iter_mut().next() else {
            warn!("DynamicTestGeometry::update_geometry called with no geometry");
            return;
        };
        let Some(shape) = self.shape.as_ref() else {
            warn!("DynamicTestGeometry::update_geometry called with no shape");
            return;
        };
        geometry.edits = shape.build();
        geometry.evaluation_status = EvaluationStatus::NeedsEvaluation;
    }
}

impl GuiModule<Scene> for DynamicTestGeometry {
    fn gui(&mut self, scene: &mut Scene, egui_ctx: &egui::Context) {
        
        let Some(mut shape) = self.shape.take() else {
            return;
        };
        
        // Obtain list of shapes
        let shapes = match &mut shape {
            Shape::Composite(a) => a,
            _ => {
                warn!("shape_composite_window_gui called on non-composite shape");
                return;
            },
        };
        
        let mut changed = false;
        let mut to_delete_indices: Vec<usize> = vec![];
        let mut to_add: Vec<Shape> = vec![];
        
        // Open window for this shape composite
        egui::Window::new("Dynamic geometry").show(egui_ctx, |ui| {
            TableBuilder::new(ui)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto()) // Primitive
                .column(Column::auto()) // Operation
                .column(Column::auto()) // Position
                .column(Column::auto()) // Rotation
                .column(Column::auto()) // Blending
                .column(Column::auto())// Dimensions
                .column(Column::auto())// delete action
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("Primitive"); });
                    header.col(|ui| { ui.strong("Operation"); });
                    header.col(|ui| { ui.strong("Position"); });
                    header.col(|ui| { ui.strong("Rotation"); });
                    header.col(|ui| { ui.strong("Blending"); });
                    header.col(|ui| { ui.strong("Dimensions"); });
                })
                .body(|mut body| {
                    for (i, shape_record) in shapes.iter_mut().enumerate() {
                        
                        // Deconstruct shape record
                        let (
                            primitive,
                            operation,
                            transform,
                            blending,
                        ) = match shape_record {
                            ShapeRecord {
                                shape: Shape::Primitive(p),
                                operation: o,
                                transform: t,
                                blending: b,
                                ..
                            } => (p, o, t, b),
                            _ => continue,
                        };
                        
                        body.row(20.0, |mut row| {
                        
                            // Primitive selector
                            row.col(|ui| {
                                let mut p_type = primitive.as_type();
                                primitive_type_sector_ui(format!("{i}_prim"), ui, &mut p_type);
                                if p_type != primitive.as_type() {
                                    *primitive = Primitive::from_type(p_type);
                                    changed = true;
                                }
                            });
                            
                            // Operation selector
                            row.col(|ui| {
                                let mut op = operation.clone();
                                operation_sector_ui(format!("{i}_op"), ui, &mut op);
                                if op != *operation {
                                    *operation = op;
                                    changed = true;
                                }
                            });
                            
                            // Position
                            row.col(|ui| {
                                let pos = transform.position.clone();
                                ui.add(egui::DragValue::new(&mut transform.position.x).speed(0.01).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut transform.position.y).speed(0.01).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut transform.position.z).speed(0.01).max_decimals(3).min_decimals(3));
                                changed = changed || pos != transform.position;
                            });
                            
                            // Rotation
                            row.col(|ui| {
                                let mut rot = transform.rotation.to_euler(glam::EulerRot::XYZ);
                                ui.add(egui::DragValue::new(&mut rot.0).speed(0.01).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut rot.1).speed(0.01).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut rot.2).speed(0.01).max_decimals(3).min_decimals(3));
                                if rot != transform.rotation.to_euler(glam::EulerRot::XYZ) {
                                    transform.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, rot.0, rot.1, rot.2);
                                    changed = true;
                                }
                            });
                            
                            // Blending
                            row.col(|ui| {
                                let b = *blending;
                                ui.add(egui::DragValue::new(blending).speed(0.01).max_decimals(3).min_decimals(3).clamp_range(0.0..=1.0));
                                changed = changed || b != *blending;
                            });
                            
                            // Dimensions
                            row.col(|ui| {
                                let dimensions: glam::Vec4 = primitive.dimensions().into();
                                match primitive {
                                    Primitive::Sphere { radius } => {
                                        ui.add(egui::DragValue::new(radius).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Cube { width, height, depth } => {
                                        ui.add(egui::DragValue::new(width).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(depth).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Cylinder { radius, height } => {
                                        ui.add(egui::DragValue::new(radius).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Torus { inner_radius, outer_radius } => {
                                        ui.add(egui::DragValue::new(inner_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(outer_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Cone { base_radius } => {
                                        ui.add(egui::DragValue::new(base_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Capsule { top_radius, bottom_radius, height } => {
                                        ui.add(egui::DragValue::new(top_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(bottom_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                }
                                changed = changed || dimensions != primitive.dimensions().into();
                            });
                            
                            // delete button with gray x emoji
                            row.col(|ui| {
                                if ui.button("✖").clicked() {
                                    to_delete_indices.push(i);
                                }
                            });
                            
                        }); // row
                    } // for shape record
                }); // table builder body
                
                // Add button
                if ui.button("➕").clicked() {
                    to_add.push(Shape::sphere(0.5));
                }
        }); // window end
        
        for i in to_delete_indices.drain(..) {
            shapes.remove(i);
            changed = true;
        }
        
        for new_child in to_add.drain(..) {
            shape = shape.add(new_child, Transform::default(), 0.0);
            changed = true;
        }
        
        self.shape = Some(shape);
        
        if changed {
            self.update_geometry(scene);
        }
    }
}

/// A simple combo box for selecting a primitive type.
/// - `id` is used to uniquely identify the combo box.
/// - `ui` is the ui to draw the combo box in.
/// - `p_type` is the primitive type to select and might be changed after the function returns.
fn primitive_type_sector_ui(id: impl std::hash::Hash, ui: &mut egui::Ui, p_type: &mut PrimitiveType) {
    egui::ComboBox::from_id_source(id)
        .width(20.0)
        .selected_text(p_type.as_ref())
        .show_ui(ui, |ui| {
            for t in PrimitiveType::iter() {
                ui.selectable_value(p_type, t.clone(), t.as_ref());
            }
        });
}

/// A simple combo box for selecting an operation.
/// - `id` is used to uniquely identify the combo box.
/// - `ui` is the ui to draw the combo box in.
/// - `operation` is the operation to select and might be changed after the function returns.
fn operation_sector_ui(id: impl std::hash::Hash, ui: &mut egui::Ui, operation: &mut Operation) {
    egui::ComboBox::from_id_source(id)
        .width(20.0)
        .selected_text(match operation {
            Operation::Add => "+".to_owned(),
            Operation::Subtract => "-".to_owned(),
            _ => "??".to_owned(),
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(operation, Operation::Add, "+");
            ui.selectable_value(operation, Operation::Subtract, "-");
        });
}
