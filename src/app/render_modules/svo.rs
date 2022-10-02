/// SVO scene rendering
/// This module handles rendering a pool of models which instantiates a geometries from distinct geometry pool

use std::borrow::Cow;

use wgpu::{PushConstantRange, util::DeviceExt};

use crate::app::{
    state::State,
    gpu::{
        vertices::{SimpleVertex, Vertex},
        texture::DepthStencilTexture, camera::GPUCameraPushConstantData
    },
    renderer::{
        RenderContext,
        render_module::RenderModule,
        render_pass::{RenderPassAttachment, RenderPassContext}
    },
};

#[derive(Debug)]
pub struct SVORenderModule {
    pipeline: wgpu::RenderPipeline,
    cube: CubeModel,
}

impl SVORenderModule {
    pub fn new(context: &RenderContext) -> Self {
        // ⬇ load and compile wgsl shader code
        let shader = context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/svo.wgsl"))),
        });
        
        let pipeline = context.gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SVO Render Pipeline"),
            
            // Specify layout of buffers used by this pipeline
            layout: Some(
                &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("SVO Render Pipeline Layout"),
                    // define buffers layout of the svo
                    bind_group_layouts: &[],
                    // set camera transform matrix as shader push constant
                    push_constant_ranges: &[PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        // set to size of push constant camera data
                        range: 0..std::mem::size_of::<GPUCameraPushConstantData>() as u32,
                    }],
                })
            ),
            
            // Describe vertex stage
            vertex:  wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SimpleVertex::vertex_layout()],
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
                cull_mode:          Some(wgpu::Face::Back), // Back face culling
                unclipped_depth:    false,
                polygon_mode:       wgpu::PolygonMode::Fill,
                // polygon_mode:       wgpu::PolygonMode::Line,
                conservative:       false,
            },
            
            // use depth buffer for depth testing (if any in context)
            depth_stencil: Some(DepthStencilTexture::stencil()),
            
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        SVORenderModule {
            pipeline,
            cube: CubeModel::new(&context.gpu.device),
        }
    }
}

impl RenderModule for SVORenderModule {
    
    #[profiler::function]
    fn prepare(&mut self, state: &State, context: &RenderContext) {
        // TODO: obtain culled list of visible nodes from all instancies of all geometries into a GPU buffers which will be rendered.
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(
        &'a self,
        context: &'a RenderContext,
        render_pass_context: &mut RenderPassContext<'pass>,
    ) {
        match render_pass_context {
            RenderPassContext {
                attachment: RenderPassAttachment::Base { .. },
                render_pass
            } => {
                render_pass.set_pipeline(&self.pipeline);
                
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    0,
                    bytemuck::cast_slice(&[context.camera.to_push_constant_data()]
                ));
                
                render_pass.set_vertex_buffer(0, self.cube.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.cube.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..1);
            },
            _ => {}
        }
    }
    
    fn finalize(&mut self) {}
}

#[derive(Debug)]
pub struct CubeModel {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl CubeModel {
    
    #[profiler::function]
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

const CUBE_VERTICES: &[SimpleVertex] = &[
    // front face
    SimpleVertex(glam::Vec3::new(-0.5,  0.5, 0.5)), // 0 TL
    SimpleVertex(glam::Vec3::new(-0.5, -0.5, 0.5)), // 1 BL
    SimpleVertex(glam::Vec3::new( 0.5,  0.5, 0.5)), // 2 TR
    SimpleVertex(glam::Vec3::new( 0.5, -0.5, 0.5)), // 3 BR
    
    // back face
    SimpleVertex(glam::Vec3::new(-0.5,  0.5, -0.5)), // 4 TL
    SimpleVertex(glam::Vec3::new(-0.5, -0.5, -0.5)), // 5 BL
    SimpleVertex(glam::Vec3::new( 0.5,  0.5, -0.5)), // 6 TR
    SimpleVertex(glam::Vec3::new( 0.5, -0.5, -0.5)), // 7 BR
];

// The cube is created from two triangle strips
const CUBE_INDICES: &[u16] = &[
    // STRIP 1
    0, 1, 2, 3, 6, 7, 4, 5,
    
    0xFFFF, // primitive restart, see: https://github.com/gpuweb/gpuweb/issues/1002#issuecomment-679334425
    
    // strip 2
    2, 6, 0, 4, 1, 5, 3, 7,
];
