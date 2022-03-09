use image::GenericImageView;
use crate::*;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Bound;
use std::ops::RangeBounds;

pub trait IntoExtent3D{
    fn into_extent_3d(self) -> wgpu::Extent3d;
}

impl IntoExtent3D for [u32; 2]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0],
            height: self[1],
            depth_or_array_layers: 1,
        }
    }
}

impl IntoExtent3D for [u32; 3]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0],
            height: self[1],
            depth_or_array_layers: self[2],
        }
    }
}

impl IntoExtent3D for [usize; 2]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0] as u32,
            height: self[1] as u32,
            depth_or_array_layers: 1,
        }
    }
}

impl IntoExtent3D for [usize; 3]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0] as u32,
            height: self[1] as u32,
            depth_or_array_layers: self[2] as u32,
        }
    }
}

impl IntoExtent3D for [i32; 2]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0] as u32,
            height: self[1] as u32,
            depth_or_array_layers: 1,
        }
    }
}

impl IntoExtent3D for [i32; 3]{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        wgpu::Extent3d{
            width: self[0] as u32,
            height: self[1] as u32,
            depth_or_array_layers: self[2] as u32,
        }
    }
}

impl IntoExtent3D for wgpu::Extent3d{
    #[inline]
    fn into_extent_3d(self) -> wgpu::Extent3d {
        self
    }
}

///
/// 
///
pub struct Texture{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
}

pub struct TextureSlice<'ts>{
    texture: &'ts Texture,
    origin: wgpu::Origin3d,
    extent: wgpu::Extent3d,
}

impl<'ts> TextureSlice<'ts>{
    pub fn copy_to_texture(&self, encoder: &mut wgpu::CommandEncoder, dst: &Texture, offset: wgpu::Origin3d){
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture{
                texture: &self.texture.texture,
                mip_level: 0,
                origin: self.origin,
                aspect: wgpu::TextureAspect::All
            },
            wgpu::ImageCopyTexture{
                texture: &dst.texture,
                mip_level: 0,
                origin: offset,
                aspect: wgpu::TextureAspect::All,
            },
            self.extent,
        );
    }

    pub fn copy_to_buffer<C: bytemuck::Pod>(&self, encoder: &mut wgpu::CommandEncoder, dst: &mut Buffer<C>, offset: wgpu::BufferAddress){
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture{
                texture: &self.texture.texture,
                mip_level: 0,
                origin: self.origin,
                aspect: wgpu::TextureAspect::All
            },
            wgpu::ImageCopyBuffer{
                buffer: &dst.buffer,
                layout: wgpu::ImageDataLayout{
                    offset,
                    bytes_per_row: std::num::NonZeroU32::new(self.texture.size.width * self.texture.format.describe().block_size as u32),
                    rows_per_image: std::num::NonZeroU32::new(self.texture.size.height),
                }
            },
            self.extent
        );
    }

    pub fn to_image(&self, device: &wgpu::Device) -> image::DynamicImage{
        let o_buf = BufferBuilder::new()
            .copy_dst()
            .read()
            .build_empty(device, std::mem::size_of::<u32>() * self.extent.width as usize * self.extent.height as usize);

        let buf_view = o_buf.slice(..).map_blocking(device);

        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(
                self.extent.width, self.extent.height,
                Vec::from(buf_view.as_ref())
        ).unwrap());
        image
    }
}

pub struct TextureBuilder<'tb>{
    pub data: Option<Vec<u8>>,
    pub size: wgpu::Extent3d,
    pub sampler_descriptor: wgpu::SamplerDescriptor<'tb>,
    pub usage: wgpu::TextureUsages,
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureDimension,
    pub label: wgpu::Label<'tb>,
}

impl<'tb> TextureBuilder<'tb>{

    pub fn new() -> Self{
        let sampler_descriptor = wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
        };
        
        let usage = wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT;

        // Default format
        let format = wgpu::TextureFormat::Rgba8Unorm;

        let dimension = wgpu::TextureDimension::D2;

        Self{
            data: None,
            size: wgpu::Extent3d::default(),
            sampler_descriptor,
            usage,
            format,
            dimension,
            label: None,
        }
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self{
        self.format = format;
        self
    }

    pub fn from_raw(mut self, data: Vec<u8>, size: wgpu::Extent3d) -> Self{
        self.data = Some(data);
        self.size = size;
        self
    }

