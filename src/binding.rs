#[allow(unused)]
use anyhow::*;
use std::{ops::{Deref, DerefMut}, marker::PhantomData};

pub trait CreateBindGroupLayout{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc;
    fn create_bind_group_layout_vis(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> BindGroupLayoutWithDesc;
}

pub trait GetBindGroupLayout{
    fn get_bind_group_layout(&self) -> &BindGroupLayoutWithDesc;
}

///
/// A trait implemented for structs that have a BindGroup.
///
pub trait GetBindGroup{
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

impl GetBindGroup for wgpu::BindGroup{
    fn get_bind_group(&self) -> &wgpu::BindGroup {
        self
    }
}

pub struct BindGroupHandle<T: BindGroupContent>{
    pub bind_group: wgpu::BindGroup,
    _ty: PhantomData<T>,
}

impl<T: BindGroupContent> BindGroupHandle<T>{
    pub fn new(content: &T, device: &wgpu::Device) -> Self{
        let bind_group_layout = T::create_bind_group_layout(device, None); 

        let bind_group = BindGroupBuilder::new(&bind_group_layout)
            .push_resources(content.resources())
            .build(device, None);

        Self{
            bind_group,
            _ty: PhantomData,
        }
    }
}

impl<C: BindGroupContent> CreateBindGroupLayout for C{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> BindGroupLayoutWithDesc {
        Self::create_bind_group_layout_vis(device, label, wgpu::ShaderStages::all())
    }

    fn create_bind_group_layout_vis(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> BindGroupLayoutWithDesc {
        BindGroupLayoutBuilder::new()
            .push_entries(C::entries(visibility))
            .create(device, label)
    }
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

#[derive(Default)]
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

    pub fn build(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::BindGroup{
        assert_eq!(self.entries.len(), self.layout_with_desc.entries.len());
        device.create_bind_group(&wgpu::BindGroupDescriptor{
            label,
            layout: &self.layout_with_desc.layout,
            entries: &self.entries,
        })
    }
}

pub trait IntoBindGroup: BindGroupContent + Sized{
    fn into_bind_group(self, device: &wgpu::Device) -> BindGroup<Self>;
}

impl<C: BindGroupContent> IntoBindGroup for C{
    fn into_bind_group(self, device: &wgpu::Device) -> BindGroup<Self> {
        BindGroup::new(self, device)
    }
}

///
/// A trait implemented for structs that can be the content of a BindGroup.
///
pub trait BindGroupContent: Sized{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>;
    fn resources(&self) -> Vec<wgpu::BindingResource>;
    fn bind_group_handle(&self, device: &wgpu::Device) -> BindGroupHandle<Self>{
        BindGroupHandle::new(self, device)
    }
}

// TODO: Derive macro for BindGroupContent.

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

    fn resources(&self) -> Vec<wgpu::BindingResource> {
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
            .build(device, None);

        Self{
            content,
            bind_group,
            bind_group_layout,
        }
    }

    ///
    /// Has to be called whenever a buffer is changed for example when expand_to_clear is called on
    /// it.
    /// TODO: Change this requirement.
    ///
    pub fn update(&mut self, device: &wgpu::Device){
        self.bind_group = BindGroupBuilder::new(&self.bind_group_layout)
            .push_resources(self.content.resources())
            .build(device, None);
    }

    pub fn new_vis(content: C, device: &wgpu::Device, visibility: wgpu::ShaderStages) -> Self{
        let bind_group_layout = Self::create_bind_group_layout_vis(device, None, visibility);

        let bind_group = BindGroupBuilder::new(&bind_group_layout)
            .push_resources(content.resources())
            .build(device, None);

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
            .create(device, label)
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
    fn get_bind_group_layout(&self) -> &BindGroupLayoutWithDesc {
        &self.bind_group_layout
    }
}

impl<C: BindGroupContent> GetBindGroup for BindGroup<C>{
    fn get_bind_group(&self) -> &wgpu::BindGroup {
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


#[allow(dead_code)]
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
        if filtering{
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        }
        else{
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

#[cfg(test)]
mod test{
    use crate::*;

    #[repr(C)]
    #[make_vert]
    struct Vert2{
        #[location = 0]
        pub pos: [f32; 2],
        #[location = 1]
        pub uv: [f32; 2],
    }

    const QUAD_IDXS: [u32; 6] = [0, 1, 2, 2, 3, 0];

    const QUAD_VERTS: [Vert2; 4] = [
        Vert2 {
            pos: [-1.0, -1.0],
            uv: [0.0, 1.0],
        },
        Vert2 {
            pos: [1.0, -1.0],
            uv: [1.0, 1.0],
        },
        Vert2 {
            pos: [1.0, 1.0],
            uv: [1.0, 0.0],
        },
        Vert2 {
            pos: [-1.0, 1.0],
            uv: [0.0, 0.0],
        },
    ];

    use winit::{window::WindowBuilder, event_loop::EventLoop};

    struct RenderData<'rd>{
        first: &'rd BindGroupHandle<Buffer<f32>>,
    }

    #[test]
    fn test_bind_group(){

        env_logger::init();
        let event_loop = EventLoop::new();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let mut gpu = GPUContextBuilder::new()
            .set_features_util()
            .build();

        let inidces = BufferBuilder::new()
            .index()
            .build(&gpu.device, &QUAD_IDXS);

        let vertices = BufferBuilder::new()
            .vertex()
            .build(&gpu.device, &QUAD_VERTS);


        let vshader = VertexShader::from_src(&gpu.device, "
        #version 460
        #if VERTEX_SHADER

        layout(location = 0) in vec2 i_pos;
        layout(location = 1) in vec2 i_uv;

        layout(location = 0) out vec2 f_pos;
        layout(location = 1) out vec2 f_uv;

        void main(){
            f_pos = i_pos;
            f_uv = i_uv;

            gl_Position = vec4(i_pos, 0.0, 1.0);
        }

        #endif
        ", None).unwrap();

        let fshader = FragmentShader::from_src(&gpu.device, "
        #version 460
        #if FRAGMENT_SHADER

        layout(location = 0) in vec2 f_pos;
        layout(location = 1) in vec2 f_uv;

        layout(location = 0) out vec4 o_color;

        void main(){
            o_color = vec4(1.0, 0., 0., 1.);
        }

        #endif
        ", None).unwrap();

        let layout = PipelineLayoutBuilder::new()
            .push_bind_group(&Buffer::<Vert2>::create_bind_group_layout(&gpu.device, None))
            .build(&gpu.device, None);

        let layout = pipeline_layout!(&gpu.device,
            bind_groups: {Buffer<f32>},
            push_constants: {}
        );

        let pipeline = RenderPipelineBuilder::new(&vshader, &fshader)
            .push_vert_layout(Vert2::buffer_layout())
            .push_target_replace(wgpu::TextureFormat::Rgba8Unorm)
            .set_layout(&layout)
            .build(&gpu.device);

        gpu.encode_img([1920, 1080], |gpu, dst, encoder|{
            let mut rpass = RenderPassBuilder::new()
                .push_color_attachment(dst.color_attachment_clear())
                .begin(encoder, None);

            let mut rpass_ppl = rpass.set_pipeline(&pipeline);

            rpass_ppl.set_vertex_buffer(0, vertices.slice(..));
            rpass_ppl.set_index_buffer(inidces.slice(..));
            rpass_ppl.draw_indexed(0..(inidces.len() as u32), 0, 0..1);
        });
    }
}
