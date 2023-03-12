
use slotmap::SlotMap;
use rand::Rng;

use crate::{
    shape_builder::Shape,
    framework::{
        gpu::vertices::ColorVertex,
        application::Context,
        math::Transform,
        camera::{OrbitCameraRig, Camera},
    },
    sdf::{
        model::{ModelPool, Model},
        geometry::{GeometryPool, Geometry},
    },
};

use super::{
    scene::Scene,
    components::{
        AxisMesh,
        Active
    },
    line::LineMesh,
    geometries::{bumpy_sphere, test_geometry}, tmp_evaluator_config::TmpEvaluatorConfigProps,
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
    
    let min_voxel_size = 0.016;
    let mut geometry_pool: GeometryPool = SlotMap::with_key();
    let g1 = Geometry::new(min_voxel_size).with_edits(bumpy_sphere().build());
    let g2 = Geometry::new(min_voxel_size).with_edits(test_geometry().build());
    
    let g1_id = geometry_pool.insert(g1);
    let g2_id = geometry_pool.insert(g2);
    
    // Create and register test model
    // ------------------------------
    
    let mut model_pool = ModelPool::new();
    // model_pool.insert(Model::new(g1_id));
    
    // model_pool.insert(Model::new(g2_id)
    //     .with_transform(Transform::IDENTITY.translate((3.0, 0.0, 0.0).into())));
    
    // for i in -5..=5 {
    //     for j in -5..=5 {
    //         model_pool.insert(
    //             Model::new(test_geometry_id).with_transform(
    //                 Transform::IDENTITY
    //                     .translate((
    //                         i as f32 * 1.5,
    //                         j as f32 * 1.5,
    //                         0.0,
    //                     ).into())
    //                     .scale(glam::Vec3::splat(0.25))
    //             )
    //         );
    //     }
    // }
    
    let mut rng = rand::thread_rng();
    for _ in 0..=4000 {
        model_pool.insert(
            Model::new([g1_id, g2_id][rng.gen_range(0..=1)]).with_transform(
                Transform::IDENTITY
                    .translate((
                        rng.gen_range(-500.0..=500.0),
                        rng.gen_range(-500.0..=500.0),
                        rng.gen_range(-500.0..=500.0),
                    ).into())
                    .scale(glam::Vec3::splat(rng.gen_range(0.21..=20.0)))
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
        camera_rig: Box::new(OrbitCameraRig::from_camera(
            Camera {
                fov:          60.0,
                far:          10000.0,
                aspect_ratio: context.window.inner_size().width as f32 / context.window.inner_size().height as f32,
                position:     glam::vec3(5.0, 5.0, 5.0),
                ..Default::default()
            },
            glam::Vec3::ZERO,
            5.0,
        )),
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
