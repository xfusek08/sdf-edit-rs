
use std::{
    borrow::Cow,
    collections::HashMap
};

use crate::{
    error,
    info,
    sdf::{
        geometry::GeometryID,
        svo::{
            self,
            Svo,
        },
    },
    demo_app::cube::{
        CUBE_INDICES_TRIANGLE_STRIP,
        CubeSolidMesh
    },
    framework::{
        gpu,
        math,
        renderer::RenderContext,
    },
};

use super::{
    BrickInstances,
    BrickInstance,
    GPUGeometryTransforms
};

// bit flags for showing solid brick, normals,  step count and depth
bitflags::bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct DisplayOptions: u32 {
        const NONE       = 0;
        const SOLID      = 0b00000001;
        const NORMALS    = 0b00000010;
        const STEP_COUNT = 0b00000100;
        const DEPTH      = 0b00001000;
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
    focal_length:       f32,
    brick_scale:        f32,
    brick_atlas_stride: f32,
    brick_voxel_size:   f32,
    display_options:    DisplayOptions,
    _padding:           [u32; 3],
}

#[derive(Debug)]
struct SVORenderRecord {
    pub render:                bool,
    pub domain:                math::BoundingCube,
    pub brick_atlas_stride:    f32,
    pub brick_voxel_size:      f32,
    pub brick_scale:           f32,
    pub node_pool_bind_group:  wgpu::BindGroup,
    pub brick_pool_bind_group: wgpu::BindGroup,
    pub brick_instance_buffer: BrickInstances,
    pub instance_buffer:       GPUGeometryTransforms,
    pub instance_bind_group:   wgpu::BindGroup,
}

#[derive(Debug)]
pub struct SvoSDFBrickPipeline {
    pipeline:                     wgpu::RenderPipeline,
    node_pool_bind_group_layout:  wgpu::BindGroupLayout,
    brick_pool_bind_group_layout: wgpu::BindGroupLayout,
    instance_bind_group_layout:   wgpu::BindGroupLayout,
    cube_solid_mesh:              CubeSolidMesh,
    svos_to_render:               HashMap<GeometryID, SVORenderRecord>,
    display_options:              DisplayOptions,
}

impl SvoSDFBrickPipeline {
    pub fn new(context: &RenderContext) -> Self {
        counters::register!("brick_selected_counter");
        
        let node_pool_bind_group_layout = svo::NodePool::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::VERTEX,
            true
        );
        
