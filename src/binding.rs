#[allow(unused)]
use anyhow::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::any::{Any, TypeId};

pub trait CreateBindGroupLayout{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc;
    fn create_bind_group_layout_vis(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> BindGroupLayoutWithDesc;
}

pub trait CreateBindGroup: CreateBindGroupLayout{
    fn create_bind_group(&self, device: &wgpu::Device, layout: &BindGroupLayoutWithDesc, label: Option<&str>) -> wgpu::BindGroup;
}

pub trait GetBindGroupLayout{
    fn get_bind_group_layout<'l>(&'l self) -> &'l BindGroupLayoutWithDesc;
}

pub trait GetBindGroup{
    fn get_bind_group<'l>(&'l self) -> &'l wgpu::BindGroup;
}


pub struct BindGroupLayoutWithDesc{
    pub layout: wgpu::BindGroupLayout,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

pub struct BindGroupLayoutEntry{
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<std::num::NonZeroU32>,
}

impl BindGroupLayoutEntry{
    pub fn new(visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self{
        Self{
            visibility,
            ty,
            count: None,
        }
    }
}

pub struct BindGroupLayoutBuilder{
    entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder{
    pub fn new() -> Self{
        Self{
            entries: Vec::new(),
        }
    }

    pub fn push_entries(mut self, mut entries: Vec<BindGroupLayoutEntry>) -> Self{
        self.entries.append(&mut entries);
        self
    }

    pub fn create(self, device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc{

        let entries: Vec<wgpu::BindGroupLayoutEntry> = self.entries.iter().enumerate().map(|(i, x)| {
            wgpu::BindGroupLayoutEntry{
                binding: i as u32,
                ty: x.ty,
                count: x.count,
                visibility: x.visibility,
            }
        }).collect();

        BindGroupLayoutWithDesc{
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
                entries: &entries,
                label,
            }),
            entries,
        }
    }
}

pub struct BindGroupBuilder<'l>{
    layout_with_desc: &'l BindGroupLayoutWithDesc,
    entries: Vec<wgpu::BindGroupEntry<'l>>,
}

impl<'l> BindGroupBuilder<'l>{
    pub fn new(layout_with_desc: &'l BindGroupLayoutWithDesc) -> Self{
        BindGroupBuilder{
            layout_with_desc,
            entries: Vec::with_capacity(layout_with_desc.entries.len()),
        }
    }

    pub fn push_resources(mut self, resources: Vec<wgpu::BindingResource<'l>>) -> Self{
        for resource in resources{
            self = self.resource(resource);
        }
        self
    }

    pub fn resource(mut self, resource: wgpu::BindingResource<'l>) -> Self{
        assert_lt!(self.entries.len(), self.layout_with_desc.entries.len());
        self.entries.push(wgpu::BindGroupEntry{
            binding: self.layout_with_desc.entries[self.entries.len()].binding,
            resource,
        });
        self
    }

