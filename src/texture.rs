use image::GenericImageView;
use anyhow::*;
use super::render_target::*;
use super::binding;
use super::binding::*;
use std::fs;
use std::fs::File;
use std::io::Read;


///
/// 
///
pub struct Texture{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
    pub size: [u32; 2],

}

impl Texture{
    pub fn load_from_path(
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        path: &str,
        label: Option<&str>,
        format: wgpu::TextureFormat,
    ) -> Result<Self>{
        let mut f = File::open(path)?;
        let metadata = fs::metadata(path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer)?;
        Self::from_bytes(
            device,
            queue,
            &buffer,
            label,
            format
        )
    }
    pub fn new_black(
        size: [u32; 2],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let data: Vec<u8> = vec![0; (size[0] * size[1] * 4) as usize];

        let extent = wgpu::Extent3d{
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label,
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
        );
        let texture_view_desc = wgpu::TextureViewDescriptor{
            format: Some(format),
            ..Default::default()
        };
        let view = texture.create_view(&texture_view_desc);
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Ok(Self{
            texture,
            view,
            sampler,
            format,
            size,
        })
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let img_data: Vec<u8> = match format{
            wgpu::TextureFormat::Rgba8Unorm     => img.flipv().to_rgba8().into_raw(),
            wgpu::TextureFormat::Rgba8UnormSrgb => img.flipv().to_rgba8().into_raw(),
            wgpu::TextureFormat::Bgra8Unorm     => img.flipv().to_bgra8().into_raw(),
            wgpu::TextureFormat::Bgra8UnormSrgb => img.flipv().to_bgra8().into_raw(),
            _ => {
                return Err(anyhow!("Format not supported"));
            }
        };

        let dims = img.dimensions();

        let extent = wgpu::Extent3d{
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label,
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture{
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &img_data,
            wgpu::ImageDataLayout{
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dims.0),
                rows_per_image: std::num::NonZeroU32::new(dims.1),
            },
            extent,
        );
        let texture_view_desc = wgpu::TextureViewDescriptor{
            format: Some(format),
            ..Default::default()
        };

        let view = texture.create_view(&texture_view_desc);
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        let size = [dims.0, dims.1];

        Ok(Self{
            texture,
            view,
            sampler,
            format,
            size,
        })
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, label, format)
    }

    pub fn copy_all_to(&self, dst: &mut Texture, encoder: &mut wgpu::CommandEncoder){
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture{
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            wgpu::ImageCopyTexture{
                texture: &dst.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d{
                width: self.size[0],
                height: self.size[1],
                depth_or_array_layers: 1,
            }
        );
    }
}

impl RenderTarget for Texture{
    fn render_pass_clear<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        self.view.render_pass_clear(encoder, label)
    }
    fn render_pass_load<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, label: Option<&'a str>) -> Result<wgpu::RenderPass<'a>> {
        self.view.render_pass_load(encoder, label)
    }
}

impl BindGroupContent for Texture{
    fn push_entries_to(bind_group_layout_builder: &mut BindGroupLayoutBuilder) {
        bind_group_layout_builder.push_entry_all_ref(binding::wgsl::texture_2d());
        bind_group_layout_builder.push_entry_all_ref(binding::wgsl::sampler());
    }

    fn push_resources_to<'bgb>(&'bgb self, bind_group_builder: &mut BindGroupBuilder<'bgb>) {
        bind_group_builder.texture_ref(&self.view);
        bind_group_builder.sampler_ref(&self.sampler);
    }
}

pub type BindGroupTexture = BindGroup<Texture>;

impl BindGroupTexture{
    pub fn load_from_path(
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        path: &str,
        label: Option<&str>,
        format: wgpu::TextureFormat,
    ) -> Result<Self>{
        Ok(binding::BindGroup::new(Texture::load_from_path(
                    device,
                    queue,
                    path,
                    label,
                    format
        )?, device))
    }
    pub fn new_black(
        size: [u32; 2],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        Ok(binding::BindGroup::new(Texture::new_black(
                    size,
                    device,
                    queue,
                    label,
                    format
        )?, device))
    }
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        Ok(binding::BindGroup::new(Texture::from_image(
                    device,
                    queue,
                    img,
                    label,
                    format
        )?, device))
    }
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
        format: wgpu::TextureFormat
    ) -> Result<Self>{
        Ok(binding::BindGroup::new(Texture::from_bytes(
                    device,
                    queue,
                    bytes,
                    label,
                    format
        )?, device))
    }
}

impl ColorAttachment for Texture{
    fn color_attachment_clear(&self) -> wgpu::RenderPassColorAttachment {
        self.view.color_attachment_clear()
    }

    fn color_attachment_clear_with(&self, color: wgpu::Color) -> wgpu::RenderPassColorAttachment {
        self.view.color_attachment_clear_with(color)
    }

    fn color_attachment_load(&self) -> wgpu::RenderPassColorAttachment {
        self.view.color_attachment_load()
    }
}
