
use std::borrow::Cow;

use crate::{
    sdf::svo::{self, Svo},
    demo_app::modules::cube::{CUBE_INDICES_TRIANGLE_STRIP, CubeSolidMesh},
    framework::{
        renderer::RenderContext,
        gpu::{self, vertices::Vertex}, math,
    }, warn,
};

use super::BrickInstances;

// bit flags for showing solid brick, normals,  step count and depth
bitflags::bitflags! {
    #[repr(C)]
    #[derive(bytemuck::Pod, bytemuck::Zeroable)]
    pub struct DisplayOptions: u32 {
        const NONE       = 0;
        const SOLID      = 0b00000001;
        const NORMALS    = 0b00000010;
        const STEP_COUNT = 0b00000100;
        const DEPTH      = 0b00001000;
        const JUST_ROOT  = 0b00010000;
    }
}

impl Default for DisplayOptions {
    fn default() -> Self { Self::NONE }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    view_projection:    glam::Mat4,
    camera_position:    glam::Vec4,
    domain:             math::BoundingCube,
    brick_scale:        f32,
    brick_atlas_stride: f32,
    brick_voxel_size:   f32,
    display_options:    DisplayOptions,
}

#[derive(Debug)]
struct SvoBindGroups {
    pub node_pool:  wgpu::BindGroup,
    pub brick_pool: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct SvoSDFBrickPipeline {
    pipeline:                     wgpu::RenderPipeline,
    node_pool_bind_group_layout:  wgpu::BindGroupLayout,
    brick_pool_bind_group_layout: wgpu::BindGroupLayout,
    cube_solid_mesh:              CubeSolidMesh,
    bind_groups:                  Option<SvoBindGroups>,
    push_constants:               PushConstants,
}

impl SvoSDFBrickPipeline {
    pub fn new(context: &RenderContext) -> Self {
        let node_pool_bind_group_layout = svo::NodePool::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::VERTEX,
            true
        );
        
        let brick_pool_bind_group_layout = svo::BrickPool::create_read_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::FRAGMENT
        );
        
        let shader = context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Pipeline brick Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("_shader.wgsl"))),
        });
        
        let pipeline = context.gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SDF Pipeline brick Pipeline"),
            
            // Specify layout of buffers used by this pipeline
            layout: Some(
                &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("SDF Pipeline brick Pipeline Layout"),
                    // define buffers layout of the svo
                    bind_group_layouts: &[
                        &node_pool_bind_group_layout,  // 0 - Node Pool
                        &brick_pool_bind_group_layout, // 1 - Brick Pool
                    ],
                    // set camera transform matrix as shader push constant
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..std::mem::size_of::<PushConstants>() as u32,
                    }],
                })
            ),
            
            // Describe vertex stage
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    gpu::vertices::SimpleVertex::vertex_layout(),
                    gpu::Buffer::<u32>::vertex_layout(),
                ],
            },
            
            // Describe fragment stage
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: context.surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            
            // Set interpretation of vertices in vertex buffer
            // - This describes how cube instances will be rendered from vertex and index buffers
            primitive: wgpu::PrimitiveState {
                topology:           wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face:         wgpu::FrontFace::Ccw,   // Counter clockwise vertices are front-facing
                cull_mode:          Some(wgpu::Face::Back), // Cull back-facing triangles
                unclipped_depth:    false,
                polygon_mode:       wgpu::PolygonMode::Fill,
                conservative:       false,
            },
            
            // use depth buffer for depth testing (if any in context)
            depth_stencil: Some(gpu::DepthStencilTexture::stencil()),
            
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        Self {
            cube_solid_mesh: CubeSolidMesh::new(&context.gpu.device),
            pipeline,
            node_pool_bind_group_layout,
            brick_pool_bind_group_layout,
            bind_groups: None,
            push_constants: PushConstants::default(),
        }
    }
    
    pub fn set_svo(&mut self, gpu: &gpu::Context, svo: &Svo) {
        self.bind_groups = Some(SvoBindGroups {
            node_pool: svo.node_pool.create_bind_group(&gpu, &self.node_pool_bind_group_layout),
            brick_pool: svo.brick_pool.create_read_bind_group(&gpu, &self.brick_pool_bind_group_layout),
        });
        self.push_constants.domain = svo.domain;
        self.push_constants.brick_atlas_stride = svo.brick_pool.atlas_stride();
        self.push_constants.brick_voxel_size = svo.brick_pool.atlas_voxel_size();
        self.push_constants.brick_scale = svo.brick_pool.atlas_scale();
    }
    
    pub fn set_display_options(&mut self, options: DisplayOptions) {
        self.push_constants.display_options = options;
    }
    
    /// Runs this pipeline for given render pass
    pub fn render_on_pass<'rpass>(&'rpass self, pass: &mut wgpu::RenderPass<'rpass>, context: &RenderContext, brick_instance_buffer: &'rpass BrickInstances) {
        let Some(bind_groups) = self.bind_groups.as_ref()  else {
            return;
        };
        
        let Some(instance_count) = brick_instance_buffer.count() else {
            warn!("Count for brick instance buffer is not loaded.");
            return;
        };
        
        pass.set_pipeline(&self.pipeline);
        
        let cpc = context.camera.to_push_constant_data();
        let mut pc = PushConstants {
            view_projection: cpc.view,
            camera_position: cpc.position,
            ..self.push_constants
        };
        
        // if no bricks are in brick instance buffer, render root
        if instance_count == 0 {
            pc.display_options |= DisplayOptions::JUST_ROOT;
        }
        
        pass.set_push_constants(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[pc]
        ));
        
        pass.set_bind_group(0, &bind_groups.node_pool, &[]);
        pass.set_bind_group(1, &bind_groups.brick_pool, &[]);
        
        pass.set_vertex_buffer(0, self.cube_solid_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, brick_instance_buffer.buffer.buffer.slice(..));
        pass.set_index_buffer(self.cube_solid_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // TODO: use indirect to avoid pulling instance buffer count from gpu
        pass.draw_indexed(
            0..CUBE_INDICES_TRIANGLE_STRIP.len() as u32,
            0,
            0..instance_count.max(1)
        );
    }
}
