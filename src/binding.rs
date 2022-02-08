#[allow(unused)]
use anyhow::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::any::{Any, TypeId};

pub trait CreateBindGroupLayout{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc;
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




pub trait ToBindGroupLayouts{
    fn bind_group_layouts<'l>(&'l self) -> Vec<&'l wgpu::BindGroupLayout>;
}



pub struct BindGroupLayoutWithDesc{
    pub layout: wgpu::BindGroupLayout,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

pub struct BindGroupLayoutBuilder{
    index: u32,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder{
    pub fn new() -> Self{
        Self{
            index: 0,
            entries: Vec::new(),
        }
    }

    pub fn entry(mut self, entry: wgpu::BindGroupLayoutEntry) -> Self{
        self.entry_ref(entry);
        self
    }

    pub fn entry_ref(&mut self, entry: wgpu::BindGroupLayoutEntry){
        self.entries.push(entry);
        self.index = entry.binding + 1;
    }

    pub fn push_entry(mut self, visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self{
        self.push_entry_ref(visibility, ty);
        self
    }

    pub fn push_entry_ref(&mut self, visibility: wgpu::ShaderStages, ty: wgpu::BindingType){
        let binding = self.index;
        self.entry_ref(wgpu::BindGroupLayoutEntry{
            binding,
            visibility,
            ty,
            count: None,
        });
    }

    #[inline]
    pub fn push_entry_compute(mut self, ty: wgpu::BindingType) -> Self{
        self.push_entry_compute_ref(ty);
        self
    }

    #[inline]
    pub fn push_entry_compute_ref(&mut self, ty: wgpu::BindingType){
        self.push_entry_ref(wgpu::ShaderStages::COMPUTE, ty);
    }

    #[inline]
    pub fn push_entry_fragment(mut self, ty: wgpu::BindingType) -> Self{
        self.push_entry_fragment_ref(ty);
        self
    }

    #[inline]
    pub fn push_entry_fragment_ref(&mut self, ty: wgpu::BindingType){
        self.push_entry_ref(wgpu::ShaderStages::FRAGMENT, ty);
    }

    #[inline]
    pub fn push_entry_vertex(mut self, ty: wgpu::BindingType) -> Self{
        self.push_entry_vertex_ref(ty);
        self
    }

    #[inline]
    pub fn push_entry_vertex_ref(&mut self, ty: wgpu::BindingType){
        self.push_entry_ref(wgpu::ShaderStages::VERTEX, ty);
    }

    #[inline]
    pub fn push_entry_all(mut self, ty: wgpu::BindingType) -> Self{
        self.push_entry_all_ref(ty);
        self
    }

    #[inline]
    pub fn push_entry_all_ref(&mut self, ty: wgpu::BindingType){
        self.push_entry_ref(wgpu::ShaderStages::all(), ty);
    }

    pub fn create(self, device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc{
        BindGroupLayoutWithDesc{
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
                entries: &self.entries,
                label,
            }),
            entries: self.entries,
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

    pub fn resource(mut self, resource: wgpu::BindingResource<'l>) -> Self{
        assert_lt!(self.entries.len(), self.layout_with_desc.entries.len());
        self.entries.push(wgpu::BindGroupEntry{
            binding: self.layout_with_desc.entries[self.entries.len()].binding,
            resource,
        });
        self
    }

    pub fn resource_ref(&mut self, resource: wgpu::BindingResource<'l>){
        assert_lt!(self.entries.len(), self.layout_with_desc.entries.len());
        self.entries.push(wgpu::BindGroupEntry{
            binding: self.layout_with_desc.entries[self.entries.len()].binding,
            resource,
        });
    }

    #[inline]
    pub fn sampler(mut self, sampler: &'l wgpu::Sampler) -> Self{
        self.resource(wgpu::BindingResource::Sampler(sampler))
    }

    #[inline]
    pub fn sampler_ref(&mut self, sampler: &'l wgpu::Sampler){
        self.resource_ref(wgpu::BindingResource::Sampler(sampler));
    }

    #[inline]
    pub fn texture(mut self, texture_view: &'l wgpu::TextureView) -> Self{
        self.resource(wgpu::BindingResource::TextureView(texture_view))
    }

    #[inline]
    pub fn texture_ref(&mut self, texture_view: &'l wgpu::TextureView){
        self.resource_ref(wgpu::BindingResource::TextureView(texture_view));
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
    fn push_entries_to(bind_group_layout_builder: &mut BindGroupLayoutBuilder);
    fn push_resources_to<'bgb>(&'bgb self, bind_group_builder: &mut BindGroupBuilder<'bgb>);
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
            fn push_entries_to(bind_group_layout_builder: &mut BindGroupLayoutBuilder){
                ($($name::push_entries_to(bind_group_layout_builder),)+);
            }
            fn push_resources_to<'bgb>(&'bgb self, bind_group_builder: &mut BindGroupBuilder<'bgb>){
                let ($($name, )+) = self;
                ($($name.push_resources_to(bind_group_builder),)+);
            }
        }
    }
}

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
    fn push_entries_to(bind_group_layout_builder: &mut BindGroupLayoutBuilder) {
        for _i in 0..N{
            C::push_entries_to(bind_group_layout_builder);
        }
    }

    fn push_resources_to<'bgb>(&'bgb self, bind_group_builder: &mut BindGroupBuilder<'bgb>) {
        for content in self{
            content.push_resources_to(bind_group_builder);
        }
    }
}


pub struct BindGroup<C: BindGroupContent>{
    pub content: C,
    bind_group: wgpu::BindGroup,
    bind_group_layout: BindGroupLayoutWithDesc,
}

impl<C: BindGroupContent> BindGroup<C>{
    pub fn new(content: C, device: &wgpu::Device) -> Self{

        let mut bind_group_layout_builder = BindGroupLayoutBuilder::new();
        C::push_entries_to(&mut bind_group_layout_builder);
        //content.push_entries_to(&mut bind_group_layout_builder);
        let bind_group_layout = bind_group_layout_builder.create(device, None);

        let mut bind_group_builder = BindGroupBuilder::new(&bind_group_layout);
        content.push_resources_to(&mut bind_group_builder);
        let bind_group = bind_group_builder.create(device, None);

        Self{
            content,
            bind_group,
            bind_group_layout,
        }
    }
}

impl<C: BindGroupContent> CreateBindGroupLayout for BindGroup<C>{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc {
        let mut bind_group_layout_builder = BindGroupLayoutBuilder::new();
        C::push_entries_to(&mut bind_group_layout_builder);
        bind_group_layout_builder.create(device, label)
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
