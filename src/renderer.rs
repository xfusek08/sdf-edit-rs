
use std::borrow::Cow;
use winit::{window::Window, dpi::PhysicalSize};

pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        // wgpu initiation
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        
        let surface = unsafe { instance.create_surface(window) };
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None
            )
            .await
            .expect("Failed to create device");
        
        let swapchain_format = surface.get_supported_formats(&adapter)[0];
        
        let shader = profiler::call!(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../resources/shaders/shader.wgsl"))),
        }));
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())]
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);
        
        Self {
            instance,
            surface_config,
            surface,
            adapter,
            device,
            queue,
            render_pipeline,
        }
    }
    
    #[profiler::function]
    pub fn draw(&mut self) {
        let frame = profiler::call!(
            self.surface
                .get_current_texture()
                .expect("Failed to acquire next swap chain texture")
        );
            
        let view = profiler::call!(frame.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        ));
        
        let mut encoder = profiler::call!(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None }
        ));
        
        {
            profiler::scope!("Prepare render pass");
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true
                            }
                        })
                    ],
                    depth_stencil_attachment: None
                }
            );
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }
        profiler::call!(self.queue.submit(Some(encoder.finish())));
        profiler::call!(frame.present());
    }
    
    #[profiler::function]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
