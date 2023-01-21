use slotmap::SlotMap;

use crate::{
    framework::{
        gpu::vertices::ColorVertex,
        application::Context,
        math::Transform,
        camera::{Camera, CameraProperties},
    },
    sdf::{
        model::{ModelPool, Model},
        geometry::{GeometryPool, Geometry},
    }, shape_builder::Shape,
};

use super::{
    scene::Scene,
    components::{AxisMesh, Active},
    modules::{line::LineMesh, TmpEvaluatorConfigProps}, bumpy_sphere::bumpy_sphere,
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
        AxisMesh,
        LineMesh {
            is_dirty: true,
            vertices: LINE_VERTICES,
        },
        Active(false),
    ));
    
    // Create and register test geometry
    // ---------------------------------
    
    let min_voxel_size = 0.01;
    let mut geometry_pool: GeometryPool = SlotMap::with_key();
        
    // lets generate a geometry using the shape builder
    let test_geometry = Geometry::new(min_voxel_size).with_edits(
        Shape::empty().add(
            bumpy_sphere(),
            Transform::IDENTITY,
            0.0
        ).build()
    );
    
    let test_geometry_id = geometry_pool.insert(test_geometry);
    
    // Create and register test model
    // ------------------------------
    
    let mut model_pool: ModelPool = SlotMap::with_key();
    let test_model = Model::new(test_geometry_id);
    model_pool.insert(test_model);
    
    Scene {
        camera: Camera::new(CameraProperties {
            aspect_ratio: context.window.inner_size().width as f32 / context.window.inner_size().height as f32,
            fov: 60.0,
            ..Default::default()
        }).orbit(glam::Vec3::ZERO, 4.0),
        geometry_pool,
        model_pool,
        world,
        counters: Default::default(),
        tmp_evaluator_config: TmpEvaluatorConfigProps {
            render_level: 0,
            min_voxel_size,
        },
        display_toggles: Default::default(),
        brick_level_break_size: 0.1,
    }
}
