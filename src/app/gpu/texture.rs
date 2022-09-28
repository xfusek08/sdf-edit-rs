
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    #[profiler::function]
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: Option<&str>) -> Self {
        let img = profiler::call!(image::load_from_memory(bytes).expect("Failed fo load texture image."));
        return Self::from_image(device, queue, &img, label);
    }
    
    #[profiler::function]
    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: &image::DynamicImage, label: Option<&str>) -> Self {
        let dimensions = wgpu::Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1, // 2D texture is just special case of flat 3d texture
        };
        
        let texture = profiler::call!(
            device.create_texture(
                &wgpu::TextureDescriptor {
                    label,
                    size: dimensions,
                    mip_level_count: 1, // we do not do that for now
                    sample_count: 1, // for multisampling
                    dimension: wgpu::TextureDimension::D2, // now we tell gpu to use 2d texture sampler for this
                    format: wgpu::TextureFormat::Rgba8UnormSrgb, // the original png image had probably sRGB color space hence sampler must cope with that and apply some color grading
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                }
            )
        );
        
        let rgba8 = profiler::call!(
            img.to_rgba8()
        );
        
        // copy data from cpu to gpu
        profiler::call!(
            queue.write_texture(
                
                // This target of byte transfer with configuration what will be copied
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All
                },
                
                // this is source of bytes
                &rgba8,
                
                // how to read data from source
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * img.width()),
                    rows_per_image: NonZeroU32::new(img.height()),
                },
                
                dimensions
            )
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = profiler::call!(
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..wgpu::SamplerDescriptor::default()
            })
        );
        
        Self{ texture, view, sampler }
    }
    
}

/// A construction of depth buffer texture according to: https://sotrh.github.io/learn-wgpu/beginner/tutorial8-depth/#a-pixels-depth
#[derive(Debug)]
pub struct DepthStencilTexture {
    texture: Texture,
}

impl DepthStencilTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    
    #[profiler::function]
    pub fn new(label: &str, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let texture = profiler::call!(
            device.create_texture(&wgpu::TextureDescriptor {
                format: Self::DEPTH_FORMAT,
                label: Some(label),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            })
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );
        
        Self {
            texture: Texture { texture, view, sampler },
        }
    }
    
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
    
    pub fn stencil() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: Self::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }
}
