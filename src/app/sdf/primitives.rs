
#[derive(Clone)]
pub enum Primitive {
    Sphere {
        center: glam::Vec3,
        radius: f32,
    },
}
