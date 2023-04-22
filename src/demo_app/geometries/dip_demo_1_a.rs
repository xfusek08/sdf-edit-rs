
use crate::shape_builder::Shape;

pub fn dip_demo_1_a() -> Shape {
    Shape::from_string(include_str!("dip_demo_1_a.json")).expect("Failed to load dip_demo_1 geometry")
}
