use rand::Rng;
use std::f32::consts::PI;

use crate::{framework::math::Transform, shape_builder::Shape};

fn spiral_bumps_on_sphere(
    number_of_bumps: u32,
    number_of_wraps: u32,
    bump_radius: f32,
    radius: f32,
) -> Shape {
    let mut rng = rand::thread_rng();
    let mut shape = Shape::empty();
    let f_number_of_bumps = number_of_bumps as f32;
    let f_number_of_wraps = number_of_wraps as f32;
    for i in 0..number_of_bumps {
        let theta = PI * i as f32 / f_number_of_bumps;
        let phi = 2.0 * theta * f_number_of_wraps;

        // TODO: Move "PI" and "2.0 *" transforms into from_polar to make it normalized to 0..1 range
        shape = shape.add(
            Shape::sphere(bump_radius),
            Transform::from_polar(radius, theta, phi),
            if rng.gen_range(0..=1) == 0 {
                glam::Vec4::new(0.7, 0.6, 0.1, 1.0)
            } else {
                glam::Vec4::new(0.1, 0.6, 0.7, 1.0)
            },
            0.0,
        );
    }
    shape
}

pub fn bumpy_sphere() -> Shape {
    let result = Shape::Composite(vec![])
        .add(
            Shape::sphere(1.0),
            Transform::IDENTITY,
            glam::Vec4::new(1.0, 0.1, 0.1, 1.0),
            0.0,
        )
        .subtract(
            spiral_bumps_on_sphere(600, 77, 0.09, 1.02),
            Transform::IDENTITY,
            glam::Vec4::ZERO,
            0.015,
        );
    result
}
