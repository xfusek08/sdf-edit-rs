use slotmap::SlotMap;

use crate::{
    framework::{
        application::Context,
        math::Transform,
        updater::Updater,
        renderer::{
            Renderer,
            RenderPassAttachment
        },
        camera::{
            Camera,
            CameraProperties,
            CameraUpdater
        },
        gui::{
            GuiRenderModule,
            GuiUpdateModule
        },
        gpu::vertices::ColorVertex,
    },
    sdf::{
        primitives::Primitive,
        model::{
            ModelPool,
            Model,
        },
        geometry::{
            GeometryPool,
            Geometry,
            GeometryEdit,
            GeometryOperation,
        },
    }
};

use super::{
    scene::Scene,
    components::Deleted,
    modules::{
        svo_evaluator::SvoEvaluatorUpdater,
        svo_wireframe::SvoWireframeRenderModule,
        svo_sdf_brick::SvoSdfBricksRenderModule,
        line::{
            LineRenderModule,
            LineMesh,
        },
        cube::{
            CubeOutlineRenderModule,
            CubeOutlineComponent,
        },
        tmp_evaluator_config::{
            VoxelSizeOutlineComponent,
            TmpEvaluatorConfigProps,
            TmpEvaluatorConfig,
        },
    },
};

pub fn define_renderer(context: &Context) -> Renderer<Scene> {
    let mut renderer = Renderer::new(context.gpu.clone(), context.window);
    
    // load modules
    let line_module = renderer.add_module(LineRenderModule::new);
    let cube_outline = renderer.add_module(CubeOutlineRenderModule::new);
    let svo_wireframe_module = renderer.add_module(SvoWireframeRenderModule::new);
    // let svo_brick_module = renderer.add_module(|c| SvoSolidBricksRenderModule::new(c));
    let svo_sdf_brick_module = renderer.add_module(SvoSdfBricksRenderModule::new);
    let gui_module = renderer.add_module(GuiRenderModule::new);
    
    // passes are executed in order of their registration
    renderer.set_render_pass(RenderPassAttachment::base, &[
        line_module,
        cube_outline,
        svo_sdf_brick_module,
        svo_wireframe_module
    ]);
    
    renderer.set_render_pass(RenderPassAttachment::gui, &[gui_module]);
    
    renderer
}

pub fn define_updater(context: &Context) -> Updater<Scene> {
    Updater::new()
        .with_module(GuiUpdateModule::new(draw_gui))
        .with_module(TmpEvaluatorConfig::default())
        .with_module(CameraUpdater)
        .with_module(SvoEvaluatorUpdater::new(context.gpu.clone())) // SVO updater needs arc reference to GPU context because it spawns threads sharing the GPU context
}

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
    
    // Show voxel size instance
    world.spawn((VoxelSizeOutlineComponent, CubeOutlineComponent::new(1.5, 0.0, 0.0, min_voxel_size)));
    
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
        }
    }
}

pub fn style_gui(mut style: egui::Style) -> egui::Style {
    // adjust intrusive window shadowing
    style.visuals.window_shadow = egui::epaint::Shadow {
        extrusion: 0.0,
        color: egui::Color32::BLACK,
    };
    style
}

pub fn draw_gui(ctx: &egui::Context, scene: &mut Scene) {
    scene.counters.gui_updates += 1;
    
    egui::Window::new("Apps")
        .default_pos((10.0, 10.0))
        .show(ctx, |ui| {
            
            egui::Grid::new("grid_1")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("gui updates:");
                    ui.label(format!("{}", scene.counters.gui_updates));
                    ui.end_row();
                    ui.label("renders:");
                    ui.label(format!("{}", scene.counters.renders));
                    ui.end_row();
                    ui.label("Min voxel Size:");
                    ui.add(
                        egui::Slider::new(&mut scene.tmp_evaluator_config.min_voxel_size, Geometry::VOXEL_SIZE_RANGE)
                            .step_by(0.001)
                            .clamp_to_range(true)
                    );
                    
                });
            
            ui.separator();
            
            egui::Grid::new("grid_2")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Camera fov:");
                    ui.add(
                        egui::Slider::new(&mut scene.camera.fov, 10.0..=150.0)
                            // .step_by(step as f64)
                            .clamp_to_range(true)
                    );
                    ui.end_row();
            });
            
            ui.separator();
            
            let mut render_level = scene.tmp_evaluator_config.render_level;
            
            ui.label("TMP SVO Stats");
            for (geometry_id, geometry) in scene.geometry_pool.iter() {
                let id = format!("{:?}", geometry_id);
                ui.label(format!("Geometry: {}", id));
                ui.label("SVO:");
                if let Some(svo) = geometry.svo.as_ref() {
                    egui::Grid::new(&id).num_columns(2).show(ui, |ui| {
                        ui.label("Render Svo Level");
                        
                        let levels = svo.levels.len() as u32;
                        if levels > 0 {
                            ui.add(egui::Slider::new(
                                &mut render_level,
                                0..=levels - 1,
                            ).step_by(1.0).clamp_to_range(true));
                        }
                        ui.label(format!("/ {}", levels - 1));
                        
                        ui.end_row();
                        ui.label("Svo Node Count:");
                        ui.label(format!("{:?}", svo.node_pool.count()));
                        ui.end_row();
                        ui.label("Svo Level Count:");
                        ui.label(format!("{}", svo.levels.len()));
                        ui.end_row();
                        ui.label("Svo Capacity:");
                        ui.label(format!("{:?}", svo.node_pool.capacity()));
                    });
                } else {
                    ui.label("None");
                }
                
                scene.tmp_evaluator_config.render_level = render_level;
                
                ui.spacing();
            }
        });
}

const LINE_VERTICES: &[ColorVertex] = &[
    ColorVertex { position: glam::Vec3::new(-2.0, 0.0, 0.0), color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(2.0, 0.0, 0.0),  color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, -2.0, 0.0), color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 2.0, 0.0),  color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, -2.0), color: glam::Vec3::new(0.0, 0.0, 2.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, 2.0),  color: glam::Vec3::new(0.0, 0.0, 2.0) },
];
