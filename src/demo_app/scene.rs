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

#[derive(Debug, Default)]
pub struct SceneCounters {
    pub gui_updates: u64,
    pub renders: u64,
}

pub struct Scene {
    pub camera: Camera,
    pub geometry_pool: GeometryPool,
    pub model_pool: ModelPool,
    
    // tmp stuff
    pub world: World,
    pub counters: SceneCounters,
    pub tmp_evaluator_config: super::modules::tmp_evaluator_config::TmpEvaluatorConfigProps,
}

impl SceneWithCamera for Scene {
    fn get_camera(&self) -> &crate::framework::camera::Camera {
        &self.camera
    }
    
    fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
