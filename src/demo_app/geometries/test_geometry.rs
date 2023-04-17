
use rand::Rng;

use crate::{shape_builder::Shape, framework::math::Transform};

pub fn test_geometry() -> Shape {
    let mut shape = Shape::Composite(vec![]);
    let mut rng = rand::thread_rng();
    let pos_range = 0.0..=1.5;
    let color_range = 0.2..=0.99;
    
    for _ in 0..10 {
        shape = shape.add(
            Shape::sphere(0.6),
            Transform::from_xyz(
                rng.gen_range(pos_range.clone()),
                rng.gen_range(pos_range.clone()),
                0.0, // rng.gen_range(pos_range.clone())
            ),
            glam::Vec4::new(rng.gen_range(color_range.clone()), rng.gen_range(color_range.clone()), rng.gen_range(color_range.clone()), 1.0),
            0.2
        );
    }
    
    shape = shape
        .subtract(
            Shape::cube(5.0, 5.0, 2.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 1.3),
            glam::Vec4::ZERO,
            0.01
        )
        .subtract(
            Shape::cube(5.0, 5.0, 2.0, 0.0),
            Transform::from_xyz(-0.5, 0.0, -1.3),
            glam::Vec4::ZERO,
            0.01
        );
    
    shape
}
