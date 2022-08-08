use bevy_ecs::prelude::Component;
use super::vertex::Vertex;

#[derive(Component)]
pub struct Mesh {
    pub vertices: &'static [Vertex],
    pub indices: &'static [u16],
}

#[derive(Component)]
pub struct Texture {
    pub texture: image::DynamicImage,
}
