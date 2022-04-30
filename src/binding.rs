#[allow(unused)]
use anyhow::*;
use std::{ops::{Deref, DerefMut}, marker::PhantomData};

pub trait CreateBindGroupLayout {
    fn create_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> BindGroupLayoutWithDesc;
}

pub trait GetBindGroupLayout {
    fn bind_group_layout(&self) -> &BindGroupLayoutWithDesc;
}

///
/// A trait implemented for structs that have a BindGroup.
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
/// A trait implemented for structs that can be the content of a BindGroup.
///
pub trait BindGroupContent: Sized {
    fn entries(visibility: Option<wgpu::ShaderStages>) -> Vec<BindGroupLayoutEntry>;
    fn resources(&self) -> Vec<wgpu::BindingResource>;
    fn into_bound(self, device: &wgpu::Device) -> Bound<Self> {
        Bound{
            bind_group: Self::create_bind_group(&self, device),
            content: self,
        }
    }
    fn create_bind_group(&self, device: &wgpu::Device) -> BindGroup<Self>{
        let layout =
            Self::create_bind_group_layout(device, None);
        let resources = self.resources();

        let entries: Vec<wgpu::BindGroupEntry> = resources
            .into_iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry {
                binding: layout.entries[i].binding,
                resource: r,
            })
            .collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &entries,
            layout: &layout.layout,
        });
        BindGroup{
            bind_group_layout: layout,
            bind_group,
            _ty: PhantomData,
        }
    }
    fn create_bind_group_layout(
        device: &wgpu::Device,
        label: wgpu::Label,
    ) -> BindGroupLayoutWithDesc {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = Self::entries(None)
            .iter()
            .enumerate()
            .map(|(i, x)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                ty: x.ty,
                count: x.count,
                visibility: x.visibility,
            })
            .collect();

        BindGroupLayoutWithDesc {
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &entries,
                label,
            }),
            entries,
        }
    }
}

// TODO: Derive macro for BindGroupContent.

macro_rules! bind_group_content_for_tuple{
    ($($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($name: BindGroupContent),+> BindGroupContent for ($($name, )+){
            fn entries(visibility: Option<wgpu::ShaderStages>) -> Vec<BindGroupLayoutEntry>{
                let mut ret = Vec::new();
                {
                    $(
                        ret.append(&mut $name::entries(visibility));
                    )+
                }
                ret
            }
            fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>>{
                let ($($name, )+) = self;
                let mut ret = Vec::new();
                {
                    $(
                        ret.append(&mut $name.resources());
                    )+
                }
                ret
            }
        }
    }
}

// TODO: add derive macro for structs
bind_group_content_for_tuple! { A }
bind_group_content_for_tuple! { A B }
bind_group_content_for_tuple! { A B C }
bind_group_content_for_tuple! { A B C D }
bind_group_content_for_tuple! { A B C D E }
bind_group_content_for_tuple! { A B C D E F }
bind_group_content_for_tuple! { A B C D E F G }
bind_group_content_for_tuple! { A B C D E F G H }
bind_group_content_for_tuple! { A B C D E F G H I }
bind_group_content_for_tuple! { A B C D E F G H I J }
bind_group_content_for_tuple! { A B C D E F G H I J K }
bind_group_content_for_tuple! { A B C D E F G H I J K L }

impl<C: BindGroupContent, const N: usize> BindGroupContent for [C; N] {
    fn entries(visibility: Option<wgpu::ShaderStages>) -> Vec<BindGroupLayoutEntry> {
        let mut ret = Vec::with_capacity(N);
        for _i in 0..N {
            ret.append(&mut C::entries(visibility));
        }
        ret
    }

    fn resources(&self) -> Vec<wgpu::BindingResource> {
        let mut ret = Vec::with_capacity(N);
        for content in self {
            ret.append(&mut content.resources());
        }
        ret
    }
}

pub struct Bound<C: BindGroupContent>{
    pub content: C,
    bind_group: BindGroup<C>,
}

impl<C: BindGroupContent> Bound<C> {
    ///
    /// Has to be called whenever a buffer is changed for example when expand_to_clear is called on
    /// it.
    /// TODO: Change this requirement.
    ///
    pub fn update(&mut self, device: &wgpu::Device) {
        self.bind_group.update(&self.content, device);
    }
}

impl<C: BindGroupContent> CreateBindGroupLayout for Bound<C> {
    fn create_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> BindGroupLayoutWithDesc {
        C::create_bind_group_layout(device, label)
    }
}

impl<C: BindGroupContent> Deref for Bound<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C: BindGroupContent> DerefMut for Bound<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<C: BindGroupContent> GetBindGroupLayout for Bound<C> {
    fn bind_group_layout(&self) -> &BindGroupLayoutWithDesc {
        &self.bind_group.bind_group_layout()
    }
}

impl<C: BindGroupContent> GetBindGroup for Bound<C> {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[cfg(feature = "imgui")]
impl From<Bound<super::Texture>> for imgui_wgpu::Texture {
    fn from(bg_texture: Bound<super::Texture>) -> Self {
        imgui_wgpu::Texture::from_raw_parts(
            bg_texture.content.texture,
            bg_texture.content.view,
            bg_texture.bind_group.bind_group,
            bg_texture.content.size,
        )
    }
}

pub struct BindGroup<C: BindGroupContent>{
    _ty: PhantomData<C>,
    bind_group: wgpu::BindGroup,
    bind_group_layout: BindGroupLayoutWithDesc,
}

impl<C: BindGroupContent> BindGroup<C>{
    pub fn update(&mut self, conent: &C, device: &wgpu::Device) {
        *self = conent.create_bind_group(device)
    }
}

impl<C: BindGroupContent> CreateBindGroupLayout for BindGroup<C> {
    fn create_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> BindGroupLayoutWithDesc {
        C::create_bind_group_layout(device, label)
    }
}

impl<C: BindGroupContent> Deref for BindGroup<C> {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.bind_group
    }
}

impl<C: BindGroupContent> DerefMut for BindGroup<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bind_group
    }
}

impl<C: BindGroupContent> GetBindGroupLayout for BindGroup<C> {
    fn bind_group_layout(&self) -> &BindGroupLayoutWithDesc {
        &self.bind_group_layout
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

    pub fn sampler() -> wgpu::BindingType {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }

    pub fn texture_2d() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }
}
