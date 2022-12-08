
/// Might carry additional data which cannot be expressed by Transform
#[derive(Debug, Clone)]
pub enum Primitive {
    Sphere,
    Cube, // box is reserved keyword
    Cylinder,
    Torus   { inner_radius: f32, outer_radius: f32 },
    Cone    { base_radius: f32 },
    Capsule { top_radius: f32, bottom_radius: f32, height: f32 },
}

impl Primitive {
    pub fn get_id(&self) -> u32 {
        match self {
            Primitive::Sphere => 0,
            Primitive::Cube => 1,
            Primitive::Cylinder => 2,
            Primitive::Torus { .. } => 3,
            Primitive::Cone { .. } => 4,
            Primitive::Capsule { .. } => 5,
        }
    }
}
