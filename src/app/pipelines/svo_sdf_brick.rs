
use std::borrow::Cow;

use crate::{
    framework::gpu::{self, vertices::Vertex},
    sdf::svo::{self, Svo},
    app::{
        renderer::RenderContext,
        objects::cube::{CUBE_INDICES_TRIANGLE_STRIP, CubeSolidMesh},
    },
};

type BrickInstanceBuffer = gpu::Buffer<u32>;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    view_projection: glam::Mat4,
    camera_position: glam::Vec4,
    brick_scale: f32,
    brick_atlas_stride: f32,
    brick_voxel_size: f32,
    padding: f32,
}

#[derive(Debug)]
struct SvoBindGroups {
    pub node_pool: wgpu::BindGroup,
    pub brick_pool: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct SvoSDFBrickPipeline {
    pub brick_instance_buffer: BrickInstanceBuffer, // public, because it is updated from outside
    pipeline: wgpu::RenderPipeline,
    node_pool_bind_group_layout: wgpu::BindGroupLayout,
    brick_pool_bind_group_layout: wgpu::BindGroupLayout,
    cube_solid_mesh: CubeSolidMesh,
    bind_groups: Option<SvoBindGroups>,
    push_constants: PushConstants,
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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/svo_sdf_brick.wgsl"))),
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
                        // set to size of push constant camera data
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
                    BrickInstanceBuffer::vertex_layout(),
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
            brick_instance_buffer: BrickInstanceBuffer::new_empty(
                &context.gpu,
                Some("SDF Pipeline brick instance buffer"),
                100,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ),
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
        self.push_constants.brick_atlas_stride = svo.brick_pool.atlas_stride();
        self.push_constants.brick_voxel_size = svo.brick_pool.atlas_voxel_size();
        self.push_constants.brick_scale = svo.brick_pool.atlas_scale();
    }
    
    /// Runs this pipeline for given render pass
    pub fn render_on_pass<'rpass>(&'rpass self, pass: &mut wgpu::RenderPass<'rpass>, context: &RenderContext) {
        if let Some(bind_groups) = self.bind_groups.as_ref() {
            pass.set_pipeline(&self.pipeline);
            
            let cpc = context.camera.to_push_constant_data();
            let pc = PushConstants {
                view_projection: cpc.view,
                camera_position: cpc.position,
                ..self.push_constants
            };
            
            pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(&[pc]
            ));
            
            pass.set_bind_group(0, &bind_groups.node_pool, &[]);
            pass.set_bind_group(1, &bind_groups.brick_pool, &[]);
            
            pass.set_vertex_buffer(0, self.cube_solid_mesh.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.brick_instance_buffer.buffer.slice(..));
            pass.set_index_buffer(self.cube_solid_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(
                0..CUBE_INDICES_TRIANGLE_STRIP.len() as u32,
                0,
                0..self.brick_instance_buffer.size as u32
            );
        }
    }
}
