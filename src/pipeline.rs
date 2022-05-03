use fxhash::FxHashMap;

use crate::*;
use std::any::{TypeId, Any};
use std::ops::{RangeBounds, Deref, DerefMut};
use std::str;

use core::num::NonZeroU32;
use core::ops::Range;

#[allow(unused)]
const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
pub const DEFAULT_ENTRY_POINT: &str = "main";

///
/// A struct representing a FragmentState.
///
pub struct FragmentState<'fs> {
    pub targets: Vec<wgpu::ColorTargetState>,
    pub entry_point: &'fs str,
    pub shader: &'fs wgpu::ShaderModule,
}

impl<'fs> FragmentState<'fs> {
    pub fn new(shader: &'fs wgpu::ShaderModule) -> Self {
        Self {
            targets: Vec::new(),
            shader,
            entry_point: DEFAULT_ENTRY_POINT,
        }
    }

    pub fn set_entry_point(mut self, entry_point: &'fs str) -> Self {
        self.entry_point = entry_point;
        self
    }

    pub fn push_target(mut self, color_target_state: wgpu::ColorTargetState) -> Self {
        self.targets.push(color_target_state);
        self
    }

    pub fn push_target_replace(mut self, format: wgpu::TextureFormat) -> Self {
        self.targets.push(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::all(),
        });
        self
    }
}

///
/// Layout of the VertexState of a Pipeline.
/// It describes the buffer layouts as well as the names used when setting by name in the
/// RenderPassPipeline process.
///
pub struct VertexState<'vs> {
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>,
    pub entry_point: &'vs str,
    pub shader: &'vs wgpu::ShaderModule,
}

impl<'vs> VertexState<'vs> {
    pub fn new(shader: &'vs wgpu::ShaderModule) -> Self {
        Self {
            vertex_buffer_layouts: Vec::new(),
            entry_point: DEFAULT_ENTRY_POINT,
            shader,
        }
    }
    pub fn set_entry_point(mut self, entry_point: &'vs str) -> Self {
        self.entry_point = entry_point;
        self
    }
    pub fn push_vert_layout(mut self, vertex_buffer_layout: wgpu::VertexBufferLayout<'vs>) -> Self {
        self.vertex_buffer_layouts.push(vertex_buffer_layout);
        self
    }
    pub fn push_vert_layouts(
        mut self,
        mut vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>,
    ) -> Self {
        self.vertex_buffer_layouts
            .append(&mut vertex_buffer_layouts);
        self
    }
}

#[derive(Copy, Clone)]
pub struct BindGroupLayoutDescriptor<'bgld> {
    pub label: wgpu::Label<'bgld>,
    pub entries: &'bgld [wgpu::BindGroupLayoutEntry],
}

impl<'bgld> BindGroupLayoutDescriptor<'bgld> {
    pub fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label,
            entries: self.entries,
        })
    }
}

#[derive(Copy, Clone)]
pub struct PipelineLayoutDescriptor<'pld> {
    pub label: wgpu::Label<'pld>,
    pub bind_group_layouts: &'pld [BindGroupLayoutDescriptor<'pld>],
    pub push_constant_ranges: &'pld [wgpu::PushConstantRange],
}

impl<'pld> PipelineLayoutDescriptor<'pld> {
    pub fn pipeline_layout(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        let bind_group_layouts: Vec<wgpu::BindGroupLayout> = self
            .bind_group_layouts
            .iter()
            .map(|bgl| bgl.bind_group_layout(device))
            .collect();

        let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> =
            bind_group_layouts.iter().map(|bgl| bgl).collect();

        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label,
            bind_group_layouts: &bind_group_layout_refs,
            push_constant_ranges: self.push_constant_ranges,
        })
    }
}

