
use std::borrow::Cow;

use crate::{
    warn,
    framework::{renderer::RenderContext, math},
    sdf::svo::{self, Svo},
};

use super::BrickInstances;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    domain:                   math::BoundingCube,
    camera_projection_matrix: glam::Mat4,
    camera_focal_length:      f32,
    node_count:               u32,
    level_break_size:         f32,
    _padding:                 [u32; 1], // TODO: level select distance
}


#[derive(Debug)]
pub struct SvoBrickSelectPipeline {
    pipeline:                          wgpu::ComputePipeline,
    node_pool_bind_group_layout:       wgpu::BindGroupLayout,
    brick_instances_bind_group_layout: wgpu::BindGroupLayout,
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
        
        let pipeline = { profiler::scope!("Create brick select pipeline");
            context.gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Brick select compute pipeline"),
                layout: Some(
                    &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Cube Outline Pipeline Layout"),
                        // define buffers layout of the svo
                        bind_group_layouts: &[
                            &node_pool_bind_group_layout,
                            &brick_instances_bind_group_layout,
                        ],
                        // set camera transform matrix as shader push constant
                        push_constant_ranges: &[wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::COMPUTE,
                            range: 0..std::mem::size_of::<PushConstants>() as u32,
                        }],
                    })
                ),
                entry_point: "main",
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
        }
    }
    
    #[profiler::function]
    pub fn run(&mut self, context: &RenderContext, svo: &Svo, brick_instances: &BrickInstances, level_break_size: f32) {
        
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
        
        // compute pass scope
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Brick select pass"),
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &node_bind_group, &[]);
            compute_pass.set_bind_group(1, &brick_instances_bind_group, &[]);
            
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[PushConstants {
                camera_projection_matrix: context.camera.projection_matrix,
                camera_focal_length: context.camera.focal_length,
                node_count,
                domain: svo.domain,
                level_break_size,
                ..Default::default()
            }]));
            
            compute_pass.dispatch_workgroups((node_count + 128 - 1) / 128, 1, 1);
        }
        
        encoder.pop_debug_group();
        context.gpu.queue.submit(Some(encoder.finish()));
    }
}
