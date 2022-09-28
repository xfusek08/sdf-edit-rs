
use hecs::World;
use slotmap::SlotMap;

use super::{
    camera::Camera,
    gui::Gui,
    sdf::{
        model::{Model, ModelID},
        geometry::GeometryPool
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
    pub model_pool: SlotMap<ModelID, Model>,
    
    // tmp stuff
    pub world: World,
    pub counters: SceneCounters,
}

pub struct State {
    pub gui: Gui,
    pub scene: Scene,
}
