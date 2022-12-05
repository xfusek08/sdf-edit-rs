use slotmap::SlotMap;

use crate::{
    framework::{
        gpu::vertices::ColorVertex,
        application::Context,
        math::Transform,
        camera::{Camera, CameraProperties},
    },
    sdf::{
        primitives::Primitive,
        model::{ModelPool, Model},
        geometry::{GeometryPool, Geometry, GeometryEdit, GeometryOperation},
    },
};

use super::{
    scene::Scene,
    modules::{line::LineMesh, tmp_evaluator_config::TmpEvaluatorConfigProps},
    components::Deleted,
};


const LINE_VERTICES: &[ColorVertex] = &[
    ColorVertex { position: glam::Vec3::new(-2.0, 0.0, 0.0), color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(2.0, 0.0, 0.0),  color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, -2.0, 0.0), color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 2.0, 0.0),  color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, -2.0), color: glam::Vec3::new(0.0, 0.0, 2.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, 2.0),  color: glam::Vec3::new(0.0, 0.0, 2.0) },
];

pub fn init_scene(context: &Context) -> Scene {
    // Create ECS world
    // ----------------
    //   - TODO: Add transform component to each entity in the world
    
    let mut world = hecs::World::new();

    // Simple Drawing of coordinate axes
    // ---------------------------------
    
    world.spawn((
        LineMesh {
            is_dirty: true,
            vertices: LINE_VERTICES,
        },
        Deleted(false),
    ));
    
    // Create and register test geometry
    // ---------------------------------
    
    let min_voxel_size = 0.01;
    let mut geometry_pool: GeometryPool = SlotMap::with_key();
    let test_geometry = Geometry::new(min_voxel_size)
        .with_edits(vec![
            GeometryEdit {
                primitive: Primitive::Sphere {
                    center: glam::Vec3::ZERO,
                    radius: 1.0
                },
                operation: GeometryOperation::Add,
                transform: Transform::default(),
                blending: 0.0,
            }
        ]);
    
    let test_geometry_id = geometry_pool.insert(test_geometry);
    
    // Create and register test model
    // ------------------------------
    
    let mut model_pool: ModelPool = SlotMap::with_key();
    let test_model = Model::new(test_geometry_id);
    model_pool.insert(test_model);
    
    Scene {
        camera: Camera::new(CameraProperties {
            aspect_ratio: context.window.inner_size().width as f32 / context.window.inner_size().height as f32,
            fov: 10.0,
            ..Default::default()
        }).orbit(glam::Vec3::ZERO, 10.0),
        geometry_pool,
        model_pool,
        world,
        counters: Default::default(),
        tmp_evaluator_config: TmpEvaluatorConfigProps {
            render_level: 0,
            min_voxel_size,
        },
        display_toggles: Default::default(),
    }
}