pub trait PipelineLayout{
    const LAYOUT: PipelineLayoutDescriptor<'static>;
    fn bind_group_layout(device: &wgpu::Device, index: usize) -> wgpu::BindGroupLayout{
        Self::LAYOUT.bind_group_layouts[index].bind_group_layout(device)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DispatchIndirect {
    x: u32,
    y: u32,
    z: u32,
}

pub trait RenderPass{
    fn color_target_states(&self) -> Vec<wgpu::ColorTargetState>;
}

#[derive(Default)]
pub struct ComputeData<'cd>{
    pub bind_groups: &'cd[(&'cd wgpu::BindGroup, &'cd [u32])],
    pub push_constants: &'cd[(u32, &'cd [u8])],
}

pub trait ComputePipeline: Deref<Target = wgpu::ComputePipeline>{
    fn set_data<'d, 'cp>(
        &'d self, 
        cpass: &'cp mut wgpu::ComputePass<'d>, 
        data: ComputeData<'d>,
    ){
        cpass.set_pipeline(self);
        for (i, bind_group) in data.bind_groups.iter().enumerate(){
            cpass.set_bind_group(i as u32, bind_group.0, bind_group.1);
        }
        for (i, push_constants) in data.push_constants.iter().enumerate(){
            cpass.set_push_constants(push_constants.0, push_constants.1);
        }
    }
    fn dispatch<'d, 'cp>(
        &'d self, 
        cpass: &'cp mut wgpu::ComputePass<'d>, 
        data: ComputeData<'d>,
        num: [u32; 3],
    ){
        self.set_data(cpass, data);
        cpass.dispatch(num[0], num[1], num[2]);
    }
    fn dispatch_indirect<'d, 'cp>(
        &'d self, 
        cpass: &'cp mut wgpu::ComputePass<'d>, 
        data: ComputeData<'d>,
        indirect_buffer: &'d Buffer<DispatchIndirect>,
        indirect_offset: wgpu::BufferAddress,
    ){
        self.set_data(cpass, data);
        cpass.dispatch_indirect(indirect_buffer, indirect_offset);
    }
}

impl<C: Deref<Target = wgpu::ComputePipeline> + PipelineLayout> ComputePipeline for C{

}

pub struct Viewport{
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
    pub depth_range: Range<f32>,
}

