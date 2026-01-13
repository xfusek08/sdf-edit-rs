use serde::{Deserialize, Serialize};

use super::{Operation, Primitive};
use crate::framework::math::{Transform, AABB};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edit {
    pub primitive: Primitive,
    pub operation: Operation,
    pub transform: Transform,
    pub blending: f32,
    pub color: glam::Vec4,
}

impl Edit {
    pub fn aabb(&self) -> AABB {
        self.primitive
            .aabb()
            .transform(&self.transform)
            .inflate(0.05)
    }
}