        let brick_pool_bind_group_layout = svo::BrickPool::create_read_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::FRAGMENT, // Brick data are read in the fragment shader for ray-marching
        );
        
        let instance_bind_group_layout = GPUGeometryTransforms::create_bind_group_layout(
            &context.gpu,
            wgpu::ShaderStages::VERTEX,
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
                        &instance_bind_group_layout,   // 2 - Instance Buffer
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
                    // Vertices of the cube to me instanced
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<gpu::vertices::SimpleVertex>() as wgpu::BufferAddress,
                        step_mode:    wgpu::VertexStepMode::Vertex,
                        attributes:   &wgpu::vertex_attr_array![0 => Float32x3],
                    },
                    
                    // Data pulled per instance -> indices of the brick in svo and index of the svo (model) instance in the instance buffer
                    wgpu::VertexBufferLayout {
                        step_mode:    wgpu::VertexStepMode::Instance,
                        array_stride: std::mem::size_of::<BrickInstance>() as wgpu::BufferAddress,
                        attributes:   &wgpu::vertex_attr_array![
                            1 => Uint32,
                            2 => Uint32,
                        ],
                    }
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
                unclipped_depth:    false, // Allow depth values to be outside of 0.0 to 1.0 range
                polygon_mode:       wgpu::PolygonMode::Fill,
                conservative:       false,
            },
            
            // use depth buffer for depth testing (if any in context)
            depth_stencil: Some(gpu::DepthStencilTexture::stencil()),
            
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        Self {
            pipeline,
            node_pool_bind_group_layout,
            brick_pool_bind_group_layout,
            instance_bind_group_layout,
            cube_solid_mesh: CubeSolidMesh::new(&context.gpu.device),
            svos_to_render:  HashMap::new(),
            display_options: DisplayOptions::default(),
        }
    }
    
    /// Submits an SVO for rendering with the given GPU context and instance transforms.
    ///
    /// This function creates or modifies an entry in the `svos_to_render` map for the given
    /// `GeometryID`, and returns references to the GPU buffers containing the geometry and
    /// brick instances.
    ///
    /// # Arguments
    ///
    /// * `gpu` - The GPU context to use for rendering.
    /// * `id` - The `GeometryID` of the SVO to render.
    /// * `svo` - The SVO data to render.
    /// * `instance_transforms` - A vector of transforms for the SVO instances to be rendered.
    ///
    /// # Returns
    ///
    /// A tuple containing references to the GPU buffers containing the geometry and brick
    /// instances for the submitted SVO.
    ///
    ///   - The `GPUGeometryTransforms` buffer will be filled with the transforms for each instance.
    ///   - The `BrickInstances` buffer will be empty and are expected to be filled with bricks that are supposed to be rendered.
    #[profiler::function]
    pub fn submit_svo(
        &mut self,
        gpu: &gpu::Context,
        id: &GeometryID,
        svo: &Svo,
        instance_transforms: &Vec<math::Transform>
    ) -> (&GPUGeometryTransforms, &BrickInstances) {
        let domain = svo.domain;
        let brick_atlas_stride =svo.brick_pool.atlas_stride();
        let brick_voxel_size =svo.brick_pool.atlas_voxel_size();
        let brick_scale =svo.brick_pool.atlas_scale();
        
        // let node_count = svo.node_pool.count().expect(format!("SVO {:?} has no node count", id).as_str());
        let bottom_level = svo.levels.last().expect(format!("SVO {:?} has no bottom level", id).as_str());
        let node_count = bottom_level.node_count;
        
        // TODO: this number is absurdly large, the more instance_transforms, the more likely they are further away,
        //       hence only a few nodes per instance will be used and there are no way, space for the whole bottom
        //       level will be need.
        let brick_count = instance_transforms.len() * node_count as usize;
        
        let record = self.svos_to_render.entry(*id)
            .and_modify(|rec|  {
                info!("{:?}: Modifying SVO with {} instances, nodes: {}, potential bricks: {}", id, instance_transforms.len(), node_count, brick_count);
                rec.render = true;
                rec.domain = domain;
                rec.brick_atlas_stride = brick_atlas_stride;
                rec.brick_voxel_size = brick_voxel_size;
                rec.brick_scale = brick_scale;
                rec.brick_instance_buffer.clear_resize(gpu, brick_count);
                rec.instance_buffer.update(gpu, instance_transforms);
                if rec.instance_buffer.update(gpu, instance_transforms) {
                    rec.instance_bind_group = rec.instance_buffer.create_bind_group(gpu, &self.instance_bind_group_layout);
                }
            })
            .or_insert_with(|| {
                info!("Adding SVO {:?} with {} instances, nodes: {}, potential bricks: {}", id, instance_transforms.len(), node_count, brick_count);
                let brick_instance_buffer = BrickInstances::new(gpu, brick_count);
                let instance_buffer = GPUGeometryTransforms::from_transforms(gpu, instance_transforms);
                let instance_bind_group = instance_buffer.create_bind_group(gpu, &self.instance_bind_group_layout);
                let node_pool_bind_group = svo.node_pool.create_bind_group(gpu, &self.node_pool_bind_group_layout);
                let brick_pool_bind_group = svo.brick_pool.create_read_bind_group(gpu, &self.brick_pool_bind_group_layout);
                SVORenderRecord {
                    render: true,
                    domain,
                    brick_atlas_stride,
                    brick_voxel_size,
                    brick_scale,
                    instance_buffer,
                    instance_bind_group,
                    brick_instance_buffer,
                    node_pool_bind_group,
                    brick_pool_bind_group,
                }
            });
        (&record.instance_buffer, &record.brick_instance_buffer)
    }
    
    pub fn set_display_options(&mut self, options: DisplayOptions) {
        self.display_options = options;
    }
    
    /// This function loads all counter buffers by issuing async map requests and then waiting for them to complete.
    /// TODO: This is a performance bottleneck. Indirect draw calls would be a better solution and no buffer mapping would be required.
    #[profiler::function]
    pub fn load_counts(&mut self, gpu: &gpu::Context) {
        let mut slices: Vec<(GeometryID, wgpu::BufferSlice)> = Vec::with_capacity(self.svos_to_render.len());
        {
            profiler::scope!("Issuing buffer map requests");
            for (id, record) in self.svos_to_render.iter_mut() {
                if !record.render {
                    continue;
                }
                let slice = record.brick_instance_buffer.count_buffer.buffer.slice(..);
                slice.map_async(wgpu::MapMode::Read, move |_| ());
                slices.push((*id, slice));
            }
        }
        
        {
            profiler::scope!("Waiting for buffer map requests");
            gpu.device.poll(wgpu::Maintain::Wait);
        }
        
        let counts = {
            profiler::scope!("Reading buffer map requests");
            slices
                .drain(..)
                .map(|(id, slice)| {
                    let view = slice.get_mapped_range();
                    let data: Vec<u32> = bytemuck::cast_slice(&view).to_vec();
                    (id, data[0])
                })
                .collect::<Vec<_>>()
        };
        
        counters::sample!("brick_selected_counter", counts.iter().map(|(_, cnt)| cnt).sum::<u32>() as f64);
        
        {
            profiler::scope!("Unmapping buffers and storing counts");
            for (id, count) in counts.iter() {
                let record = self.svos_to_render.get_mut(id).unwrap();
                if !record.render {
                    continue;
                }
                record.brick_instance_buffer.count = Some(*count);
                record.brick_instance_buffer.count_buffer.buffer.unmap();
            }
        }
        
    }
    
    /// Runs this pipeline for given render pass
    #[profiler::function]
    pub fn render_on_pass<'rpass>(
        &'rpass self,
        pass: &mut wgpu::RenderPass<'rpass>,
        context: &RenderContext,
    ) {
        // common pass setup
        pass.set_vertex_buffer(0, self.cube_solid_mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(self.cube_solid_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        for (id, record) in self.svos_to_render.iter() {
            // skip not submitted svos
            if !record.render {
                continue;
            }
            
            let Some(instance_count) = record.brick_instance_buffer.count else {
                error!("Count for brick instance buffer is not loaded.");
                continue;
            };
            
            info!("Rendering SVO {:?} with {} instances, buffer capacity: {}",
                id,
                instance_count,
                record.brick_instance_buffer.buffer.capacity
            );
            
            pass.set_pipeline(&self.pipeline);
            
            let pc = PushConstants {
                view_projection:    context.camera.view_projection_matrix,
                camera_position:    glam::Vec4::from((context.camera.transform.position, 1.0)),
                focal_length:       context.camera.camera.focal_length(),
                domain:             record.domain,
                brick_scale:        record.brick_scale,
                brick_atlas_stride: record.brick_atlas_stride,
                brick_voxel_size:   record.brick_voxel_size,
                display_options:    self.display_options,
                ..Default::default()
            };
            
            pass.set_push_constants(wgpu::ShaderStages::VERTEX_FRAGMENT, 0, bytemuck::cast_slice(&[pc]));
            
            pass.set_vertex_buffer(1, record.brick_instance_buffer.buffer.buffer.slice(..));
            
            pass.set_bind_group(0, &record.node_pool_bind_group, &[]);
            pass.set_bind_group(1, &record.brick_pool_bind_group, &[]);
            pass.set_bind_group(2, &record.instance_bind_group, &[]);
            
            // TODO: use indirect to avoid pulling instance buffer count from gpu
            pass.draw_indexed(
                0..CUBE_INDICES_TRIANGLE_STRIP.len() as u32,
                0,
                0..instance_count.max(1)
            );
        }
    }
}
