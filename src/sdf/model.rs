use slotmap::{new_key_type, SlotMap};
use crate::framework::math::Transform;

use super::geometry::GeometryID;

new_key_type! {
    /// An index of geometry instance which can be shared between multiple models
    pub struct ModelID;
}

pub type ModelPool = SlotMap<ModelID, Model>;

// TODO: Create an enum from model to allow making a grouped model composites, where each node has transform and material but
// only leafs has geometry in addition.

pub enum ModelPayload {
    Geometry { geometry: GeometryID },
    Group    { children: Vec<ModelID> },
}

pub struct Model {
    pub payload: ModelPayload,
    pub transform: Transform,
    // TODO: Add (Optional?) material to model
}

impl Model {
    pub fn new(geometry: GeometryID) -> Self {
        Self {
            payload: ModelPayload::Geometry { geometry },
            transform: Transform::default(),
        }
    }
    
    pub fn new_group(children: Vec<ModelID>) -> Self {
        Self {
            payload: ModelPayload::Group { children },
            transform: Transform::default(),
        }
    }
    
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}
