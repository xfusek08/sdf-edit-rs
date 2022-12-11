
use enum_utils::ToIndex;

use crate::framework::math::AABB;

/// Might carry additional data which cannot be expressed by Transform
#[derive(Debug, PartialEq, Clone, ToIndex)]
pub enum Primitive {
    Sphere   { radius: f32 },
    Cube     { width: f32, height: f32, depth: f32 },
    Cylinder { radius: f32, height: f32 },
    Torus    { inner_radius: f32, outer_radius: f32 },
    Cone     { base_radius: f32 },
    Capsule  { top_radius: f32, bottom_radius: f32, height: f32 },
}

impl Primitive {
    pub fn dimensions(&self) -> [f32;4] {
        match self {
            Primitive::Sphere   { radius } => [*radius, 0.0, 0.0, 0.0],
            Primitive::Cube     { width, height, depth } => [*width, *height, *depth, 0.0],
            Primitive::Cylinder { radius, height } => [*radius, *height, 0.0, 0.0],
            Primitive::Torus    { inner_radius, outer_radius } => [*inner_radius, *outer_radius, 0.0, 0.0],
            Primitive::Cone     { base_radius } => [*base_radius, 0.0, 0.0, 0.0],
            Primitive::Capsule  { top_radius, bottom_radius, height } => [*top_radius, *bottom_radius, *height, 0.0],
        }
    }
    
    /// TODO: account for blending
    pub fn aabb(&self) -> AABB {
        match self {
            Primitive::Sphere { radius } => AABB::new(
                glam::Vec3::splat(-radius.clone()),
                glam::Vec3::splat(radius.clone())
            ),
            Primitive::Cube { width, height, depth } => AABB::new(
                glam::Vec3::new(-width.clone(), -height.clone(), -depth.clone()),
                glam::Vec3::new(width.clone(), height.clone(), depth.clone())
            ),
            Primitive::Cylinder { radius, height } => AABB::new(
                glam::Vec3::new(-radius.clone(), -height.clone(), -radius.clone()),
                glam::Vec3::new(radius.clone(), height.clone(), radius.clone())
            ),
            Primitive::Torus { inner_radius, outer_radius } => AABB::new(
                glam::Vec3::new(-outer_radius.clone(), -outer_radius.clone(), -inner_radius.clone()),
                glam::Vec3::new(outer_radius.clone(), outer_radius.clone(), inner_radius.clone())
            ),
            Primitive::Cone { base_radius } => AABB::new(
                glam::Vec3::new(-base_radius.clone(), 0.0, -base_radius.clone()),
                glam::Vec3::new(base_radius.clone(), 1.0, base_radius.clone())
            ),
            Primitive::Capsule { top_radius, bottom_radius, height } => AABB::new(
                glam::Vec3::new(-top_radius.clone(), -height.clone(), -bottom_radius.clone()),
                glam::Vec3::new(top_radius.clone(), height.clone(), bottom_radius.clone())
            ),
        }
    }
}
