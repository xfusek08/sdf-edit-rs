
use egui::{Layout, Align};
use egui_extras::{TableBuilder, Column};
use strum::IntoEnumIterator;

use crate::{
    warn,
    demo_app::scene::Scene,
    shape_builder::{Shape, ShapeRecord},
    sdf::geometry::{EvaluationStatus, Primitive, PrimitiveType, Operation},
    framework::{math::Transform, gui::GuiModule},
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
        
        let mut changed = false;
        let mut to_delete_indices: Vec<usize> = vec![];
        let mut to_add: Vec<Shape> = vec![];
        
        // Open window for this shape composite
        egui::Window::new("Dynamic geometry").show(egui_ctx, |ui| {
            let Some(mut shape) = self.shape.take() else {
                return;
            };
            
            ui.horizontal(|ui| {
                let fd = rfd::FileDialog::new().add_filter("json", &["json", "JSON"]);
                    
                if ui.button("Import").clicked() {
                    if let Some(file_name) = fd.clone().pick_file() {
                        if let Ok(new_shape) = Shape::load_store_edits(file_name) {
                            shape = new_shape;
                            changed = true;
                        } else {
                            warn!("Failed to load shape");
                        }
                    }
                }
                
                if ui.button("Export").clicked() {
                    if let Some(file_name) = fd.save_file() {
                        if let Err(e) = shape.store_flat_edits(file_name) {
                            warn!("Failed to save shape: {}", e);
                        }
                    }
                }
            });
            
            // Obtain list of shapes
            let shapes = match &mut shape {
                Shape::Composite(a) => a,
                _ => {
                    warn!("shape_composite_window_gui called on non-composite shape");
                    return;
                },
            };
            
            TableBuilder::new(ui)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto()) // Primitive
                .column(Column::auto()) // Operation
                .column(Column::auto()) // Position
                .column(Column::auto()) // Rotation
                .column(Column::auto()) // Blending
                .column(Column::auto())// Dimensions
                .column(Column::auto())// delete action
                .min_scrolled_height(0.0)
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
                                let orig_radians: glam::Vec3 = transform.rotation.to_euler(glam::EulerRot::XYZ).into();
                                
                                let (mut x_deg, mut y_deg, mut z_deg) = (orig_radians.x.to_degrees(), orig_radians.y.to_degrees(), orig_radians.z.to_degrees());
                                ui.add(egui::DragValue::new(&mut x_deg).speed(0.1).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut y_deg).speed(0.1).max_decimals(3).min_decimals(3));
                                ui.add(egui::DragValue::new(&mut z_deg).speed(0.1).max_decimals(3).min_decimals(3));
                                
                                let new_radians: glam::Vec3 = (x_deg.to_radians(), y_deg.to_radians(), z_deg.to_radians()).into();
                                if new_radians.distance_squared(orig_radians) > 0.0001 { // TODO: Magic squared error constant
                                    transform.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, new_radians.x, new_radians.y, new_radians.z);
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
                                    Primitive::Cube { width, height, depth, bevel } => {
                                        ui.add(egui::DragValue::new(width).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(depth).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(bevel).speed(0.01).max_decimals(3).min_decimals(3).clamp_range(0.0..=0.45));
                                    },
                                    Primitive::Cylinder { diameter, height } => {
                                        ui.add(egui::DragValue::new(diameter).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Torus { inner_radius, outer_radius } => {
                                        ui.add(egui::DragValue::new(inner_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(outer_radius).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Cone { diameter, height } => {
                                        ui.add(egui::DragValue::new(diameter).speed(0.01).max_decimals(3).min_decimals(3));
                                        ui.add(egui::DragValue::new(height).speed(0.01).max_decimals(3).min_decimals(3));
                                    },
                                    Primitive::Capsule { radius, height } => {
                                        ui.add(egui::DragValue::new(radius).speed(0.01).max_decimals(3).min_decimals(3));
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
                
                for i in to_delete_indices.drain(..) {
                    shapes.remove(i);
                    changed = true;
                    if shapes.len() == 0 {
                        to_add.push(Shape::sphere(0.5));
                    }
                }
                
                for new_child in to_add.drain(..) {
                    shape = shape.add(new_child, Transform::default(), 0.0);
                    changed = true;
                }
                
                self.shape = Some(shape);
                
        }); // window end
        
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
