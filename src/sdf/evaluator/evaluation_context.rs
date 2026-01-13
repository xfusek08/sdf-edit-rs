use crate::{
    framework::gpu,
    sdf::{geometry, svo},
};

pub struct EvaluationContext {
    pub svo: svo::Svo,
    pub edits: geometry::GPUEdits,
    pub layouts: EvaluationContextLayouts,
    pub bind_groups: EvaluationContextBindGroups,
}

pub struct EvaluationContextLayouts {
    pub node_pool: wgpu::BindGroupLayout,
    pub brick_pool: wgpu::BindGroupLayout,
    pub edits: wgpu::BindGroupLayout,
}

pub struct EvaluationContextBindGroups {
    pub node_pool: wgpu::BindGroup,
    pub brick_pool: wgpu::BindGroup,
    pub edits: wgpu::BindGroup,
}

impl EvaluationContext {
    #[profiler::function]
    pub fn new(gpu: &gpu::Context, svo: svo::Svo, edits: geometry::GPUEdits) -> Self {
        let layouts = EvaluationContextLayouts::new(gpu);
        let bind_groups = EvaluationContextBindGroups {
            node_pool: svo.node_pool.create_bind_group(gpu, &layouts.node_pool),
            brick_pool: svo
                .brick_pool
                .create_write_bind_group(gpu, &layouts.brick_pool),
            edits: edits.create_bind_group(gpu, &layouts.edits),
        };
        Self {
            svo,
            edits,
            layouts,
            bind_groups,
        }
    }
}

impl EvaluationContextLayouts {
    #[profiler::function]
    pub fn new(gpu: &gpu::Context) -> Self {
        Self {
            node_pool: svo::NodePool::create_bind_group_layout(
                gpu,
                wgpu::ShaderStages::COMPUTE,
                false,
            ),
            brick_pool: svo::BrickPool::create_write_bind_group_layout(
                gpu,
                wgpu::ShaderStages::COMPUTE,
            ),
            edits: geometry::GPUEdits::create_bind_group_layout(gpu, wgpu::ShaderStages::COMPUTE),
        }
    }
}
