use crate::shape_builder::Shape;


pub fn test_geometry() -> Shape {
    Shape::from_string(include_str!("test_geometry.json"))
        .expect("Failed to load test geometry")
}
