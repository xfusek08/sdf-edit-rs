
use std::sync::Arc;
use crate::framework::gpu;

use super::Camera;

#[derive(Debug)]
pub struct RenderContext {
    
    /// A GPU context which is shared with whole application
    pub gpu: Arc<gpu::Context>,
    
    /// Configuration of surface is renderers responsibility
    pub surface_config: wgpu::SurfaceConfiguration,
    
    /// A part of surface configuration
    pub scale_factor: f64,
    
    /// Shared GPU resources provided for all render modules
    
    /// A camera GPU resource
    pub camera: Camera,
    
}