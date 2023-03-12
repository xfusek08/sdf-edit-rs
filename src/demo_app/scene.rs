use hecs::World;

use crate::{
    framework::camera::{
        SceneWithCamera,
        CameraRig
    },
    sdf::{
        geometry::GeometryPool,
        model::ModelPool
    },
};

use super::{
    svo_sdf_brick::DisplayOptions,
    tmp_evaluator_config::TmpEvaluatorConfigProps
};


#[derive(Debug, Default)]
pub struct SceneCounters {
    pub gui_updates: u64,
    pub renders: u64,
}

#[derive(Debug, Default)]
pub struct DisplayToggles {
    pub show_axes: bool,
    pub brick_display_options: DisplayOptions,
    pub show_wireframe: bool,
    pub show_voxel_size_reference: bool,
}

pub struct Scene {
    pub camera_rig: Box<dyn CameraRig>,
    pub geometry_pool: GeometryPool,
    pub model_pool: ModelPool,
    pub display_toggles: DisplayToggles,
    pub brick_level_break_size: f32,
    
    // tmp stuff
    pub world: World,
    pub counters: SceneCounters,
    pub tmp_evaluator_config: TmpEvaluatorConfigProps,
}

impl SceneWithCamera for Scene {
    fn get_camera_rig(&self) -> &dyn CameraRig {
        self.camera_rig.as_ref()
    }
    
    fn get_camera_mut(&mut self) -> &mut dyn CameraRig {
        self.camera_rig.as_mut()
    }
}
