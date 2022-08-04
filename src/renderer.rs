
use std::borrow::Cow;

use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        
        let surface = unsafe { instance.create_surface(window) };
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }).await.expect("Failed to find an appropriate adapter");
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None
            ).await.expect("Failed to create device");
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,     // texture will be used to draw on screen
            format:       surface.get_supported_formats(&adapter)[0], // texture format - select first supported one
            width:        window.inner_size().width,
            height:       window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,                    // VSynch essentially - capping renders on display frame rate
        };
        surface.configure(&device, &surface_config);
        
        // ⬇ load and compile wgsl shader code
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../resources/shaders/shader.wgsl"))),
        });
        
        // ⬇ define layout of buffers for out render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        // ⬇ Create render pipeline (think more flexible OpenGL program)
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            // ⬇ Vertex shader -> define an entry point in our shader
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // <- Vertex buffer to be passed into the VS
            },
            // ⬇ Fragment shader -> define an entry point in our shader
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // ⬇ configure expected outputs from fragment shader
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,          // <- format out target texture (surface texture we will render into)
                    blend: Some(wgpu::BlendState::REPLACE), // <- how to bled colors (with alpha) previous frame
                    write_mask: wgpu::ColorWrites::ALL,     // <- which color component will be overridden by FS?
                })],
            }),
            // ⬇ How to interpret vertices in Vertex buffer and build primitives from them?
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // <- (primitive type in OpenGL) triplets of vertices are individual triangles
                strip_index_format: None,                        // <- format of indices in index buffer when drawing indexed topology
                front_face: wgpu::FrontFace::Ccw,                // <- Counter clockwise vertices are front-facing
                cull_mode: Some(wgpu::Face::Back),               // <- Cull Back faces of vertices.
                unclipped_depth: false,                          // <- ??? Requires Features::DEPTH_CLIP_CONTROL
                polygon_mode: wgpu::PolygonMode::Fill,           // <- Fill polygons with solid interpolated data
                conservative: false,                             // <- Enables conservative rasterization (Requires Features::CONSERVATIVE_RASTERIZATION)
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
            surface_config,
            surface,
            device,
            queue,
            render_pipeline,
        }
    }
    
    #[profiler::function]
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        
        // ask surface to provide us a texture we will draw into
        let output = profiler::call!(
            self
                .surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        );
        
        // View on surface texture understandable by RenderPassColorAttachment
        let view = profiler::call!(
            output.texture.create_view(&wgpu::TextureViewDescriptor::default())
        );
        
        // Create an encoder for building a GPU commands for this frame
        let mut encoder = profiler::call!(
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            })
        );
        
        // prepare a render pass command in the encoder
        { profiler::scope!("Prepare render pass");
            // ⬇ Make render_pass mutable to be able add a pipeline to it
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // ⬇ set target to which gpu will be drawing into - might be one or more textures
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, // <- Draw into our surface texture view
                    resolve_target: None, // <- Final resolved output (None -> view)
                    ops: wgpu::Operations {
                        // ⬇ What to do with previous frame colors? -> Clear with a solid opaque color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // ⬇ store rendered data to view?
                        store: true
                    }
                })],
                // ⬇ We do not use that for now
                depth_stencil_attachment: None
            });
            
            render_pass.set_pipeline(&self.render_pipeline); // <- set pipeline for render pass (OpenGL use program)
            render_pass.draw(0..3, 0..1); // <- Tell the pipeline how we want int to start what and haw many thing to draw. In this case we want to draw 3 vertices and one instance.
        } // drop render_pass here - because commands must not be borrowed before calling `finish()` on encoder
        
        profiler::call!(self.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
        Ok(())
    }
    
    #[profiler::function]
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}
