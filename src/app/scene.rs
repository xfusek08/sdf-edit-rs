
use glam::Vec3;
use hecs::World;
use slotmap::SlotMap;
use winit::window::Window;

use super::{
    camera::{Camera, CameraProperties},
    render_modules::lines::LineMesh,
    components::Deleted,
    model::AXIS_VERTICES,
    transform::Transform,
    sdf::{
        model::{Model, ModelID},
        geometry::{Geometry, GeometryEdit, GeometryID, GeometryPool},
        primitives::Primitive
    },
};

#[derive(Default)]
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

impl Scene {
    
    #[profiler::function]
    pub fn new(window: &Window) -> Scene {
        
        // Create camera, which is sort of unique object outside of ECS world
        let camera = Camera::new(CameraProperties {
            aspect_ratio: window.inner_size().width as f32 / window.inner_size().height as f32,
            ..Default::default()
        }).orbit(
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
        
        // create and register test geometry
        let mut geometry_pool: GeometryPool = SlotMap::with_key();
        let test_geometry = Geometry::new().with_edits(vec![
            GeometryEdit {
                primitive: Primitive::Sphere {
                    center: Vec3::ZERO,
                    radius: 1.0
                },
                operation: super::sdf::geometry::GeometryOperation::Add,
                transform: Transform::default(),
                blending: 0.0,
            }
        ]);
        let test_geometry_id = geometry_pool.insert(test_geometry);
        
        // create and register tes model
        let mut model_pool: SlotMap<ModelID, Model> = SlotMap::with_key();
        let test_model = Model::new(test_geometry_id);
        model_pool.insert(test_model);
        
        Self {
            geometry_pool,
            model_pool,
            camera,
            world,
            counters: Default::default(),
        }
    }
}
