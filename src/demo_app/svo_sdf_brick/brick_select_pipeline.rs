
use std::borrow::Cow;

use super::{
    BrickInstances,
    GPUGeometryTransforms,
};

use crate::{
    warn,
    sdf::svo::{
        self,
        Svo,
    },
    framework::{
        math,
        renderer::RenderContext,
    },
};

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    domain:                   math::BoundingCube,
    camera_projection_matrix: glam::Mat4,
    camera_focal_length:      f32,
    camera_far:               f32,
    camera_near:              f32,
    node_count:               u32,
    level_break_size:         f32,
    _padding:                 [u32; 3], // TODO: level select distance
}


#[derive(Debug)]
pub struct SvoBrickSelectPipeline {
    pipeline:                                        wgpu::ComputePipeline,
    node_pool_bind_group_layout:                     wgpu::BindGroupLayout,
    brick_instances_bind_group_layout:               wgpu::BindGroupLayout,
    geometry_instances_transforms_bind_group_layout: wgpu::BindGroupLayout,
}

impl SvoBrickSelectPipeline {
    
    #[profiler::function]
    pub fn new(context: &RenderContext) -> Self {
        
        let node_pool_bind_group_layout = svo::NodePool::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::COMPUTE,
            true
        );
        
        let brick_instances_bind_group_layout = BrickInstances::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::COMPUTE,
            false
        );
        
        let geometry_instances_transforms_bind_group_layout = GPUGeometryTransforms::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::COMPUTE,
        );
        
        let pipeline = {
            profiler::scope!("Create brick select pipeline");
            context.gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Brick select compute pipeline"),
                entry_point: "main",
                layout: Some(
                    &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Cube Outline Pipeline Layout"),
                        // define buffers layout of the svo
                        bind_group_layouts: &[
                            &node_pool_bind_group_layout,
                            &brick_instances_bind_group_layout,
                            &geometry_instances_transforms_bind_group_layout,
                        ],
                        // set camera transform matrix as shader push constant
                        push_constant_ranges: &[wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::COMPUTE,
                            range: 0..std::mem::size_of::<PushConstants>() as u32,
                        }],
                    })
                ),
                module: &context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Brick Select Compute Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("_brick_select.wgsl"))),
                }),
            })
        };
        
        Self {
            pipeline,
            node_pool_bind_group_layout,
            brick_instances_bind_group_layout,
            geometry_instances_transforms_bind_group_layout,
        }
    }
    
    #[profiler::function]
    pub fn run(
        &mut self,
        context:          &RenderContext,
        svo:              &Svo,
        level_break_size: f32,
        brick_instances:  &BrickInstances,
        transforms:       &GPUGeometryTransforms,
    ) {
        
        let Some(node_count) = svo.node_pool.count() else {
            warn!("SvoBrickSelectPipeline::run: Svo node pool is empty or node count is not loaded back from the gpu");
            return;
        };
        
        // Prepare encoder
        let mut encoder = context.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Brick select encoder"),
        });
        
        // Prepare bind groups
        let node_bind_group = svo.node_pool.create_bind_group(&context.gpu, &self.node_pool_bind_group_layout);
        let brick_instances_bind_group = brick_instances.create_bind_group(&context.gpu, &self.brick_instances_bind_group_layout);
        let geometry_instances_transforms_bind_group = transforms.create_bind_group(&context.gpu, &self.geometry_instances_transforms_bind_group_layout);
        
        // compute pass scope
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Brick select pass"),
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &node_bind_group, &[]);
            compute_pass.set_bind_group(1, &brick_instances_bind_group, &[]);
            compute_pass.set_bind_group(2, &geometry_instances_transforms_bind_group, &[]);
            
            let frustum_camera = crate::framework::camera::Camera {
                position: (2.0, 0.0, 0.0).into(),
                ..context.camera.camera
            }.look_at((0.0, 0.0, 0.0).into());
            
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[PushConstants {
                node_count,
                level_break_size,
                // camera_projection_matrix: frustum_camera.view_projection_matrix(),
                // camera_focal_length:      frustum_camera.focal_length(),
                // camera_far:               frustum_camera.far,
                // camera_near:              frustum_camera.near,
                camera_projection_matrix: context.camera.view_projection_matrix,
                camera_focal_length:      context.camera.camera.focal_length(),
                camera_far:               context.camera.camera.far,
                camera_near:              context.camera.camera.near,
                domain:                   svo.domain,
                ..Default::default()
            }]));
            
            compute_pass.dispatch_workgroups(node_count, 1, 1);
        }
        
        context.gpu.queue.submit(Some(encoder.finish()));
    }
}