    pub fn from_image(mut self, img: &image::DynamicImage) -> Self{
        let img_data: Vec<u8> = match self.format{
            wgpu::TextureFormat::Rgba8Unorm     => img.flipv().to_rgba8().into_raw(),
            wgpu::TextureFormat::Rgba8UnormSrgb => img.flipv().to_rgba8().into_raw(),
            wgpu::TextureFormat::Bgra8Unorm     => img.flipv().to_bgra8().into_raw(),
            wgpu::TextureFormat::Bgra8UnormSrgb => img.flipv().to_bgra8().into_raw(),
            _ => {
                panic!("TextureFormat not supported")
            }
        };
        let dims = img.dimensions();

        let extent = wgpu::Extent3d{
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        self.data = Some(img_data);
        self.size = extent;
        self
    }

    pub fn from_bytes(self, bytes: &[u8]) -> Self{
        let img = image::load_from_memory(bytes).unwrap();
        Self::from_image(self, &img)
    }

    pub fn load_from_path(self, path: &std::path::Path) -> Self{
        let buffer = fs::read(path).unwrap();
        Self::from_bytes(self, &buffer)
    }

    pub fn clear<Z: IntoExtent3D>(mut self, size: Z) -> Self{
        self.size = size.into_extent_3d();
        self.data = None;
        self
    }

    pub fn label(mut self, label: wgpu::Label<'tb>) -> Self{
        self.label = label;
        self
    }

    pub fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> Texture{
        let texture = device.create_texture(
            &wgpu::TextureDescriptor{
                label: self.label,
                size: self.size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: self.usage
            }
        );
        let texture_view_desc = wgpu::TextureViewDescriptor{
            format: Some(self.format),
            ..Default::default()
        };
        let view = texture.create_view(&texture_view_desc);
        let sampler = device.create_sampler(
            &self.sampler_descriptor
        );

        if let Some(data) = &self.data{
            queue.write_texture(
                wgpu::ImageCopyTexture{
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::ImageDataLayout{
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * self.size.width),
                    rows_per_image: std::num::NonZeroU32::new(self.size.height),
                },
                self.size,
            );
        }

        Texture{
            texture,
            view,
            sampler,
            format: self.format,
            size: self.size,
        }
    }

    pub fn build_bind_group(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> BindGroup<Texture>{
        BindGroup::new(self.build(device, queue), device)
    }

    pub fn build_bind_group_vis(&mut self, visibility: wgpu::ShaderStages, device: &wgpu::Device, queue: &wgpu::Queue) -> BindGroup<Texture>{
        BindGroup::new_vis(self.build(device, queue), device, visibility)
    }

}

impl Texture{
    pub fn slice<'ts, S: RangeBounds<u32>>(&'ts self, bound_x: S, bound_y: S, bound_z: S) -> TextureSlice<'ts>{
        let range_x = {
            let start_bound = match bound_x.start_bound(){
                Bound::Unbounded => 0,
                Bound::Included(x) => {x + 0},
                Bound::Excluded(x) => {x + 1},
            };
            let end_bound = match bound_x.end_bound(){
                Bound::Unbounded => self.size.width as u32,
                Bound::Included(x) => {(x + 1).max(self.size.width)},
                Bound::Excluded(x) => {(x + 0).max(self.size.width)},
            };
            start_bound..end_bound
        };
        let range_y = {
            let start_bound = match bound_y.start_bound(){
                Bound::Unbounded => 0,
                Bound::Included(x) => {x + 0},
                Bound::Excluded(x) => {x + 1},
            };
            let end_bound = match bound_y.end_bound(){
                Bound::Unbounded => self.size.height as u32,
                Bound::Included(x) => {(x + 1).max(self.size.height)},
                Bound::Excluded(x) => {(x + 0).max(self.size.height)},
            };
            start_bound..end_bound
        };
        let range_z = {
            let start_bound = match bound_z.start_bound(){
                Bound::Unbounded => 0,
                Bound::Included(x) => {x + 0},
                Bound::Excluded(x) => {x + 1},
            };
            let end_bound = match bound_z.end_bound(){
                Bound::Unbounded => self.size.depth_or_array_layers as u32,
                Bound::Included(x) => {(x + 1).max(self.size.depth_or_array_layers)},
                Bound::Excluded(x) => {(x + 0).max(self.size.depth_or_array_layers)},
            };
            start_bound..end_bound
        };

        let origin = wgpu::Origin3d{
            x: range_x.start,
            y: range_y.start,
            z: range_z.start,
        };

        let extent = wgpu::Extent3d{
            width: range_x.end - range_x.start,
            height: range_y.end - range_y.start,
            depth_or_array_layers: range_z.end - range_z.start,
        };

        TextureSlice{
            texture: self,
            origin,
            extent,
        }
    }
}

// TODO: decide on weather to use struct initialisation or function initialisation.
impl BindGroupContent for Texture{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<binding::BindGroupLayoutEntry>{
        vec!{
            BindGroupLayoutEntry{
                visibility,
                ty: binding::wgsl::texture_2d(),
                count: None,
            },
            BindGroupLayoutEntry{
                visibility,
                ty: binding::wgsl::sampler(),
                count: None,
            }
        }
    }

    fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>> {
        vec!{
            wgpu::BindingResource::TextureView(&self.view),
            wgpu::BindingResource::Sampler(&self.sampler),
        }
    }
}

pub type BindGroupTexture = BindGroup<Texture>;

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

impl ColorAttachment for imgui_wgpu::Texture{
    fn color_attachment_clear(&self) -> wgpu::RenderPassColorAttachment {
        self.view().color_attachment_clear()
    }

    fn color_attachment_clear_with(&self, color: wgpu::Color) -> wgpu::RenderPassColorAttachment {
        self.view().color_attachment_clear_with(color)
    }

    fn color_attachment_load(&self) -> wgpu::RenderPassColorAttachment {
        self.view().color_attachment_load()
    }
}
