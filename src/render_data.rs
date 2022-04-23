use std::ops::Deref;

use crate::*;

pub trait PipelineData{

}

///
/// A Trait implemented with #[derive(ComputeData)].
///
/// ```rust
/// use ewgpu::*;
///
/// #[derive(ComputeData)]
/// struct ExampleComputeData{
///    #[bind_group]
///    src: &'rd BoundAll<Buffer<u32>>,
///    #[bind_group]
///    dst: &'rd BoundAll<Buffer<u32>>,
///    #[push_constant]
///    pc01: PushConstantCompute<u32>,
/// };
///
/// let mut gpu = GPUContextBuilder::new()
///     .set_features_util()
///     .set_limits(wgpu::Limits{
///         max_push_constant_size: 128,
///         ..Default::default()
///     })
///     .build();
///
/// let cshader = ComputeShader::from_src(&gpu.device, "
/// #version 460
/// #if COMPUTE_SHADER
/// 
/// layout(set = 0, binding = 0) buffer InBuffer{
///     uint in_buf[];
/// };
/// layout(set = 1, binding = 0) buffer OutBuffer{
///     uint out_buf[];
/// };
/// layout(push_constant) uniform PushConstant{
///     uint a;
/// };
/// 
/// void main(){
///     uint i = gl_GlobalInvocationID.x;
///     out_buf[i] = in_buf[i] + a;
/// }
/// 
/// #endif
/// ", None).unwrap();
///
/// let pipeline = ComputePipelineBuilder::<ExampleComputeData>::new(&cshader)
///     .build(&gpu.device);
///
/// let in_buffer = BufferBuilder::new()
///     .storage()
///     .build(&gpu.device, &[1 as u32])
///     .into_bound(&gpu.device);
/// let out_buffer = BufferBuilder::new()
///     .storage().read()
///     .build(&gpu.device, &[1 as u32])
///     .into_bound(&gpu.device);
///
/// gpu.encode(|gpu, encoder|{
///     let mut cpass = ComputePass::new(encoder, None);
///
///     let comp_data = TestComputeData{
///         src: &in_buffer,
///         dst: &out_buffer,
///         pc01: PushConstant::new(1),
///     };
///
///     cpass.set_pipeline(&pipeline)
///         .set_compute_data(&comp_data)
///         .dispatch(1, 1, 1);
///
///     assert_eq!(out_buffer.slice(..).map_blocking(&gpu.device).as_ref(), &[2]);
///     
/// })
///
///
/// ```
///
pub trait ComputeData: PipelineData{
    fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>;
    fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>;
    fn push_const_layouts() -> Vec<PushConstantLayout>{
        vec![]
    }
    fn push_consts<'d>(&'d self) -> Vec<(&'d[u8], wgpu::ShaderStages)>{
        vec![]
    }
}


///
///
///
pub trait RenderData: PipelineData{
    fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>;
    fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>;
    fn vert_buffer_slices<'d>(&'d self) -> Vec<wgpu::BufferSlice<'d>>{
        vec![]
    }
    fn vert_layout() -> Vec<wgpu::VertexBufferLayout<'static>>;
    fn idx_buffer_slice<'d>(&'d self) -> (wgpu::IndexFormat, wgpu::BufferSlice<'d>);
    fn push_const_layouts() -> Vec<PushConstantLayout>{
        vec![]
    }
    fn push_consts<'d>(&'d self) -> Vec<(&'d[u8], wgpu::ShaderStages)>{
        vec![]
    }
}

///
/// A trait with a function returning a wgpu::VertexBufferLayout for any buffer_slice that has a 
/// content of that implements VertLayout.
///
pub trait BufferSliceVertLayout{
    fn buffer_slice_vert_layout() -> wgpu::VertexBufferLayout<'static>;
}

impl<'bs, V: VertLayout> BufferSliceVertLayout for BufferSlice<'bs, V>{
    fn buffer_slice_vert_layout() -> wgpu::VertexBufferLayout<'static> {
        V::buffer_layout()
    }
}

///
/// A trait with a function returning the wgpu::IndexFormat for a BufferSlice with content type u32
/// and u16
///
pub trait IndexBufferFormat{
    fn index_buffer_format() -> wgpu::IndexFormat;
}

impl<'bs> IndexBufferFormat for BufferSlice<'bs, u32>{
    fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}

impl<'bs> IndexBufferFormat for BufferSlice<'bs, u16>{
    fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
