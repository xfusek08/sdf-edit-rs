
use std::borrow::Cow;

use crate::{
    framework::{
        renderer::{
            self,
            RenderContext,
        },
        gpu::{
            self,
            vertices::Vertex
        },
    },
};

use super::{
    CubeOutlineComponent,
    CubeWireframeMesh,
    CUBE_INDICES_LINE_STRIP
};

type CubeInstanceBuffer = gpu::Buffer<CubeOutlineComponent>;
impl CubeInstanceBuffer {
    pub fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Vec4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Float32x4],
        }
    }
}

#[derive(Debug)]
pub struct CubeOutlinePipeline {
    pub instance_buffer: CubeInstanceBuffer,
    cube_wireframe_mesh: CubeWireframeMesh,
    pipeline: wgpu::RenderPipeline,
}

impl CubeOutlinePipeline {
    
    pub fn new(context: &RenderContext) -> Self {
        let shader = context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube Outline Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("_outline_shader.wgsl"))),
        });
        
        let pipeline = context.gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cube Outline Pipeline"),
            
            // Specify layout of buffers used by this pipeline
            layout: Some(
                &context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Cube Outline Pipeline Layout"),
                    // define buffers layout of the svo
                    bind_group_layouts: &[],
                    // set camera transform matrix as shader push constant
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        // set to size of push constant camera data
                        range: 0..std::mem::size_of::<renderer::camera::PushConstantData>() as u32,
                    }],
                })
            ),
            
            // Describe vertex stage
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    gpu::vertices::SimpleVertex::vertex_layout(),
                    CubeInstanceBuffer::vertex_layout(),
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
                topology:           wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face:         wgpu::FrontFace::Ccw,   // Counter clockwise vertices are front-facing
                cull_mode:          None,
                unclipped_depth:    false,
                polygon_mode:       wgpu::PolygonMode::Line,
                conservative:       false,
            },
            
            // use depth buffer for depth testing (if any in context)
            depth_stencil: Some(gpu::DepthStencilTexture::stencil()),
            
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        Self {
            instance_buffer: CubeInstanceBuffer::new(
                &context.gpu,
                Some("Outlined cube instance buffer"),
                &[],
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            ),
            cube_wireframe_mesh: CubeWireframeMesh::new(&context.gpu.device),
            pipeline,
        }
    }
    
    /// Runs this pipeline for given render pass
    pub fn render_on_pass<'rpass>(&'rpass self, pass: &mut wgpu::RenderPass<'rpass>, camera: &renderer::camera::Camera) {
        pass.set_pipeline(&self.pipeline);
        pass.set_push_constants(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[camera.to_push_constant_data()]
        ));
        
        pass.set_vertex_buffer(0, self.cube_wireframe_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..));
        pass.set_index_buffer(self.cube_wireframe_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..CUBE_INDICES_LINE_STRIP.len() as u32, 0, 0..self.instance_buffer.size as u32);
    }
    
}
