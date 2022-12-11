
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
        let mut aabb = self.primitive.aabb();
        aabb.transform(&self.transform);
        aabb
    }
}
