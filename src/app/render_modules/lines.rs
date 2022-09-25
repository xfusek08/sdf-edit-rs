/// TODO: Better line renderer: Components will carry only begin and end points, and the renderer will draw all lines in single draw call using vertex buffer.
///    - Instancing is not necessary here because it would not bring any advantage over using single vertex buffer.
///    - In the future versions it is possible to implement:
///      - Anti-aliasing
///      - Line width
///      - Curved lines


use std::{
    collections::{HashMap, hash_map::Entry},
    borrow::Cow
};

use hecs::Entity;

use crate::app::{
    gpu::{
        GPUContext,
        buffers::VertexBuffer,
        vertices::{ ColorVertex, Vertex}
    },
    rendering::{RenderModule, RenderContext},
    gui::Gui,
    scene::Scene,
    components::Deleted
};

// ECS Components to define line (renderable) entity
// -------------------------------------------------

pub struct LineMesh {
    pub is_dirty: bool,
    pub vertices: &'static [ColorVertex],
}

// Line Render Resource
// --------------------

struct LineRenderResource {
    vertex_buffer: VertexBuffer,
}
impl LineRenderResource {
    fn new(mesh: &LineMesh, context: &GPUContext) -> Self {
        Self {
            vertex_buffer: VertexBuffer::new(Some("Line Vertex Buffer"), mesh.vertices, context)
        }
    }
    fn update(&mut self, mesh: &LineMesh, context: &GPUContext) {
        self.vertex_buffer.update(context, mesh.vertices);
    }
}

// Line Renderer
// -------------

pub struct LineRenderer {
    pipeline: wgpu::RenderPipeline,
    render_resources: HashMap<Entity, LineRenderResource>,
}

// Construct this render module (a pipeline) from render context
impl LineRenderer {
    
    #[profiler::function]
    pub fn new(context: &RenderContext) -> Self {
        
        // ⬇ load and compile wgsl shader code
        let shader = context.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../../resources/shaders/line.wgsl"))),
        });
        
        // ⬇ define layout of buffers for out render pipeline
        let pipeline_layout = context.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Render Pipeline Layout"),
            bind_group_layouts: &[&context.camera.bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // ⬇ Create render pipeline (think more flexible OpenGL program)
        let pipeline = context.gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(&pipeline_layout),
            // ⬇ Vertex shader -> define an entry point in our shader
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ColorVertex::layout()], // <- List of configurations where each item is a description of one vertex buffer (vertex puller configuration)
            },
            // ⬇ Fragment shader -> define an entry point in our shader
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // ⬇ configure expected outputs from fragment shader
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.surface_config.format,     // <- format out target texture (surface texture we will render into)
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // <- how to bled colors (with alpha) previous frame
                    write_mask: wgpu::ColorWrites::ALL,            // <- which color component will be overridden by FS?
                })],
            }),
            // ⬇ How to interpret vertices in Vertex buffer and build primitives from them?
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList, // <- (primitive type in OpenGL) triplets of vertices are individual triangles
                strip_index_format: None,                    // <- format of indices in index buffer when drawing indexed topology
                front_face: wgpu::FrontFace::Ccw,            // <- Counter clockwise vertices are front-facing
                cull_mode: Some(wgpu::Face::Back),           // <- Cull Back faces of vertices.
                unclipped_depth: false,                      // <- ??? Requires Features::DEPTH_CLIP_CONTROL
                polygon_mode: wgpu::PolygonMode::Fill,       // <- Fill polygons with solid interpolated data
                conservative: false,                         // <- Enables conservative rasterization (Requires Features::CONSERVATIVE_RASTERIZATION)
            },
            depth_stencil: None, // <- do not use stencils
            // ⬇ configure multisampling
            multisample: wgpu::MultisampleState {
                count: 1, // <- number of samples
                mask: !0, // use all the samples
                alpha_to_coverage_enabled: false, // <- an antialiasing settings ???
            },
            multiview: None, // <- this allows us to set drawing into array of textures (maximum render attachments count)
        });
        
        Self {
            pipeline,
            render_resources: HashMap::new()
        }
    }
    
}

impl RenderModule for LineRenderer {
    
    #[profiler::function]
    fn prepare(&mut self, _: &Gui, scene: &Scene, context: &RenderContext) {
        // For each proper line entity is scene world, update render resources
        for (
            entity,
            (
                mesh,
                Deleted(deleted)
            )
        ) in scene.world.query::<(
            &LineMesh,
            &Deleted
        )>().iter() {
            profiler::scope!("Preparing line entity");
            
            if *deleted {
                self.render_resources.remove(&entity);
                continue;
            }
            
            if !mesh.is_dirty {
                continue;
            }
            
            match self.render_resources.entry(entity) {
                Entry::Occupied(mut oe) => {
                    oe.get_mut().update(mesh, &context.gpu);
                },
                Entry::Vacant(ve) => {
                    ve.insert(LineRenderResource::new(mesh, &context.gpu));
                },
            }
        }
    }
    
    #[profiler::function]
    fn render<'pass, 'a: 'pass>(&'a mut self, context: &'a RenderContext, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &context.camera.bind_group, &[]);
        
        // TODO: for now there is one draw call per line entity, but we can optimize this by drawing all line entities in one draw call using instanced rendering
        for (_, LineRenderResource { vertex_buffer }) in &self.render_resources {
            profiler::scope!("Draw Line entity");
            render_pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
            render_pass.draw(0..vertex_buffer.size as u32, 0..1);
        }
        
    }
    
    #[profiler::function]
    fn finalize(&mut self, _gui: &mut Gui, scene: &mut crate::app::scene::Scene) {
        // for each entity in scene world with line mesh
        for (_, mesh) in scene.world.query_mut::<&mut LineMesh>() {
            mesh.is_dirty = false;
        }
    }
    
}
