
use slotmap::SlotMap;
use rand::Rng;

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
    },
    shape_builder::Shape,
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
    
    let mut model_pool = ModelPool::new();
    // model_pool.insert(
    //     Model::new(test_geometry_id).with_transform(Transform::IDENTITY
    //         .translate((1.0, 0.0, 0.0).into())
    //         // .scale(glam::Vec3::splat(2.5))
    //         // .rotate(glam::Quat::from_rotation_x((45 as f32).to_radians()))
    //     )
    // );
    
    // model_pool.insert(
    //     Model::new(test_geometry_id).with_transform(Transform::IDENTITY
    //         // .translate((5.0, 0.0, 0.0).into())
    //         // .scale(glam::Vec3::splat(3.5))
    //         // .rotate(glam::Quat::from_rotation_x((45 as f32).to_radians()))
    //     )
    // );
    
    // model_pool.insert(
    //     Model::new(test_geometry_id)
    //         .with_transform(
    //             Transform::IDENTITY
    //                 .translate((3.0, 0.0, 0.0).into())
    //         )
    // );
    
    
    let mut rng = rand::thread_rng();
    for z in 0..=500 {
        model_pool.insert(
            Model::new(test_geometry_id).with_transform(
                Transform::IDENTITY
                    .translate((
                        rng.gen_range(-40.0..=40.0),
                        rng.gen_range(-40.0..=40.0),
                        rng.gen_range(-40.0..=40.0),
                    ).into())
                    .scale(glam::Vec3::splat(rng.gen_range(0.21..=5.0)))
                    .rotate(glam::Quat::from_euler(
                        glam::EulerRot::XYZ,
                        rng.gen_range(0.0..=360.0 as f32).to_radians(),
                        rng.gen_range(0.0..=360.0 as f32).to_radians(),
                        rng.gen_range(0.0..=360.0 as f32).to_radians()
                    ))
            )
        );
    }
    
    Scene {
        camera: Camera::new(CameraProperties {
            aspect_ratio: context.window.inner_size().width as f32 / context.window.inner_size().height as f32,
            fov: 60.0,
            far: 300000.0,
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
        brick_level_break_size: 0.03,
    }
}
