///! This is updater module which checks if there is a geometry requesting an SVO evaluation
///! And sends the svo of the geometry for evaluation

use std::sync::Arc;

use crate::{
    demo_app::scene::Scene,
    framework::{
        gpu,
        updater::{
            UpdaterModule,
            UpdateContext,
            UpdateResultAction,
            InputUpdateResult,
            ResizeContext,
            AfterRenderContext
        }
    },
};

use super::cube::CubeOutlineComponent;

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

impl UpdaterModule<Scene> for TmpEvaluatorConfig {
    #[profiler::function]
    fn update(&mut self, context: &mut UpdateContext<Scene>) -> UpdateResultAction {
        let scene_props = context.scene.tmp_evaluator_config.clone();

        if let Some(TmpEvaluatorConfigProps {
            render_level,
            min_voxel_size,
        }) = self.prev_props {
            if scene_props.min_voxel_size != min_voxel_size {
                // Update all geometries to new voxel size
                context
                    .scene
                    .geometry_pool
                    .iter_mut()
                    .for_each(|(_, geometry)| {
                        geometry.set_min_voxel_size(scene_props.min_voxel_size);
                    });
            }
        }
        
        // Update voxel size outline components to new voxel size
        for (_, (_, cube_component)) in context.scene.world.query::<(&VoxelSizeOutlineComponent, &mut CubeOutlineComponent)>().iter() {
            let half_length = scene_props.min_voxel_size * 0.5;
            cube_component.set_position(glam::Vec3::new(0.6 + half_length, half_length, 0.0));
            cube_component.set_size(scene_props.min_voxel_size);
        }
        
        self.prev_props = Some(scene_props);
        UpdateResultAction::None
    }

    fn input(&mut self, _: &mut UpdateContext<Scene>) -> InputUpdateResult {
        InputUpdateResult::default()
    }

    fn resize(&mut self, context: &mut ResizeContext<Scene>) -> UpdateResultAction {
        UpdateResultAction::None
    }

    fn after_render(&mut self, state: &mut AfterRenderContext<Scene>) {}
}
