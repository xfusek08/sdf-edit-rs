
use std::f32::consts::PI;

use crate::{
    shape_builder::Shape,
    framework::math::Transform
};

fn spiral_bumps_on_sphere(number_of_bumps: u32, number_of_wraps: u32, bump_radius: f32, radius: f32) -> Shape {
    let mut shape = Shape::empty();
    let f_number_of_bumps = number_of_bumps as f32;
    let f_number_of_wraps = number_of_wraps as f32;
    for i in 0..number_of_bumps {
        let theta = PI * i as f32 / f_number_of_bumps;
        let phi = 2.0 * theta * f_number_of_wraps;
        
        // TODO: Move "PI" and "2.0 *" transforms into from_polar to make it normalized to 0..1 range
        
        let p = Transform::from_polar(radius, theta, phi);
        shape = shape.add(
            // Shape::cube(bump_radius * 0.6, bump_radius * 0.6, bump_radius * 0.6),
            // p.clone().rotate(glam::Quat::from_rotation_arc(glam::Vec3::Y, p.position)),
            Shape::sphere(bump_radius),
            Transform::from_polar(radius, theta, phi),
            0.0
        );
    }
    shape
}

pub fn bumpy_sphere() -> Shape {
    let result = Shape::sphere(1.0)
        .subtract(
            spiral_bumps_on_sphere(400, 17, 0.1, 1.02),
            Transform::IDENTITY,
            0.03
        );
    result
}
