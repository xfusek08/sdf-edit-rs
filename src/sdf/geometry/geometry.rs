use std::ops::RangeInclusive;

use slotmap::{new_key_type, SlotMap};

use crate::{sdf::svo::Svo, framework::math::AABB};

use super::{Edit, Operation};


// ============================================================================================
// Geometry Pool
// ============================================================================================

new_key_type! {
    /// An index of geometry instance which can be shared between multiple models
    pub struct GeometryID;
}
pub type GeometryPool = SlotMap<GeometryID, Geometry>;


// ============================================================================================
// Geometry Evaluation Status
// ============================================================================================

pub enum EvaluationStatus {
    NeedsEvaluation,
    Evaluating,
    Evaluated,
}


// ============================================================================================
// Geometry
// ============================================================================================

pub struct Geometry {
    
    /// An Evaluated SVO in GPU memory, when None, the geometry is not evaluated or is being evaluated and evaluator is owning it.
    /// To determine if the geometry is being evaluated, check the `evaluation_status` field.
    /// TODO: When evaluator will be asynchronous, this field will ne none until new svo is ready, but maybe we should keep the old one until new is ready.
    pub svo: Option<Svo>,
    
    /// The status of the geometry evaluation used by evaluator
    ///   - `NeedsEvaluation` means that the geometry has been edited and needs to be evaluated
    ///      evaluator on next update collects all geometries with this status and spawns and evaluation job.
    ///   - `Evaluating` means that the geometry is currently being evaluated by evaluator.
    ///   - `Evaluated` means that the geometry does not need to be evaluated.
    pub evaluation_status: EvaluationStatus,
    
    /// A list of edits that compose this geometry on CPU
    edits: Vec<Edit>,
    
    /// A configuration for this geometry.
    /// This is used to configure next evaluation on a svo, which will redivide the svo until individual voxels are smaller than this value.
    min_voxel_size: f32,
    
    aabb: AABB,
}

impl Geometry {
    pub const VOXEL_SIZE_RANGE: RangeInclusive<f32> = 0.005..=0.1;
    
    pub fn new(min_voxel_size: f32) -> Self {
        Self {
            edits:             vec![],
            svo:               None,
            evaluation_status: EvaluationStatus::NeedsEvaluation,
            min_voxel_size:    min_voxel_size.clamp(*Self::VOXEL_SIZE_RANGE.start(), *Self::VOXEL_SIZE_RANGE.end()),
            aabb:              AABB::ZERO,
        }
    }
    
    pub fn with_edits(mut self, edits: Vec<Edit>) -> Self {
        self.edits = edits;
        self.recompute_aabb();
        self
    }
    
    pub fn set_edits(&mut self, edits: Vec<Edit>) {
        self.edits = edits;
        self.recompute_aabb();
        self.evaluation_status = EvaluationStatus::NeedsEvaluation;
    }
    
    pub fn min_voxel_size(&self) -> f32 {
        self.min_voxel_size
    }
    
    pub fn set_min_voxel_size(&mut self, min_voxel_size: f32) {
        self.min_voxel_size = min_voxel_size.clamp(*Self::VOXEL_SIZE_RANGE.start(), *Self::VOXEL_SIZE_RANGE.end());
        self.evaluation_status = EvaluationStatus::NeedsEvaluation;
    }
    
    pub fn total_aabb(&self) -> &AABB {
        &self.aabb
    }
    
    pub fn edits(&self) -> &[Edit] {
        &self.edits
    }
    
    fn recompute_aabb(&mut self) {
        let mut edit_iter = self.edits.iter();
        let Some(first_edit) = edit_iter.next() else {
            self.aabb = AABB::ZERO;
            return;
        };
        
        let mut aabb = first_edit.aabb();
        edit_iter
            .filter(|e| e.operation == Operation::Add)
            .for_each(|e| aabb = aabb.add(&e.aabb()));
        
        self.aabb = aabb;
    }
    
}
