
use std::ops::{Deref, DerefMut};

use slotmap::{new_key_type, SlotMap};

use crate::framework::math::Transform;
use super::geometry::GeometryID;

new_key_type! {
    /// An index of geometry instance which can be shared between multiple models
    pub struct ModelID;
}

// TODO: Create an enum from model to allow making a grouped model composites, where each node has transform and material but
// only leafs has geometry in addition.

pub struct Model {
    pub geometry_id: GeometryID,
    pub transform: Transform,
    // TODO: Add (Optional?) material to model
}

impl Model {
    pub fn new(geometry_id: GeometryID) -> Self {
        Self {
            geometry_id,
            transform: Transform::default(),
        }
    }
    
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    
}

pub struct ModelPool {
    data: SlotMap<ModelID, Model>,
}

impl Deref for ModelPool {
    type Target = SlotMap<ModelID, Model>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ModelPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl ModelPool {
    pub fn new() -> Self {
        Self {
            data: SlotMap::with_key(),
        }
    }
}
