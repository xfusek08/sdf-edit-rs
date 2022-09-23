use std::sync::Arc;

use crate::app::transform::Transform;

use super::geometry::Geometry;

// TODO: Create an enum from model to allow making a grouped model composites, where each node has transform and material but
// only leafs has geometry in addition.

pub struct Model {
    geometry: Arc<Geometry>,
    transform: Transform,
}
