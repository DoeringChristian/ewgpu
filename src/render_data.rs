use std::ops::Deref;

use crate::*;

pub trait PipelineData{

}

pub trait ComputeData: PipelineData{
    fn bind_groups<'d>(&'d self) -> Vec<&'d wgpu::BindGroup>;
    fn bind_group_layouts(device: &wgpu::Device) -> Vec<BindGroupLayoutWithDesc>;
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
