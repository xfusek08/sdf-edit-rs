use winit::window::Window;

#[derive(Debug)]
pub struct Context {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device:  wgpu::Device,
    pub queue:   wgpu::Queue,
}

impl Context {
    
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
        
        let (device, queue) = Self::new_device_queue(&adapter).await;
        
        Self {
            adapter,
            surface,
            device,
            queue,
        }
    }
    
    #[profiler::function]
    pub async fn new_device_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features:
                    wgpu::Features::PUSH_CONSTANTS // To allow push constants
                    | wgpu::Features::POLYGON_MODE_LINE // to allow wireframe rendering
                    | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS // to allow mapping of primary buffers to memory
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES // to allow sampling storage textures see: https://github.com/gfx-rs/wgpu/issues/1412 and https://github.com/gfx-rs/wgpu-rs/issues/877#issuecomment-826896142
                ,
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    max_compute_invocations_per_workgroup: 512, // to allow 8x8x8 workgroups
                    ..Default::default()
                },
            },
            None
        ).await.expect("Failed to create device")
    }
    
}