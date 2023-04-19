
use crate::shape_builder::Shape;

pub fn mouse() -> Shape {
    Shape::from_string(include_str!("mouse.json")).expect("Failed to load test geometry")
}
