
use slotmap::SlotMap;
use rand::Rng;

use crate::{
    framework::{
        gpu::vertices::ColorVertex,
        application::Context,
        math::Transform,
        camera::{
            OrbitCameraRig,
            Camera,
            CameraRig
        },
    },
    sdf::geometry::{
        GeometryPool,
        Geometry,
    },
};

use super::{
    scene::Scene,
    line::LineMesh,
    tmp_evaluator_config::TmpEvaluatorConfigProps,
    components::{
        AxisMesh,
        Active,
    },
    geometries::{
        bumpy_sphere,
        test_geometry,
        mouse,
    },
    continuous_rotation::ContinuousRotation,
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
    
    let g1_id = geometry_pool.insert(
        Geometry::new(min_voxel_size).with_edits(bumpy_sphere().build())
    );
    
    // Create and register test model
    // ------------------------------
    
    #[cfg(feature = "lod_test")]
    {
        let g2_id = geometry_pool.insert(
            Geometry::new(min_voxel_size).with_edits(test_geometry().build())
        );
        
        let g3_id = geometry_pool.insert(
            Geometry::new(min_voxel_size).with_edits(mouse().build())
        );
        
        let mut rng = rand::thread_rng();
        for i in -50..=50 {
            for j in -50..=50 {
                world.spawn((
                    [g1_id, g2_id, g3_id][rng.gen_range(0..=2)],
                    Transform::IDENTITY
                        .translate((
                            (i * 3) as f32 + rng.gen_range(-0.3..=0.3),
                            0.0,
                            (j * 3) as f32 + rng.gen_range(-0.3..=0.3)
                        ).into())
                        .scale(glam::Vec3::splat(rng.gen_range(0.5..=1.5)))
                        .rotate(glam::Quat::from_euler(
                            glam::EulerRot::XYZ,
                            rng.gen_range(-20.0..=20.0 as f32).to_radians(),
                            rng.gen_range(-20.0..=20.0 as f32).to_radians(),
                            rng.gen_range(-20.0..=20.0 as f32).to_radians()
                        )),
                    // ContinuousRotation::random()
                ));
            }
        }
    }
    
    #[cfg(not(feature = "lod_test"))]
    world.spawn((
        g1_id,
        Transform::IDENTITY,
        // ContinuousRotation::random(),
    ));
    
    Scene {
        camera_rig: CameraRig::Orbit(OrbitCameraRig::from_camera(
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
