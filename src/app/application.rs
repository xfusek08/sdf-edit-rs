
use std::sync::Arc;
use slotmap::SlotMap;
use winit_input_helper::WinitInputHelper;

use winit::{
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget},
    error::OsError,
    platform::run_return::EventLoopExtRunReturn,
    event::Event,
    dpi::PhysicalSize,
};

use super::{
    update_modules::{
        gui::GuiUpdater,
        camera::CameraUpdater,
        svo::SVOUpdater,
        tmp_evaluator_config::{
            TmpEvaluatorConfigProps,
            TmpEvaluatorConfig, VoxelSizeOutlineComponent
        }
    },
    render_modules::{
        lines::{LineMesh, LineRenderModule},
        svo_wireframe::SVOWireframeRenderModule,
        gui::GUIRenderModule,
        cube_outline::CubeOutlineRenderModule,
    },
    sdf::{
        geometry::{GeometryEdit, Geometry, GeometryPool},
        primitives::Primitive,
        model::{ModelID, Model}
    },
    state::{State, Scene},
    gpu::{GPUContext, vertices::ColorVertex},
    renderer::{Renderer, render_pass::RenderPassAttachment},
    updating::{Updater, UpdateContext, ResizeContext},
    camera::{Camera, CameraProperties},
    clock::Clock,
    gui::Gui,
    components::Deleted,
    math::Transform,
    objects::cube::CubeOutlineComponent,
};

use crate::error;

/// Create application state
fn init_state<T>(event_loop: &EventLoopWindowTarget<T>, window: &Window) -> State {
    // Create ECS world
    // ----------------
    //   - TODO: Add transform component to each entity in the world
    let mut world = hecs::World::new();

    // Simple Drawing of coordinate axes
    // NOTE: This is a temporary line rendering system which will be changed, see file `src/app/render_modules/lines.rs` for more info.
    world.spawn((
        LineMesh {
            is_dirty: true,
            vertices: LINE_VERTICES,
        },
        Deleted(false),
    ));
    
    // create and register test geometry
    let min_voxel_size = 0.5;
    let mut geometry_pool: GeometryPool = SlotMap::with_key();
    let test_geometry = Geometry::new(min_voxel_size)
        .with_edits(vec![
            GeometryEdit {
                primitive: Primitive::Sphere {
                    center: glam::Vec3::ZERO,
                    radius: 1.0
                },
                operation: super::sdf::geometry::GeometryOperation::Add,
                transform: Transform::default(),
                blending: 0.0,
            }
        ]);
    
    let test_geometry_id = geometry_pool.insert(test_geometry);
    
    // create and register test model
    let mut model_pool: SlotMap<ModelID, Model> = SlotMap::with_key();
    let test_model = Model::new(test_geometry_id);
    model_pool.insert(test_model);
    
    // Show voxel size instance
    world.spawn((VoxelSizeOutlineComponent, CubeOutlineComponent::new(1.5, 0.0, 0.0, min_voxel_size)));
    
    State {
        gui: Gui::new(&event_loop),
        scene: Scene {
            camera: Camera::new(CameraProperties {
                aspect_ratio: window.inner_size().width as f32 / window.inner_size().height as f32,
                fov: 10.0,
                ..Default::default()
            }).orbit(glam::Vec3::ZERO, 10.0),
            geometry_pool,
            model_pool,
            world,
            counters: Default::default(),
            tmp_evaluator_config: TmpEvaluatorConfigProps {
                render_svo_level_begin: 0,
                render_svo_level_end: 10,
                min_voxel_size,
            }
        },
    }
}

/// Defines "dynamic" structure of renderer, imagine as simple render graph.
fn init_renderer(gpu: Arc<GPUContext>, window: &Window) -> Renderer {
    let mut renderer = Renderer::new(gpu, window);
    
    // load modules
    let line_module = renderer.add_module(|c| LineRenderModule::new(c));
    let cube_outline = renderer.add_module(|c| CubeOutlineRenderModule::new(c));
    let svo_module = renderer.add_module(|c| SVOWireframeRenderModule::new(c));
    let gui_module = renderer.add_module(|c| GUIRenderModule::new(c));
    
    // passes are executed in order of their registration
    renderer.set_render_pass(|c| RenderPassAttachment::base(c), &[line_module, cube_outline, svo_module]);
    renderer.set_render_pass(|c| RenderPassAttachment::gui(c), &[gui_module]);
    
    renderer
}

#[derive(Debug, Clone)]
pub enum ControlFlowResultAction {
    None, Redraw, Exit
}
impl ControlFlowResultAction {
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (ControlFlowResultAction::Exit, _) => ControlFlowResultAction::Exit,
            (_, ControlFlowResultAction::Exit) => ControlFlowResultAction::Exit,
            (ControlFlowResultAction::Redraw, _) => ControlFlowResultAction::Redraw,
            (_, ControlFlowResultAction::Redraw) => ControlFlowResultAction::Redraw,
            _ => ControlFlowResultAction::None,
        }
    }
}

#[derive(Default)]
pub struct ApplicationConfig;