    pub fn create(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::BindGroup{
        assert_eq!(self.entries.len(), self.layout_with_desc.entries.len());
        device.create_bind_group(&wgpu::BindGroupDescriptor{
            label,
            layout: &self.layout_with_desc.layout,
            entries: &self.entries,
        })
    }
}



pub trait BindGroupContent{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>;
    fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>>;
}

// TODO: Macro for Structs that can be bind groups.
macro_rules! bind_group_content{
    ($struct_name:ident, $($field_name:ident),+) =>{
    }
}

macro_rules! bind_group_content_for_tuple{
    ($($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($name: BindGroupContent),+> BindGroupContent for ($($name, )+){
            fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>{
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
bind_group_content_for_tuple!{ A }
bind_group_content_for_tuple!{ A B }
bind_group_content_for_tuple!{ A B C }
bind_group_content_for_tuple!{ A B C D }
bind_group_content_for_tuple!{ A B C D E }
bind_group_content_for_tuple!{ A B C D E F }
bind_group_content_for_tuple!{ A B C D E F G }
bind_group_content_for_tuple!{ A B C D E F G H }
bind_group_content_for_tuple!{ A B C D E F G H I }
bind_group_content_for_tuple!{ A B C D E F G H I J }
bind_group_content_for_tuple!{ A B C D E F G H I J K }
bind_group_content_for_tuple!{ A B C D E F G H I J K L }


impl<C: BindGroupContent, const N: usize> BindGroupContent for [C; N]{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>{
        let mut ret = Vec::with_capacity(N);
        for _i in 0..N{
            ret.append(&mut C::entries(visibility));
        }
        ret
    }

    fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>> {
        let mut ret = Vec::with_capacity(N);
        for content in self{
            ret.append(&mut content.resources());
        }
        ret
    }
}


pub struct BindGroup<C: BindGroupContent>{
    pub content: C,
    bind_group: wgpu::BindGroup,
    bind_group_layout: BindGroupLayoutWithDesc,
}

impl<C: BindGroupContent> BindGroup<C>{
    pub fn new(content: C, device: &wgpu::Device) -> Self{
        let bind_group_layout = Self::create_bind_group_layout(device, None); 

        let bind_group = BindGroupBuilder::new(&bind_group_layout)
            .push_resources(content.resources())
            .create(device, None);

        Self{
            content,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn new_vis(content: C, device: &wgpu::Device, visibility: wgpu::ShaderStages) -> Self{
        let bind_group_layout = Self::create_bind_group_layout_vis(device, None, visibility);

        let bind_group = BindGroupBuilder::new(&bind_group_layout)
            .push_resources(content.resources())
            .create(device, None);

        Self{
            content,
            bind_group,
            bind_group_layout,
        }
    }
}

impl<C: BindGroupContent> CreateBindGroupLayout for BindGroup<C>{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc {
        Self::create_bind_group_layout_vis(device, label, wgpu::ShaderStages::all())
    }
    fn create_bind_group_layout_vis(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> BindGroupLayoutWithDesc {
        BindGroupLayoutBuilder::new()
            .push_entries(C::entries(visibility))
            .create(device, None)
    }
}

impl<C: BindGroupContent> Deref for BindGroup<C>{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C: BindGroupContent> DerefMut for BindGroup<C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<C: BindGroupContent> GetBindGroupLayout for BindGroup<C>{
    fn get_bind_group_layout<'l>(&'l self) -> &'l BindGroupLayoutWithDesc {
        &self.bind_group_layout
    }
}

impl<C: BindGroupContent> GetBindGroup for BindGroup<C>{
    fn get_bind_group<'l>(&'l self) -> &'l wgpu::BindGroup {
        &self.bind_group
    }
}

impl From<BindGroup<super::Texture>> for imgui_wgpu::Texture{
    fn from(bg_texture: BindGroup<super::Texture>) -> Self {
        imgui_wgpu::Texture::from_raw_parts(
            bg_texture.content.texture,
            bg_texture.content.view,
            bg_texture.bind_group,
            bg_texture.content.size,
        )
    }
}



mod glsl{
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
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
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
    pub fn image2D(format: wgpu::TextureFormat, access: wgpu::StorageTextureAccess) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D2,
            format,
        }
    }

    #[allow(non_snake_case)]
    pub fn image2DArray(format: wgpu::TextureFormat, access: wgpu::StorageTextureAccess) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D2Array,
            format,
        }
    }

    #[allow(non_snake_case)]
    pub fn image3D(format: wgpu::TextureFormat, access: wgpu::StorageTextureAccess) -> wgpu::BindingType {
        wgpu::BindingType::StorageTexture {
            access,
            view_dimension: wgpu::TextureViewDimension::D3,
            format,
        }
    }
}

pub mod wgsl{
    pub fn buffer(read_only: bool) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn uniform() -> wgpu::BindingType{
        wgpu::BindingType::Buffer{
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn sampler() -> wgpu::BindingType{
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }

    pub fn texture_2d() -> wgpu::BindingType{
        wgpu::BindingType::Texture{
            sample_type: wgpu::TextureSampleType::Float{ filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }
}