pub struct ScissorRect{
    pub x_range: Range<u32>,
    pub y_range: Range<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirect {
    pub vertex_count: u32, // The number of vertices to draw.
    pub instance_count: u32, // The number of instances to draw.
    pub first_vertex: u32, // The Index of the first vertex to draw.
    pub first_instance: u32, // The instance ID of the first instance to draw.
    // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
}

#[derive(Default)]
pub struct RenderData<'rd>{
    pub bind_groups: Vec<(&'rd wgpu::BindGroup, &'rd [u32])>,
    pub push_constants: Vec<(wgpu::ShaderStages, u32, &'rd[u8])>,
    pub index_buffer: Option<(wgpu::BufferSlice<'rd>, wgpu::IndexFormat)>,
    pub vertex_buffers: Vec<wgpu::BufferSlice<'rd>>,
    pub viewport: Option<Viewport>,
    pub scissor_rect: Option<ScissorRect>,
    pub stencil_reference: Option<u32>,
    pub blend_constant: Option<wgpu::Color>,
}

pub trait RenderPipeline: Deref<Target = wgpu::RenderPipeline>{
    fn set_data<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
    ){
        rpass.set_pipeline(self);

        for (i, bind_group) in data.bind_groups.iter().enumerate(){
            rpass.set_bind_group(i as u32, &bind_group.0, bind_group.1);
        }

        for (_, push_constants) in data.push_constants.iter().enumerate(){
            rpass.set_push_constants(push_constants.0, push_constants.1, push_constants.2);
        }

        if let Some(index_buffer) = data.index_buffer{
            rpass.set_index_buffer(index_buffer.0, index_buffer.1);
        }

        for (i, vertex_buffer) in data.vertex_buffers.iter().enumerate(){
            rpass.set_vertex_buffer(i as u32, *vertex_buffer);
        }

        if let Some(viewport) = data.viewport{
            rpass.set_viewport(
                viewport.x_range.start, 
                viewport.y_range.start, 
                viewport.x_range.end - viewport.x_range.start,
                viewport.y_range.end - viewport.y_range.start,
                viewport.depth_range.start, viewport.depth_range.end,
            );
        }

        if let Some(scissor_rect) = data.scissor_rect{
            rpass.set_scissor_rect(
                scissor_rect.x_range.start,
                scissor_rect.y_range.start,
                scissor_rect.x_range.end - scissor_rect.x_range.start,
                scissor_rect.y_range.end - scissor_rect.y_range.start,
            );
        }

        if let Some(stencil_reference) = data.stencil_reference{
            rpass.set_stencil_reference(stencil_reference);
        }

        if let Some(blend_constant) = data.blend_constant{
            rpass.set_blend_constant(blend_constant);
        }
    }

    fn draw<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        vertices: Range<u32>,
        instances: Range<u32>,
    ){
        self.set_data(rpass, data);
        rpass.draw(vertices, instances);
    }
    fn draw_indexed<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ){
        self.set_data(rpass, data);
        rpass.draw_indexed(indices, base_vertex, instances);
    }
    // TODO: Determine weather indirect offset is in bytes.
    fn draw_indirect<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
    ){
        self.set_data(rpass, data);
        rpass.draw_indirect(indirect_buffer, indirect_offset);
    }
    fn draw_indexed_indirect<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
    ){
        self.set_data(rpass, data);
        rpass.draw_indexed_indirect(indirect_buffer, indirect_offset);
    }
    fn multi_draw_indirect<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
        count: u32,
    ){
        self.set_data(rpass, data);
        rpass.multi_draw_indirect(indirect_buffer, indirect_offset, count.min(indirect_buffer.len() as u32));
    }
    fn multi_draw_indirect_count<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
        count_buffer: &'d Buffer<u32>,
        count_offset: wgpu::BufferAddress,
        max_count: u32,
    ){
        self.set_data(rpass, data);
        rpass.multi_draw_indirect_count(
            indirect_buffer, 
            indirect_offset, 
            count_buffer,
            count_offset,
            max_count.min(count_buffer.len() as u32),
        );
    }
    fn multi_draw_indexed_indirect<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
        count: u32,
    ){
        self.set_data(rpass, data);
        rpass.multi_draw_indexed_indirect(
            indirect_buffer, 
            indirect_offset, 
            count,
        );
    }
    fn multi_draw_indexed_indirect_count<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
        count_buffer: &'d Buffer<u32>,
        count_offset: wgpu::BufferAddress,
        max_count: u32,
    ){
        self.set_data(rpass, data);
        rpass.multi_draw_indexed_indirect_count(
            indirect_buffer, 
            indirect_offset, 
            count_buffer,
            count_offset,
            max_count.min(count_buffer.len() as u32),
        );
    }
}

impl<R: Deref<Target = wgpu::RenderPipeline> + PipelineLayout> RenderPipeline for R{
}

static mut PIPELINES: Option<ResourceManager> = None;

#[cfg(test)]
mod test {
    use crate::*;
    #[derive(DerefMut)]
    struct TestRenderPipeline {
        rppl: wgpu::RenderPipeline,
    }
    impl TestRenderPipeline {
        pub fn new(device: &wgpu::Device) -> Self {
            let b1 = BufferBuilder::new()
                .storage()
                .build(device, &[1])
                .into_bound(device, &Self::bind_group_layout(device, 0));
            todo!()
        }
    }
    impl PipelineLayout for TestRenderPipeline {
        const LAYOUT: PipelineLayoutDescriptor<'static> = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::all(),
                        ty: binding::wgsl::uniform(),
                        count: None,
                    },
                ],
            }],
            push_constant_ranges: &[],
        };
    }
}
