
use crate::shape_builder::Shape;

pub fn mickey_mouse() -> Shape {
    Shape::from_string(include_str!("mickey_mouse.json")).expect("Failed to load test geometry")
}
