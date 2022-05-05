use super::pipeline::*;
#[allow(unused)]
use anyhow::*;
use std::marker::PhantomData;

pub trait GetBindGroupLayout {
    fn bind_group_layout(&self) -> &BindGroupLayoutWithDesc;
}

///
/// A trait implemented for structs that have a BindGroup.
/// TODO: evaluate usefullness.
///
pub trait GetBindGroup {
    fn bind_group(&self) -> &wgpu::BindGroup;
}

impl GetBindGroup for wgpu::BindGroup {
    fn bind_group(&self) -> &wgpu::BindGroup {
        self
    }
}

pub struct BindGroupLayoutWithDesc {
    pub layout: wgpu::BindGroupLayout,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

pub struct BindGroupLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<std::num::NonZeroU32>,
}

impl BindGroupLayoutEntry {
    pub fn new(visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self {
        Self {
            visibility,
            ty,
            count: None,
        }
    }
}

///
/// A trait that should be implemented by any struct that can be bound to a pipeline.
///
/// example: 
///
/// ```rust 
///
/// #[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
/// pub struct LineVert{
///     pub pos: [f32; 4],
///     pub color: [f32; 4],
///     pub width: [f32; 4],
/// }
///
/// #[derive(BindGroupContent)]
/// pub struct Line{
///     pub indices: Buffer<u32>,
///     pub vertices: Buffer<LineVert>,
/// }
///
/// impl BindGroupLayout for Line{
///     fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout{
///         device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
///             label: Some("Line"),
///             entries: &[
///                 wgpu::BindGroupLayoutEntry{
///                     binding: 0,
///                     ..wgsl::buffer_entry(false)
///                 },
///                 wgpu::BindGroupLayoutEntry{
///                     binding: 0,
///                     ..wgsl::buffer_entry(false)
///                 }
///             ]
///         })
///     }
/// }
/// ```
///
pub trait BindGroupLayout: BindGroupContent {
    // TODO: macro for simpler generation of BindGroupLayouts.
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
}

///
/// Trait implemented by Resources that can be bound to a pipeline.
/// For example: Texture, Buffer, Uniform.
///
pub trait BindingResource{
    fn resource(&self) -> wgpu::BindingResource;
}

///
/// Shorthand functions to convert bind_group contents into bind_groups or bind them.
///
pub trait IntoBindGroup: BindGroupLayout + BindGroupContent{
    fn into_bound(self, device: &wgpu::Device) -> Bound<Self> {
        self.into_bound_with(device, &Self::bind_group_layout(device))
    }
    fn to_bind_group(&self, device: &wgpu::Device) -> BindGroup<Self> {
        self.into_bind_group(device, &Self::bind_group_layout(device))
    }
}
impl<C: BindGroupLayout + BindGroupContent> IntoBindGroup for C{
}

///
/// A trait implemented for structs that can be the content of a BindGroup.
///
pub trait BindGroupContent: Sized {
    fn resources(&self) -> Vec<wgpu::BindingResource>;
    fn into_bound_with(self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Bound<Self> {
        Bound {
            bind_group: self.into_bind_group(device, layout),
            content: self,
        }
    }
    fn into_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> BindGroup<Self> {
        let resources = self.resources();

        let entries: Vec<wgpu::BindGroupEntry> = resources
            .into_iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: r,
            })
            .collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &entries,
            layout: &layout,
        });
        BindGroup {
            bind_group,
            _ty: PhantomData,
        }
    }
}

#[derive(DerefMut)]
pub struct Bound<C: BindGroupContent> {
    #[target]
    pub content: C,
    bind_group: BindGroup<C>,
}

impl<C: BindGroupContent> Bound<C>{
    pub fn with_offsets<'a>(&'a self, offsets: &'a [u32]) -> BindGroupWithOffsets{
        BindGroupWithOffsets{
            bind_group: self.bind_group(),
            offsets,
        }
    }
}


impl<C: BindGroupContent> GetBindGroup for Bound<C> {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/*
#[cfg(feature = "imgui")]
impl From<Bound<super::Texture>> for imgui_wgpu::Texture {
    fn from(bg_texture: Bound<(super::TextureView, super::Texture)>) -> Self {
        imgui_wgpu::Texture::from_raw_parts(
            bg_texture.content.1.texture,
            bg_texture.content.0.view,
            bg_texture.bind_group.1.bind_group,
            bg_texture.content.1.size,
        )
    }
}
*/

#[derive(DerefMut)]
pub struct BindGroup<C: BindGroupContent> {
    _ty: PhantomData<C>,
    #[target]
    bind_group: wgpu::BindGroup,
}

impl<C: BindGroupContent> BindGroup<C>{
    pub fn from_wgpu(bind_group: wgpu::BindGroup) -> Self{
        Self{
            _ty: PhantomData,
            bind_group,
        }
    }
    pub fn with_offsets<'a>(&'a self, offsets: &'a [u32]) -> BindGroupWithOffsets{
        BindGroupWithOffsets{
            bind_group: self,
            offsets,
        }
    }
}

impl<C: BindGroupContent> GetBindGroup for BindGroup<C> {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[allow(dead_code)]
mod glsl {
    pub fn buffer(read_only: bool) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn uniform() -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn sampler(filtering: bool) -> wgpu::BindingType {
        if filtering {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        } else {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering)
        }
    }

    #[allow(non_snake_case)]
    pub fn texture2D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn texture2DArray() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn itexture2D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Sint,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn utexture2D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn texture3D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn itexture3D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Sint,
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn utexture3D() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn textureCube() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::Cube,
            multisampled: false,
        }
    }

    #[allow(non_snake_case)]
    pub fn image2D(
        format: wgpu::TextureFormat,
        access: wgpu::StorageTextureAccess,
    ) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D2,
            format,
        }
    }

    #[allow(non_snake_case)]
    pub fn image2DArray(
        format: wgpu::TextureFormat,
        access: wgpu::StorageTextureAccess,
    ) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D2Array,
            format,
        }
    }

    #[allow(non_snake_case)]
    pub fn image3D(
        format: wgpu::TextureFormat,
        access: wgpu::StorageTextureAccess,
    ) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D3,
            format,
        }
    }
}

pub mod wgsl {
    pub const fn buffer(read_only: bool) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub const fn buffer_entry(read_only: bool) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub const fn uniform() -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub const fn uniform_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub const fn sampler() -> wgpu::BindingType {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }

    pub const fn sampler_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    pub const fn texture_2d() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }

    pub const fn texture_2d_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }
}
