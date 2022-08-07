use anyhow::Result;
use std::num::NonZeroU32;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    #[profiler::function]
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: Option<&str>) -> Result<Self> {
        let img = profiler::call!(
            image::load_from_memory(include_bytes!("../resources/textures/happy-tree.png"))
                .expect("Failed fo load texture image.")
        );
        return Self::from_image(device, queue, &img, label);
    }
    
    #[profiler::function]
    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: &image::DynamicImage, label: Option<&str>) -> Result<Self> {
        let dimensions = wgpu::Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1, // 2D texture is just special case of flat 3d texture
        };
        
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size: dimensions,
                mip_level_count: 1, // we do not do that for now
                sample_count: 1, // for multisampling
                dimension: wgpu::TextureDimension::D2, // now we tell gpu to use 2d texture sampler for this
                format: wgpu::TextureFormat::Rgba8UnormSrgb, // the original png image had probably sRGB color space hence sampler must cope with that and apply some color grading
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );
        
        let rgba8 = profiler::call!(img.to_rgba8());
        
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        
        Ok(Self{ texture, view, sampler })
    }
}
