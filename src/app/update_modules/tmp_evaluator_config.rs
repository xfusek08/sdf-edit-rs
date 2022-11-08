///! This is updater module which checks if there is a geometry requesting an SVO evaluation
///! And sends the svo of the geometry for evaluation

use std::sync::Arc;

use crate::framework::gpu;
use crate::app::{
    application::ControlFlowResultAction,
    objects::cube::CubeOutlineComponent,
    updating::{InputUpdateResult, UpdateContext, UpdaterModule},
};

#[derive(Clone)]
pub struct TmpEvaluatorConfigProps {
    pub render_level: u32,
    pub min_voxel_size: f32,
}

pub struct VoxelSizeOutlineComponent;

#[derive(Default)]
pub struct TmpEvaluatorConfig {
    prev_props: Option<TmpEvaluatorConfigProps>,
}

impl TmpEvaluatorConfig {
    pub fn new(gpu: Arc<gpu::Context>) -> TmpEvaluatorConfig {
        Self { prev_props: None }
    }
}

impl<'a> UpdaterModule for TmpEvaluatorConfig {
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext) -> ControlFlowResultAction {
        let scene_props = context.state.scene.tmp_evaluator_config.clone();

        if let Some(TmpEvaluatorConfigProps {
            render_level,
            min_voxel_size,
        }) = self.prev_props {
            if scene_props.min_voxel_size != min_voxel_size {
                // Update all geometries to new voxel size
                context
                    .state
                    .scene
                    .geometry_pool
                    .iter_mut()
                    .for_each(|(_, geometry)| {
                        geometry.set_min_voxel_size(scene_props.min_voxel_size);
                    });
            }
        }
        
        // Update voxel size outline components to new voxel size
        for (_, (_, cube_component)) in context.state.scene.world.query::<(&VoxelSizeOutlineComponent, &mut CubeOutlineComponent)>().iter() {
            let half_length = scene_props.min_voxel_size * 0.5;
            cube_component.set_position(glam::Vec3::new(0.6 + half_length, half_length, 0.0));
            cube_component.set_size(scene_props.min_voxel_size);
        }
        
        self.prev_props = Some(scene_props);
        ControlFlowResultAction::None
    }

    fn input(&mut self, _: &mut UpdateContext) -> crate::app::updating::InputUpdateResult {
        InputUpdateResult::default()
    }

    fn resize(&mut self, _: &mut crate::app::updating::ResizeContext) -> ControlFlowResultAction {
        ControlFlowResultAction::None
    }

    fn after_render(&mut self, state: &mut crate::app::state::State) {}
}
