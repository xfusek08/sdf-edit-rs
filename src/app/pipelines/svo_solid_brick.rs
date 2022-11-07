
use std::borrow::Cow;

use crate::{
    framework::gpu::{self, vertices::Vertex},
    sdf::svo::{self, Svo},
    app::{
        objects::cube::{CubeSolidMesh, CUBE_INDICES_TRIANGLE_STRIP},
        renderer::RenderContext,
    },
};

type SDFBrickInstanceBuffer = gpu::Buffer<u32>;
#[derive(Debug)]
struct SvoBindGroups {
    pub node_pool: wgpu::BindGroup,
    pub brick_pool: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct SvoSolidBrickPipeline {
    pub brick_instance_buffer: SDFBrickInstanceBuffer, // public, because it is updated from outside
    node_pool_bind_group_layout: wgpu::BindGroupLayout,
    brick_pool_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    cube_solid_mesh: CubeSolidMesh,
    bind_groups: Option<SvoBindGroups>,
}

impl SvoSolidBrickPipeline {
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
            label: Some("Solid Pipeline brick Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/svo_solid_brick.wgsl"))),
        });
        
        let pipeline = context.gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Solid Pipeline brick Pipeline"),
            
            // Specify layout of buffers used by this pipeline
            layout: Some(
                &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Solid Pipeline brick Pipeline Layout"),
                    // define buffers layout of the svo
                    bind_group_layouts: &[
                        &node_pool_bind_group_layout,  // 0 - Node Pool
                        &brick_pool_bind_group_layout, // 1 - Brick Pool
                    ],
                    // set camera transform matrix as shader push constant
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        // set to size of push constant camera data
                        range: 0..std::mem::size_of::<gpu::camera::PushConstantData>() as u32,
                    }],
                })
            ),
            
            // Describe vertex stage
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    gpu::vertices::SimpleVertex::vertex_layout(),
                    SDFBrickInstanceBuffer::vertex_layout(),
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
            brick_instance_buffer: SDFBrickInstanceBuffer::new_empty(
                &context.gpu,
                Some("Solid Pipeline brick instance buffer"),
                100,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ),
            cube_solid_mesh: CubeSolidMesh::new(&context.gpu.device),
            pipeline,
            node_pool_bind_group_layout,
            brick_pool_bind_group_layout,
            bind_groups: None,
        }
    }
    
    pub fn set_svo(&mut self, gpu: &gpu::Context, svo: &Svo) {
        self.bind_groups = Some(SvoBindGroups {
            node_pool: svo.node_pool.create_bind_group(&gpu, &self.node_pool_bind_group_layout),
            brick_pool: svo.brick_pool.create_read_bind_group(&gpu, &self.brick_pool_bind_group_layout),
        });
    }
    
    /// Runs this pipeline for given render pass
    pub fn render_on_pass<'rpass>(&'rpass self, pass: &mut wgpu::RenderPass<'rpass>, context: &RenderContext) {
        if let Some(bind_groups) = self.bind_groups.as_ref() {
            pass.set_pipeline(&self.pipeline);
            pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(&[context.camera.to_push_constant_data()]
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
