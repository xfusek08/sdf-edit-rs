
use enum_utils::ToIndex;
use serde::{Serialize, Deserialize};
use strum_macros::{AsRefStr, EnumIter};

use crate::framework::math::AABB;

#[derive(Clone, Debug, ToIndex, AsRefStr, EnumIter, PartialEq)]
pub enum PrimitiveType {
    Sphere,
    Cube,
    Cylinder,
    Torus,
    Cone,
    Capsule,
}

/// Might carry additional data which cannot be expressed by Transform
#[derive(Debug, PartialEq, Clone, ToIndex, Serialize, Deserialize)]
pub enum Primitive {
    Sphere   { radius: f32 },
    Cube     { width: f32, height: f32, depth: f32, bevel: f32 },
    Cylinder { diameter: f32, height: f32 },
    Torus    { inner_radius: f32, outer_radius: f32 },
    Cone     { diameter: f32, height: f32 },
    Capsule  { radius: f32, height: f32 },
}

// API - Primitive
impl Primitive {
    pub fn default_sphere()   -> Self { Primitive::Sphere   { radius: 1.0 } }
    pub fn default_cube()     -> Self { Primitive::Cube     { width: 1.0, height: 1.0, depth: 1.0, bevel: 0.0 } }
    pub fn default_cylinder() -> Self { Primitive::Cylinder { diameter: 1.0, height: 1.0 } }
    pub fn default_torus()    -> Self { Primitive::Torus    { inner_radius: 0.8, outer_radius: 0.2 } }
    pub fn default_cone()     -> Self { Primitive::Cone     { diameter: 1.0, height: 1.0 } }
    pub fn default_capsule()  -> Self { Primitive::Capsule  { radius: 0.5, height: 1.0 } }
    
    pub fn from_type(p_type: PrimitiveType) -> Primitive {
        match p_type {
            PrimitiveType::Sphere   => Primitive::default_sphere(),
            PrimitiveType::Cube     => Primitive::default_cube(),
            PrimitiveType::Cylinder => Primitive::default_cylinder(),
            PrimitiveType::Torus    => Primitive::default_torus(),
            PrimitiveType::Cone     => Primitive::default_cone(),
            PrimitiveType::Capsule  => Primitive::default_capsule(),
        }
    }
    
    pub fn as_type(&self) -> PrimitiveType {
        match self {
            Primitive::Sphere   { .. } => PrimitiveType::Sphere,
            Primitive::Cube     { .. } => PrimitiveType::Cube,
            Primitive::Cylinder { .. } => PrimitiveType::Cylinder,
            Primitive::Torus    { .. } => PrimitiveType::Torus,
            Primitive::Cone     { .. } => PrimitiveType::Cone,
            Primitive::Capsule  { .. } => PrimitiveType::Capsule,
        }
    }
    
    pub fn dimension_data(&self) -> [f32;4] {
        match self {
            Primitive::Sphere   { radius } =>
                [*radius, 0.0, 0.0, 0.0],
            Primitive::Cube     { width, height, depth, bevel } =>
                [*width, *height, *depth, *bevel],
            Primitive::Cylinder { diameter, height } =>
                [*diameter, *height, 0.0, 0.0],
            Primitive::Torus    { inner_radius, outer_radius } =>
                [*inner_radius, *outer_radius, 0.0, 0.0],
            Primitive::Cone     { diameter, height } =>
                [*diameter, *height, 0.0, 0.0],
            Primitive::Capsule  { radius, height } =>
                [*radius, *height, 0.0, 0.0],
        }
    }
    
    /// TODO: account for blending
    pub fn aabb(&self) -> AABB {
        match self {
            Primitive::Sphere { radius } => AABB::new(
                glam::Vec3::splat(-radius),
                glam::Vec3::splat(*radius)
            ),
            Primitive::Cube { width, height, depth, .. } => AABB::new(
                glam::Vec3::new(-width, -height, -depth) * 0.5,
                glam::Vec3::new(*width, *height, *depth) * 0.5,
            ),
            Primitive::Cylinder { diameter, height } => AABB::new(
                glam::Vec3::new(-diameter, -height, -diameter) * 0.5,
                glam::Vec3::new(*diameter, *height, *diameter) * 0.5,
            ),
            Primitive::Torus { inner_radius, outer_radius } => AABB::new(
                glam::Vec3::new(-inner_radius -outer_radius, -outer_radius, -inner_radius - outer_radius),
                glam::Vec3::new(inner_radius + outer_radius, *outer_radius, inner_radius + outer_radius)
            ),
            Primitive::Cone { diameter, height } => AABB::new(
                glam::Vec3::new(-diameter, -height, -diameter) * 0.5,
                glam::Vec3::new(*diameter, *height, *diameter) * 0.5,
            ),
            Primitive::Capsule { radius, height } => AABB::new(
                glam::Vec3::new(-radius, -height - radius, -radius),
                glam::Vec3::new(*radius, height + radius, *radius)
            ),
        }
    }
    
    // TODO: implement changing of type for primitive
}
