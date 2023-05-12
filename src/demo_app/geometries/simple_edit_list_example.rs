
use crate::shape_builder::Shape;

pub fn simple_edit_list_example() -> Shape {
    let json_string = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/models/simple_edit_list_example.json"));
    Shape::from_string(json_string).expect("Failed to load dip_demo_1 geometry")
}
