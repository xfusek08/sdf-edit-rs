
use std::collections::HashMap;

use super::{
    SvoSDFBrickPipeline,
    SvoBrickSelectPipeline,
};

use crate::{
    error,
    demo_app::scene::Scene,
    sdf::geometry::{GeometryID, Geometry},
    framework::{
        math::{Transform, Frustum},
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPass,
        }, gui::Gui,
    },
};

///! This is main renderer of evaluated geometries

#[derive(Debug)]
pub struct SvoSdfBricksRenderModule {
    render_pipeline: SvoSDFBrickPipeline,
    brick_select_compute_pipeline: SvoBrickSelectPipeline,
}

impl SvoSdfBricksRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        counters::register!("object_selected_counter");
        counters::register!("object_instance_counter");
        Self {
            render_pipeline: SvoSDFBrickPipeline::new(context),
            brick_select_compute_pipeline: SvoBrickSelectPipeline::new(context),
        }
    }
}

impl RenderModule<Scene> for SvoSdfBricksRenderModule {
    
    /// Prepares list of nodes to be rendered in this frame.
    #[profiler::function(pinned)]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        
        self.render_pipeline.set_display_options(scene.display_toggles.brick_display_options);
        
        // let frustum_camera = crate::framework::camera::Camera {
        //     position: (2.0, 0.0, 0.0).into(),
        //     ..scene.camera_rig.camera
        // }.look_at((0.0, 0.0, 0.0).into());
        let frustum_camera = scene.camera_rig.camera();
        let frustum = Frustum::from_camera(&frustum_camera);
        
        // Gather transforms (instances) for each geometry in the view frustum
        let mut geometry_instances: HashMap<GeometryID, (&Geometry, Vec<Transform>)> = HashMap::new();
        {
            profiler::scope!("Gather Geometry Instances, to be rendered", pinned);
            counters::sample!("object_instance_counter", scene.model_pool.len() as f64);
            for (_, model) in scene.model_pool.iter() {
                let (transform, geometry_id) = (&model.transform, &model.geometry_id);
                let geometry = scene.geometry_pool.get(*geometry_id).expect("Unexpected Error: Geometry not found in pool");
                
                // Frustum culling
                if !geometry.total_aabb().transform(transform).in_frustum(&frustum) {
                    continue;
                }
                
                geometry_instances.entry(*geometry_id)
                    .and_modify(|(_, transforms)| { transforms.push(transform.clone()) })
                    .or_insert_with(|| (geometry, vec![transform.clone()]));
            }
            counters::sample!(
                "object_selected_counter",
                geometry_instances.iter().fold(0, |acc, (_, (_, transforms))| acc + transforms.len()) as f64
            );
        }
        
        // For each geometry, select bricks to be rendered and submit them to the pipeline
        for (geometry_id, (geometry, transforms)) in geometry_instances.iter() {
            
            let Some(svo) = &geometry.svo else {
                error!("Cannot instantiate Geometry {:?}, geometry has no SVO", geometry_id);
                continue;
            };
            
            let (
                gpu_transforms,
                brick_instances,
            ) = self.render_pipeline.submit_svo(&context.gpu, geometry_id, svo, &transforms);
            
            self.brick_select_compute_pipeline.run(
                context,
                svo,
                scene.brick_level_break_size,
                brick_instances,
                gpu_transforms,
            );
        }
        
        self.render_pipeline.load_counts(&context.gpu);
        
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
                self.render_pipeline.render_on_pass(render_pass, context);
            }
            _ => {}
        }
    }

    fn finalize(&mut self) {}
}
