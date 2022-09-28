use winit::window::Window;

#[derive(Debug)]
pub struct GPUContext {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device:  wgpu::Device,
    pub queue:   wgpu::Queue,
}

impl GPUContext {
    
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        let instance = profiler::call!(
            wgpu::Instance::new(wgpu::Backends::VULKAN)
        );
        
        let surface = profiler::call!(
            unsafe { instance.create_surface(window) }
        );
        
        let adapter = profiler::call!(
            instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                }
            ).await.expect("Failed to find an appropriate adapter")
        );
        
        let (device, queue) = profiler::call!(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                },
                None
            ).await.expect("Failed to create device")
        );
        
        GPUContext {
            adapter,
            surface,
            device,
            queue,
        }
    }
    
}
