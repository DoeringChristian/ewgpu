#[allow(unused)]
use anyhow::*;
use std::{ops::{Deref, DerefMut}, marker::PhantomData};

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

pub trait GetBindGroup{
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

///
/// A trait implemented for structs that can be the content of a BindGroup.
///
pub trait BindGroupContent: Sized{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<BindGroupLayoutEntry>;
    fn resources(&self) -> Vec<wgpu::BindingResource>;
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> BindGroupLayoutWithDesc{
        let entries = Self::entries(visibility);
        let entries: Vec<wgpu::BindGroupLayoutEntry> = entries.iter().enumerate().map(|(i, x)| {
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
    ///
    /// Shader stage S can be inferred from render_data.
    ///
    fn create_bind_group<const S: u32>(&self, device: &wgpu::Device) -> BindGroup<S, Self>{
        let layout = Self::create_bind_group_layout(device, None, wgpu::ShaderStages::from_bits_truncate(S));
        let resources = self.resources();

        let entries: Vec<wgpu::BindGroupEntry> = resources.into_iter().enumerate().map(|(i, r)|{
            wgpu::BindGroupEntry{
                binding: layout.entries[i].binding,
                resource: r,
            }
        }).collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: None,
            entries: &entries,
            layout: &layout.layout,
        });
        BindGroup{
            bind_group,
            _ty: PhantomData,
        }
    }

    fn into_bound<const S: u32>(self, device: &wgpu::Device) -> Bound<S, Self>{
        Bound{
            bind_group: Self::create_bind_group(&self, device),
            content: self,
        }
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

#[allow(non_snake_case)]
pub mod ShaderStages{
    pub const NONE: u32 = 0;
    pub const VERTEX: u32 = 1 << 0;
    pub const FRAGMENT: u32 = 1 << 1;
    pub const COMPUTE: u32 = 1 << 2;

    pub const VERTEX_FRAGMENT: u32 = VERTEX | FRAGMENT;
    pub const ALL: u32 = VERTEX | FRAGMENT | COMPUTE;
}

pub type BindGroupNone<C>           = BindGroup<{ShaderStages::NONE}, C>;
pub type BindGroupAll<C>            = BindGroup<{ShaderStages::ALL}, C>;
pub type BindGroupVertex<C>         = BindGroup<{ShaderStages::VERTEX}, C>;
pub type BindGroupFragment<C>       = BindGroup<{ShaderStages::FRAGMENT}, C>;
pub type BindGroupCompute<C>        = BindGroup<{ShaderStages::COMPUTE}, C>;
pub type BindGroupVertexFragment<C> = BindGroup<{ShaderStages::VERTEX_FRAGMENT}, C>;

///
/// Wrapper for a wgpu::BindGroup
///
pub struct BindGroup<const S: u32, C: BindGroupContent>{
    bind_group: wgpu::BindGroup,
    _ty: PhantomData<C>,
}

impl<const S: u32, C: BindGroupContent> BindGroup<S, C>{
    pub fn create_bind_group_layout(device: &wgpu::Device, label: wgpu::Label) -> BindGroupLayoutWithDesc{
        C::create_bind_group_layout(device, label, wgpu::ShaderStages::from_bits_truncate(S))
    }
}

impl<const S: u32, C: BindGroupContent> GetBindGroup for BindGroup<S, C>{
    fn get_bind_group(&self) -> &wgpu::BindGroup{
        &self.bind_group
    }
}

pub type BoundNone<C>           = Bound<{ShaderStages::NONE}, C>;
pub type BoundAll<C>            = Bound<{ShaderStages::ALL}, C>;
pub type BoundVertex<C>         = Bound<{ShaderStages::VERTEX}, C>;
pub type BoundFragment<C>       = Bound<{ShaderStages::FRAGMENT}, C>;
pub type BoundCompute<C>        = Bound<{ShaderStages::COMPUTE}, C>;
pub type BoundVertexFragment<C> = Bound<{ShaderStages::VERTEX_FRAGMENT}, C>;

///
/// Wraps the content with a bind group.
///
pub struct Bound<const S: u32, C: BindGroupContent>{
    bind_group: BindGroup<S, C>,
    content: C,
}

impl<const S: u32, C: BindGroupContent> Bound<S, C>{
    pub fn create_bind_group_layout(device: &wgpu::Device, label: wgpu::Label) -> BindGroupLayoutWithDesc{
        C::create_bind_group_layout(device, label, wgpu::ShaderStages::from_bits_truncate(S))
    }
}

impl<const S: u32, C: BindGroupContent> GetBindGroup for Bound<S, C>{
    fn get_bind_group(&self) -> &wgpu::BindGroup{
        &self.bind_group.get_bind_group()
    }
}

impl<const S: u32, C: BindGroupContent> Deref for Bound<S, C>{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<const S: u32, C: BindGroupContent> DerefMut for Bound<S, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
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

    struct RenderData{
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
