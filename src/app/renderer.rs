
use std::collections::HashMap;
use std::{borrow::Cow};

use bevy_ecs::prelude::Entity;
use bevy_ecs::query::{With, Changed, Added};
use bevy_ecs::system::{Query, ResMut};
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::components::Mesh;
use super::components::Texture as TextureComponent;
use super::camera::Camera;
use super::texture::Texture;
use super::vertex::Vertex;

struct RenderModel {
    vertex_count: usize,
    index_count: usize,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: Texture,
    texture_bind_group: wgpu::BindGroup,
}

struct RenderScene {
    models: HashMap<Entity, RenderModel>,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
}

pub struct Renderer {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    prepared_scene: Option<RenderScene>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        
        let surface = unsafe { instance.create_surface(window) };
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
        }).await.expect("Failed to find an appropriate adapter");
        
        let (device, queue) = adapter.request_device(
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
        
        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    },
                ]
            }
        );
        
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );
        
        // ⬇ load and compile wgsl shader code
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../resources/shaders/shader.wgsl"))),
        });
        
        // ⬇ define layout of buffers for out render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
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
                buffers: &[Vertex::desc()], // <- List of configurations where each item is a description of one vertex buffer (vertex puller configuration)
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
            texture_bind_group_layout,
            camera_bind_group_layout,
            prepared_scene: None,
        }
    }
    
    #[profiler::function]
    pub fn prepare_system(
        mut renderer: ResMut<Renderer>,
        camera_query: Query<&Camera>,
        changed_camera_query: Query<&Camera, Changed<Camera>>,
        models_query: Query<Entity, (With<Mesh>, With<TextureComponent>)>,
        models_changed_meshes_query: Query<(Entity, &Mesh, &TextureComponent), (With<Mesh>, With<TextureComponent>, Changed<Mesh>)>,
        models_changed_textures_query: Query<(Entity, &TextureComponent), (With<Mesh>, With<TextureComponent>, Changed<TextureComponent>)>,
    ) {
        
        let mut render_scene = match renderer.prepared_scene.take() {
            Some(scene) => scene,
            None => {
                // create new render scene when not exists
                let camera = camera_query.get_single().unwrap();
                let transform = camera.view_projection_matrix();
                let camera_uniform_buffer = renderer.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("camera_uniform_buffer"),
                        contents: bytemuck::cast_slice(&[transform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }
                );
                let camera_bind_group = renderer.device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("camera_bind_group"),
                        layout: &renderer.camera_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: camera_uniform_buffer.as_entire_binding()
                            }
                        ]
                    },
                );
                RenderScene {
                    models: HashMap::new(),
                    camera_uniform_buffer,
                    camera_bind_group,
                }
            }
        };
        
        
        // delete non existing models
        // TODO: maybe guard this with `was deleted` flag on scene
        render_scene.models.retain(|entity, _| { models_query.contains(*entity) });
        
        // update or insert changed model meshes
        for (entity, mesh, texture_component) in models_changed_meshes_query.iter() {
            match render_scene.models.get_mut(&entity) {
                Some(model) => {
                    // update existing models buffers
                },
                None => {
                    // create new model with new buffers
                    let vertex_buffer = renderer.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(mesh.vertices), // <- vertex buffer casted as array of bytes
                            usage: wgpu::BufferUsages::VERTEX,              // <- mark this buffer to be used as vertex buffer
                        }
                    );
                    let index_buffer = renderer.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(mesh.indices), // <- index buffer casted as array of bytes
                            usage: wgpu::BufferUsages::INDEX,              // <- mark this buffer to be used as vertex buffer
                        }
                    );
                    
                    let texture = Texture::from_image(
                        &renderer.device,
                        &renderer.queue,
                        &texture_component.texture,
                        Some("Texture")
                    ).unwrap();
                    
                    let texture_bind_group = renderer.device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            label: Some("texture_bind_group"),
                            layout: &renderer.texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&texture.view)
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&texture.sampler)
                                },
                            ]
                        }
                    );
                    
                    render_scene.models.insert(entity, RenderModel {
                        vertex_count: mesh.vertices.len(),
                        index_count: mesh.indices.len(),
                        vertex_buffer,
                        index_buffer,
                        texture,
                        texture_bind_group,
                    });
                }
            }
        }
        
        for (entity, texture_component) in models_changed_textures_query.iter() {
            let model = render_scene.models.get_mut(&entity).unwrap(); // previous loop inserted new items
            
            model.texture = Texture::from_image(
                &renderer.device,
                &renderer.queue,
                &texture_component.texture,
                Some("Texture")
            ).unwrap();
            
            model.texture_bind_group = renderer.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("texture_bind_group"),
                    layout: &renderer.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&model.texture.view)
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&model.texture.sampler)
                        },
                    ]
                }
            );
        }
        
        // update camera uniform
        if let Ok(camera) = changed_camera_query.get_single() {
            renderer.queue.write_buffer(
                &render_scene.camera_uniform_buffer,
                0,
                bytemuck::cast_slice(&[camera.view_projection_matrix()])
            );
        }
        
        renderer.prepared_scene = Some(render_scene);
    }

    
    #[profiler::function]
    pub fn render_system(renderer: ResMut<Renderer>) {
        
        // ask surface to provide us a texture we will draw into
        let output = profiler::call!(
            renderer
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
            renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
            
            render_pass.set_pipeline(&renderer.render_pipeline); // <- set pipeline for render pass (OpenGL use program)
            if let Some(scene) = renderer.prepared_scene.as_ref() {
                render_pass.set_bind_group(1, &scene.camera_bind_group, &[]);
                for (_, model) in &scene.models {
                    profiler::scope!("Render Pipeline - model draw");
                    render_pass.set_bind_group(0, &model.texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..)); // <- set a part of vertex buffers to be used in this render pass.
                    render_pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // <- set a part of index buffers to be used in this render pass.
                    render_pass.draw_indexed(0..model.index_count as u32, 0, 0..1); // <- Tell the pipeline how we want int to start what and haw many thing to draw. In this case we want to draw 3 vertices and one instance.
                }
            }
            
        } // drop render_pass here - because commands must not be borrowed before calling `finish()` on encoder
        
        profiler::call!(renderer.queue.submit(Some(encoder.finish())));
        profiler::call!(output.present());
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
