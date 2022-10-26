use std::{ops::RangeInclusive, sync::Arc};
use slotmap::{new_key_type, SlotMap};
use crate::app::math::Transform;
use super::{svo::Octree, primitives::Primitive};

#[derive(Clone)]
pub enum GeometryOperation {
    Add,
    Subtract,
    Intersect,
    // TODO: Paint
}

#[derive(Clone)]
pub struct GeometryEdit {
    pub primitive: Primitive,
    pub operation: GeometryOperation,
    pub transform: Transform,
    pub blending:  f32,
}

pub type GeometryEditList = Vec<GeometryEdit>;

pub enum GeometryEvaluationStatus {
    NeedsEvaluation,
    Evaluating,
    Evaluated,
}

pub struct Geometry {
    
    /// A Sparse Voxel Octree evaluated into GPU memory
    pub svo: Option<Arc<Octree>>,
    
    /// A list of edits that compose this geometry
    pub edits: GeometryEditList,
    
    /// The status of the geometry evaluation used by evaluator
    ///   - `NeedsEvaluation` means that the geometry has been edited and needs to be evaluated
    ///      evaluator on next update collects all geometries with this status and spawns and evaluation job.
    ///   - `Evaluating` means that the geometry is currently being evaluated by evaluator.
    ///   - `Evaluated` means that the geometry does not need to be evaluated.
    pub evaluation_status: GeometryEvaluationStatus,
    
    /// nodes containing voxels bigger that this will be subdivided
    min_voxel_size: f32,
}

impl Geometry {
    pub const VOXEL_SIZE_RANGE: RangeInclusive<f32> = 0.001..=0.1;
    
    pub fn new(min_voxel_size: f32) -> Self {
        Self {
            svo:               None,
            edits:             vec![],
            evaluation_status: GeometryEvaluationStatus::NeedsEvaluation,
            min_voxel_size: min_voxel_size.clamp(*Self::VOXEL_SIZE_RANGE.start(), *Self::VOXEL_SIZE_RANGE.end()),
        }
    }
    
    pub fn with_edits(mut self, edits: GeometryEditList) -> Self {
        self.edits = edits;
        self
    }
    
    pub fn min_voxel_size(&self) -> f32 {
        self.min_voxel_size
    }
    
    pub fn set_min_voxel_size(&mut self, min_voxel_size: f32) {
        self.min_voxel_size = min_voxel_size.clamp(*Self::VOXEL_SIZE_RANGE.start(), *Self::VOXEL_SIZE_RANGE.end());
        self.evaluation_status = GeometryEvaluationStatus::NeedsEvaluation;
    }
    
}

new_key_type! {
    /// An index of geometry instance which can be shared between multiple models
    pub struct GeometryID;
}

pub type GeometryPool = SlotMap<GeometryID, Geometry>;
