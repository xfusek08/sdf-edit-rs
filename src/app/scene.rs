
use bevy_ecs::{world::{World, Mut}, prelude::Entity};
use glam::Vec3;

use super::{camera::Camera, components::{Mesh, Texture}, model::{PENTAGON_VERTICES, PENTAGON_INDICES}};

pub struct Scene {
    pub world: World,
    pub camera_entity: Entity,
}

impl Scene {
    pub fn new() -> Self {
        let mut world = World::default();
        
        // add camera to scene
        let camera_entity = world.spawn().insert(Camera::new().orbit(Vec3::ZERO, 10.0)).id();
        
        // add model to scene
        world.spawn()
            .insert(Mesh { vertices: PENTAGON_VERTICES, indices: PENTAGON_INDICES })
            .insert(Texture {
                texture: image::load_from_memory(
                    include_bytes!("../../resources/textures/happy-tree.png")
                ).expect("Failed fo load texture image.")
            });
            
        Self { world, camera_entity }
    }
    
    pub fn get_camera_mut(&mut self) -> Mut<Camera> {
        self.world.get_mut::<Camera>(self.camera_entity).unwrap()
    }
}
