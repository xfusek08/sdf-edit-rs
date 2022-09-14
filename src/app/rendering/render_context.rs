use winit::window::Window;

use super::camera::CameraRenderResource;

pub struct RenderContext {
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    
    // shared render resources
    pub camera: CameraRenderResource,
    pub scale_factor: f64,
}

impl RenderContext {
    
    #[profiler::function]
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        
        let surface = unsafe { instance.create_surface(window) };
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        ).await.expect("Failed to find an appropriate adapter");
        
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
        
        let camera = CameraRenderResource::new(&device);
        
        RenderContext {
            surface,
            surface_config,
            device,
            queue,
            camera,
            scale_factor: window.scale_factor(),
        }
    }
    
}
