
use slotmap::Key;

use super::{
    SvoSDFBrickPipeline,
    SvoBrickSelectPipeline,
};

use crate::{
    error,
    demo_app::scene::Scene,
    sdf::geometry::{
        GeometryID,
        Geometry,
    },
    framework::{
        gui::Gui,
        math::{Transform, Frustum},
        renderer::{
            RenderModule,
            RenderContext,
            RenderPassContext,
            RenderPass,
        },
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
        
        let frustum_camera = scene.camera_rig.camera();
        // let frustum_camera = crate::framework::camera::Camera {
        //     position: (0.0, 00.0, 0.0).into(),
        //     ..*scene.camera_rig.camera()
        // }.look_at((1.0, 0.0, 1.0).into());
        let frustum = Frustum::from_camera(&frustum_camera);
        
        // Gather transforms (instances) for each geometry in the view frustum into buckets, basically a map where each index corresponds to a geometry id
        let mut buckets: Vec<Option<(GeometryID, &Geometry, Vec<Transform>)>> = vec![None; scene.geometry_pool.capacity()];
        {
            #[inline]
            fn geometry_id_to_index(id: &GeometryID) -> usize {
                (id.data().as_ffi() & 0xffff_ffff) as usize - 1
            }
            
            profiler::scope!("Gather Geometry Instances, to be rendered", pinned);
            
            for (id, geometry) in scene.geometry_pool.iter() {
                buckets[geometry_id_to_index(&id)] = Some((id, geometry, vec![]));
            }
            
            counters::sample!("object_instance_counter", scene.world.query::<(&GeometryID, &Transform)>().iter().count() as f64);
            
            #[cfg(feature = "counters")]
            let mut cnt: u32 = 0;
            
            for (_, (geometry_id, transform)) in scene.world.query::<(&GeometryID, &Transform)>().iter() {
                
                let Some((_, geometry, transforms)) = &mut buckets[geometry_id_to_index(geometry_id)] else {
                    error!("Cannot find geometry {:?} in the bucket", geometry_id);
                    continue;
                };
                
                // Frustum culling
                if !geometry.total_aabb().transform(transform).in_frustum(&frustum) {
                    continue;
                }
                
                #[cfg(feature = "counters")]
                { cnt += 1; }
                
                transforms.push(transform.clone());
            }
            
            counters::sample!("object_selected_counter", cnt as f64);
        }
        
        buckets.iter()
            .filter_map(|x| x.as_ref())
            .for_each(|(geometry_id, geometry, transforms)| {
                
                // this skips unseen instances
                if transforms.is_empty() {
                    return;
                }
                
                // this ensures that the geometry has a svo
                let Some(svo) = &geometry.svo else {
                    error!("Cannot instantiate Geometry {:?}, geometry has no SVO", geometry_id);
                    return;
                };
                
                // submit instances of the svo to the render pipeline
                let (
                    gpu_transforms,
                    brick_instances,
                ) = self.render_pipeline.submit_svo(&context.gpu, geometry_id, svo, &transforms);
                
                // use the brick select compute pipeline to fill associated buffers
                self.brick_select_compute_pipeline.run(
                    context,
                    svo,
                    scene.brick_level_break_size,
                    brick_instances,
                    gpu_transforms,
                    &frustum,
                );
            });
        
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
