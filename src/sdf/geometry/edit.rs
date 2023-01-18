
use crate::framework::math::{Transform, AABB};
use super::{Primitive, Operation};


#[derive(Clone, Debug)]
pub struct Edit {
    pub primitive: Primitive,
    pub operation: Operation,
    pub transform: Transform,
    pub blending:  f32,
}

impl Edit {
    pub fn aabb(&self) -> AABB {
        self.primitive
            .aabb()
            .transform(&self.transform)
            .inflate(0.05)
    }
}
