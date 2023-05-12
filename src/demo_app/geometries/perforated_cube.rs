
use crate::shape_builder::Shape;

pub fn perforated_cube() -> Shape {
    let json_string = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/models/perforated_cube.json"));
    Shape::from_string(json_string).expect("Failed to load dip_demo_1 geometry")
}
