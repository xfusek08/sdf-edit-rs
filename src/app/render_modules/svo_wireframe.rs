/// SVO scene rendering
/// This module handles rendering a pool of models which instantiates a geometries from distinct geometry pool

use std::borrow::Cow;

use wgpu::{PushConstantRange, util::DeviceExt};

use crate::app::{
    state::State,
    gpu::{
        vertices::{SimpleVertex, Vertex},
        texture::DepthStencilTexture, camera::GPUCameraPushConstantData, buffers::Buffer
    },
    renderer::{
        RenderContext,
        render_module::RenderModule,
        render_pass::{RenderPassAttachment, RenderPassContext}
    }, sdf::svo::NodePool,
};

type NodeVertexBuffer = Buffer<glam::Vec4>;
impl NodeVertexBuffer {
    pub fn vertex_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<glam::Vec4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![1 => Float32x4],
        }
    }
}

#[derive(Debug)]
pub struct SVOWireframeRenderModule {
    pipeline: wgpu::RenderPipeline,
    node_vertex_buffer: NodeVertexBuffer,
    cube: CubeModel,
}

impl SVOWireframeRenderModule {
    pub fn new(context: &RenderContext) -> Self {
        // ⬇ load and compile wgsl shader code
        let shader = context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/svo_wireframe.wgsl"))),
        });
        
        let node_vertex_buffer = NodeVertexBuffer::new(
            &context.gpu,
            Some("SVO Node Vertex Buffer for rendering"),
            &[
                glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
                glam::Vec4::new(0.25, 0.25, 0.25, 0.5),
                // glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            ],
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        
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
                buffers: &[
                    SimpleVertex::vertex_layout(),
                    NodeVertexBuffer::vertex_layout(),
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
            depth_stencil: Some(DepthStencilTexture::stencil()),
            
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        SVOWireframeRenderModule {
            pipeline,
            node_vertex_buffer,
            cube: CubeModel::new(&context.gpu.device),
        }
    }
}

impl RenderModule for SVOWireframeRenderModule {
    
    #[profiler::function]
    fn prepare(&mut self, state: &State, context: &RenderContext) {
        // NOTE: For now this implementation just copies all SVO vertices from all geometries into a single buffer
        // -------------------------------------------------------------------------------------------------------
        
        // Get all nodes from all valid node pools from all geometries with their node count
        let values: Vec<(u32, &NodePool)> = state.scene.geometry_pool
            .iter()
            .filter_map(|(_, geometry)| {
                if let Some(svo) = &geometry.svo {
                    if let Some(cnt) = &svo.node_pool.count() {
                        return Some((cnt.clone(), &svo.node_pool));
                    }
                }
                None
            })
            .collect();
        
        // Prepare command encoder
        let mut encoder = context.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SVO Render Module Encoder For copying vertices into node vertex buffer"),
        });
            
        // Lets ensure we have enough space in the buffer for all vertices by summing all node counts
        let total_count: u32 = values.iter().map(|(cnt, _)| cnt).sum();
        let total_count = total_count as usize;
        if total_count > self.node_vertex_buffer.capacity {
            profiler::scope!("Resizing node vertex buffer");
            encoder.push_debug_group("Resizing SVO Node Vertex Buffer");
            encoder.insert_debug_marker("Resizing node vertex buffer");
            self.node_vertex_buffer.resize(&context.gpu, total_count);
            encoder.pop_debug_group();
        }
        
        
        // Copy all vertices into the buffer from all node pools
        let mut vertices_copied = 0;
        self.node_vertex_buffer.size = 0;
        { profiler::scope!("Pushing all vertices from SVO to svo wireframe renderer vertex buffer");
            encoder.push_debug_group("Copying vertices from node pool to svo renderer");
            values.iter().for_each(|(cnt, node_pool)| {
                { profiler::scope!("SVO vertex buffer -> svo renderer vertex buffer");
                    encoder.copy_buffer_to_buffer(
                        node_pool.vertex_buffer(),
                        0,
                        &self.node_vertex_buffer.buffer,
                        vertices_copied as u64,
                        (cnt.clone() as usize * std::mem::size_of::<glam::Vec4>()) as u64
                    );
                }
                self.node_vertex_buffer.size += cnt.clone() as usize;
                vertices_copied += cnt.clone();
            });
            encoder.pop_debug_group();
        }
        
        // Submit command to queue
        profiler::call!(context.gpu.queue.submit(Some(encoder.finish())));
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
                render_pass.set_vertex_buffer(1, self.node_vertex_buffer.buffer.slice(..));
                render_pass.set_index_buffer(self.cube.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..CUBE_INDICES_LINE_STRIP.len() as u32, 0, 0..self.node_vertex_buffer.size as u32);
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
            contents: bytemuck::cast_slice(&CUBE_INDICES_LINE_STRIP),
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

const PRIMITIVE_RESTART: u16 = 0xFFFF; // primitive restart, see: https://github.com/gpuweb/gpuweb/issues/1002#issuecomment-679334425

const CUBE_INDICES_TRIANGLE_STRIP: &[u16] = &[
    // STRIP 1
    0, 1, 2, 3, 6, 7, 4, 5,
    
    PRIMITIVE_RESTART,
    
    // strip 2
    2, 6, 0, 4, 1, 5, 3, 7,
];

const CUBE_INDICES_LINE_STRIP: &[u16] = &[
    0, 1, 3, 7, 5, 1,
    PRIMITIVE_RESTART,
    5, 4, 0, 2, 6, 4,
    PRIMITIVE_RESTART,
    3, 2, 6, 7
];
