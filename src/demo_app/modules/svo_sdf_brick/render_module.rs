
use std::collections::HashMap;

use super::{
    SvoSDFBrickPipeline,
    SvoBrickSelectPipeline,
    BrickInstances,
    GPUGeometryTransforms,
};

use crate::{
    error,
    demo_app::scene::Scene,
    sdf::geometry::GeometryID,
    framework::{
        math::{Transform, Frustum},
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPass,
        }, gui::Gui,
    }, info,
};

///! This is main renderer of evaluated geometries

#[derive(Debug)]
pub struct SvoSdfBricksRenderModule {
    pipeline: SvoSDFBrickPipeline,
    brick_select_compute_pipeline: SvoBrickSelectPipeline,
    brick_instances: BrickInstances,
    instance_transforms: HashMap<GeometryID, GPUGeometryTransforms>,
}

impl SvoSdfBricksRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        counters::register!("brick_selected_counter");
        counters::register!("object_selected_counter");
        counters::register!("object_instance_counter");
        Self {
            pipeline: SvoSDFBrickPipeline::new(context),
            brick_select_compute_pipeline: SvoBrickSelectPipeline::new(context),
            brick_instances: BrickInstances::new(&context.gpu, 1024),
            instance_transforms: HashMap::new(),
        }
    }
}

impl RenderModule<Scene> for SvoSdfBricksRenderModule {
    
    /// Prepares list of nodes to be rendered in this frame.
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        
        // Gather transforms (instances) for each geometry
        let mut geometry_instances: HashMap<GeometryID, Vec<Transform>> = HashMap::new();
        {
            profiler::scope!("Gather Geometry Instances");
            for (_, model) in scene.model_pool.iter() {
                let (transform, geometry_id) = (&model.transform, &model.geometry_id);
                geometry_instances.entry(*geometry_id)
                    .and_modify(|transforms| { transforms.push(transform.clone()) })
                    .or_insert_with(|| vec![transform.clone()]);
            }
        }
        
        // let frustum_camera = crate::framework::camera::Camera {
        //     position: (2.0, 0.0, 0.0).into(),
        //     ..scene.camera_rig.camera
        // }.look_at((0.0, 0.0, 0.0).into());
        let frustum_camera = scene.camera_rig.camera();
        let frustum = Frustum::from_camera(&frustum_camera);
        
        for (geometry_id, transforms) in geometry_instances.iter() {
            let geometry = scene.geometry_pool.get(*geometry_id).expect("Unexpected Error: Geometry not found in pool");
            
            counters::sample!("object_instance_counter", transforms.len() as f64);
            let transforms = {
                profiler::scope!("Frustum Culling (Geometry Level)");
                transforms.iter().filter_map(|t| {
                    if geometry.total_aabb().transform(t).in_frustum(&frustum) {
                        Some(t.clone())
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            };
            counters::sample!("object_selected_counter", transforms.len() as f64);
            
            let Some(svo) = &geometry.svo else {
                error!("Cannot instantiate Geometry {:?}, geometry has no SVO", geometry_id);
                continue;
            };
            
            let Some(node_count) = svo.node_pool.count() else {
                error!("Cannot instantiate Geometry {:?}, its SVO node pool is empty or does not have node_count loaded from GPU", geometry_id);
                continue;
            };
            
            let gpu_transforms = self.instance_transforms.entry(*geometry_id)
                .and_modify(|gpu_transforms| { gpu_transforms.update(&context.gpu, &transforms) })
                .or_insert_with(|| GPUGeometryTransforms::from_transforms(&context.gpu, &transforms));
            
            self.brick_instances.clear_resize(&context.gpu, transforms.len() * node_count as usize);
            self.brick_select_compute_pipeline.run(
                context,
                svo,
                scene.brick_level_break_size,
                &self.brick_instances,
                &gpu_transforms
            );
            
            {
                profiler::scope!("BrickInstances::load_count", pinned);
                // TODO: (!!!SLOW!!!) this will not be needed when we will use indirect draw.
                // TODO: Add node count to GUI display -> there has to be a global stat counter accessible even when scene is immutable
                let cnt = self.brick_instances.load_count(&context.gpu);
                counters::sample!("brick_selected_counter", cnt as f64);
            };
            
            self.pipeline.set_svo(&context.gpu, svo, &gpu_transforms);
            self.pipeline.set_display_options(scene.display_toggles.brick_display_options);
            
            // NOTE: For now on we render only one (first) geometry with all its instances
            //       Instead of set_svo and set_display_options there would be "push" or "submit" methods.
            break;
        }
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPass::Base { .. },
                render_pass,
            } => {
                self.pipeline.render_on_pass(render_pass, context, &self.brick_instances);
            }
            _ => {}
        }
    }

    fn finalize(&mut self) {}
}