pub async fn run(config: ApplicationConfig) {
    
    let mut event_loop = EventLoop::new();
    let window = create_window(&event_loop, config).unwrap();
    let gpu = Arc::new(GPUContext::new(&window).await);
    
    // Updating system
    let mut updater = Updater::new()
        .with_module(GuiUpdater)
        .with_module(TmpEvaluatorConfig::default())
        .with_module(CameraUpdater)
        .with_module(SVOUpdater::new(gpu.clone())); // SVO updater needs arc reference to GPU context because it spawns threads sharing the GPU context
    
    // Rendering system
    let mut renderer = init_renderer(gpu.clone(), &window);
    
    // Application state
    let mut state = init_state(&event_loop, &window); // contains all that is to be rendered and can be updated
    
    // Execution control
    let mut input = WinitInputHelper::new(); // Helps with translating window events to remembered input state
    let mut clock = Clock::now(30); // 30 ticks per second
    
    // Main loop
    let mut event_consumed_by_gui = false;
    
    event_loop.run_return(move |event, _, control_flow| {
        profiler::scope!("Event incoming");
        
        // Proces window events
        let mut flow_result_action = ControlFlowResultAction::None;
        
        // Resize subroutine
        let resize = &mut |size: &PhysicalSize<u32>, scale_factor: f64, state: &mut State| {
            renderer.resize(size, scale_factor);
            updater.resize(ResizeContext { size, scale_factor, state })
        };
        
        match event {
            Event::NewEvents(_) |
            Event::MainEventsCleared |
            Event::WindowEvent { .. } => {
                profiler::scope!("Processing input event");
                
                // Let gui process window event and when it does not handle it, update scene
                if let Event::WindowEvent { event, .. } = &event {
                    profiler::scope!("Processing input event by GUI");
                    event_consumed_by_gui = state.gui.on_event(&event);
                }
                
                // Let input helper process event to somewhat coherent input state and work with that.
                //   (input.update(..) returns true only on Event::MainEventsCleared hence `update_scene` variable)
                if input.update(&event) {
                    flow_result_action = flow_result_action.combine(
                        if let Some(size) = input.window_resized() {
                            resize(&size, input.scale_factor().unwrap_or(1.0), &mut state)
                        } else if let Some(scale_factor) = input.scale_factor_changed() {
                            resize(&window.inner_size(), scale_factor, &mut state)
                        } else if input.quit() {
                            ControlFlowResultAction::Exit
                        } else if !event_consumed_by_gui {
                            updater.input(UpdateContext {
                                state:  &mut state,
                                input:  &input,
                                tick:   clock.current_tick(),
                                window: &window,
                            })
                        } else {
                            ControlFlowResultAction::None
                        }
                    );
                }
                
            },
            
            // Render frame when windows requests a redraw not on every update
            // This is because application could only redraw when there are changes saving CPU time and power.
            Event::RedrawRequested(_) => {
                profiler::scope!("Processing redraw request");
                
                renderer.prepare(&state);
                renderer.render();
                renderer.finalize();
                
                // this could be run in parallel with render and finalize
                updater.after_render(&mut state);
                
                // ! TMP stuff
                state.scene.counters.renders += 1;
                
                for (_, mesh) in state.scene.world.query_mut::<&mut LineMesh>() {
                    mesh.is_dirty = false;
                }
            },
            _ => {} // Ignore other events
        }
        
        // Tick clock and update on tick if app is still running
        if clock.tick() {
            // It is time to tick the application
            updater.update(UpdateContext {
                state:  &mut state,
                input:  &input,
                tick:   clock.current_tick(),
                window: &window,
            });
            
            // Render updated state
            // TODO: Do not redraw when window is not visible
            flow_result_action = ControlFlowResultAction::Redraw;
        } else {
            // Schedule next tick as a time to wake up in case of idling
            *control_flow = ControlFlow::WaitUntil(clock.next_scheduled_tick().clone())
        };
        
        // Decide on final control flow based on combination of all result actions
        match flow_result_action {
            ControlFlowResultAction::Exit => *control_flow = ControlFlow::Exit,
            ControlFlowResultAction::Redraw => window.request_redraw(),
            _ => {},
        }
    });
}

// Helper functions

#[profiler::function]
fn create_window<T>(event_loop: &EventLoop<T>, config: ApplicationConfig) -> Result<Window, OsError> {
    WindowBuilder::new()
        .with_title("Rust Game")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
}

/// TODO: Obsolete delete after line renderer is refactored
#[profiler::function]
pub fn remove_deleted_entities(state: &mut State) {
    
    // fill buffer with entities to delete
    let mut entities_to_delete = Vec::with_capacity(state.scene.world.len() as usize);
    for (entity, (Deleted(deleted),)) in state.scene.world.query::<(&Deleted,)>().iter() {
        if *deleted {
            entities_to_delete.push(entity);
        }
    }
    
    // delete entities
    for entity in entities_to_delete {
        if let Err(_) = state.scene.world.despawn(entity) {
            error!("Failed to despawn entity {:?}", entity);
        }
    }
    
}

// Temporary axis vertex data

const LINE_VERTICES: &[ColorVertex] = &[
    ColorVertex { position: glam::Vec3::new(-2.0, 0.0, 0.0), color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(2.0, 0.0, 0.0),  color: glam::Vec3::new(2.0, 0.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, -2.0, 0.0), color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 2.0, 0.0),  color: glam::Vec3::new(0.0, 2.0, 0.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, -2.0), color: glam::Vec3::new(0.0, 0.0, 2.0) },
    ColorVertex { position: glam::Vec3::new(0.0, 0.0, 2.0),  color: glam::Vec3::new(0.0, 0.0, 2.0) },
];