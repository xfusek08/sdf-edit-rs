
use crate::shape_builder::Shape;

pub fn dip_demo_1_b() -> Shape {
    Shape::from_string(include_str!("dip_demo_1_b.json")).expect("Failed to load dip_demo_1 geometry")
}
