use hecs::World;

use crate::{
    framework::camera::{
        Camera,
        SceneWithCamera
    },
    sdf::{
        geometry::GeometryPool,
        model::ModelPool
    },
};

use super::modules::svo_sdf_brick;

#[derive(Debug, Default)]
pub struct SceneCounters {
    pub gui_updates: u64,
    pub renders: u64,
}

#[derive(Debug, Default)]
pub struct DisplayToggles {
    pub show_axes: bool,
    pub brick_display_options: svo_sdf_brick::DisplayOptions,
    pub show_wireframe: bool,
    pub show_voxel_size_reference: bool,
}

pub struct Scene {
    pub camera: Camera,
    pub geometry_pool: GeometryPool,
    pub model_pool: ModelPool,
    pub display_toggles: DisplayToggles,
    pub brick_level_break_size: f32,
    
    // tmp stuff
    pub world: World,
    pub counters: SceneCounters,
    pub tmp_evaluator_config: super::modules::TmpEvaluatorConfigProps,
}

impl SceneWithCamera for Scene {
    fn get_camera(&self) -> &crate::framework::camera::Camera {
        &self.camera
    }
    
    fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
