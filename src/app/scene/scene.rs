use glam::Vec3;
use hecs::World;

use crate::app::{
    model::AXIS_VERTICES,
    rendering::modules::line_renderer::LineMesh,
};

use super::{Camera, components::Deleted};

pub struct Scene {
    pub camera: Camera,
    pub world: World,
}

impl Scene {
    pub fn new() -> Scene {
        
        // Create camera, which is sort of unique object outside of ECS world
        let camera = Camera::new().orbit(
            Vec3::new(0.0, 0.0, 0.0),
            1.0
        );
        
        // Create ECS world
        // ----------------
        //   - TODO: Add transform component to each entity in the world
        let mut world = World::new();
    
        // World coordinate axis
        world.spawn((
            LineMesh { is_dirty: true, vertices: AXIS_VERTICES },
            Deleted(false),
        ));
        
        Self {
            camera,
            world,
        }
    }
}
