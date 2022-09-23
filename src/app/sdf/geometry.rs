use crate::app::transform::Transform;

use super::{svo::SVOctree, primitives::Primitive};

pub enum GeometryOperation {
    Add,
    Subtract,
    Intersect,
    // TODO: Paint
}

pub struct GeometryEdit {
    pub primitive: Primitive,
    pub operation: GeometryOperation,
    pub transform: Transform,
    pub blending:  f32,
}

pub struct GeometryEditList {
    pub edits: Vec<GeometryEdit>,
}

pub enum GeometryEvaluationStatus {
    NeedsEvaluation,
    Evaluating,
    Evaluated,
}

pub struct Geometry {
    
    /// A Sparse Voxel Octree evaluated into GPU memory
    pub svo: Option<SVOctree>,
    
    /// A list of edits that compose this geometry
    pub edits: GeometryEditList,
    
    
    /// The status of the geometry evaluation used by evaluator
    ///   - `NeedsEvaluation` means that the geometry has been edited and needs to be evaluated
    ///      evaluator on next update collects all geometries with this status and spawns and evaluation job.
    ///   - `Evaluating` means that the geometry is currently being evaluated by evaluator.
    ///   - `Evaluated` means that the geometry does not need to be evaluated.
    pub evaluation_status: GeometryEvaluationStatus,
    
}
