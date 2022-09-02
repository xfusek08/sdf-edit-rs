use glam::Vec3;
use hecs::World;

use crate::app::{
    model::AXIS_VERTICES,
    rendering::render_modules::line_render_module::{LineMesh, LineMeshChangedFlag}
};

use super::Camera;

pub struct Scene {
    pub camera: Camera,
    pub world: World,
}

impl Scene {
    pub fn new() -> Scene {
        
        // Create camera, which is sort of unique object outside of ECS world
        let camera = Camera::new();
        
        // Create ECS world
        // ----------------
        //   - TODO: Add transform component to each entity in the world
        let mut world = World::new();
    
        // World coordinate axis
        world.spawn((
            LineMesh { vertices: AXIS_VERTICES },
            LineMeshChangedFlag(false),
        ));
        
        Self { camera, world }
    }
}
