use fxhash::FxHashMap;

use crate::*;
use std::any::{Any, TypeId};
use std::ops::{Deref, DerefMut, RangeBounds};
use std::str;

use core::num::NonZeroU32;
use core::ops::Range;

#[allow(unused)]
const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
pub const DEFAULT_ENTRY_POINT: &str = "main";

///
/// A trait that needs to be implemented for a Pipeline (Compute and Render).
///
pub trait PipelineLayout {
    ///
    /// Returns the PipelineLayout for this pipeline.
    ///
    #[allow(unused_variables)]
    fn layout(device: &wgpu::Device) -> Option<wgpu::PipelineLayout> {
        None
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DispatchIndirect {
    x: u32,
    y: u32,
    z: u32,
}

pub trait RenderPass {
    fn color_target_states(&self) -> Vec<wgpu::ColorTargetState>;
}

///
/// A struct defining an bind group and its dynamic offsets.
/// This is used in the RenderData and ComputeData to specify BindGroups.
///
#[derive(DerefMut)]
pub struct BindGroupWithOffsets<'bgd>{
    #[target]
    pub bind_group: &'bgd wgpu::BindGroup,
    pub offsets: &'bgd [u32],
}

impl<'bgd, B: 'static + BindGroupContent> From<&'bgd BindGroup<B>> for BindGroupWithOffsets<'bgd>{
    fn from(src: &'bgd BindGroup<B>) -> Self {
        BindGroupWithOffsets{
            bind_group: src.bind_group(),
            offsets: &[],
        }
    }
}

impl<'bgd, B: 'static + BindGroupContent> From<&'bgd Bound<B>> for BindGroupWithOffsets<'bgd>{
    fn from(src: &'bgd Bound<B>) -> Self {
        BindGroupWithOffsets{
            bind_group: src.bind_group(),
            offsets: &[],
        }
    }
}

impl<'bgd> From<&'bgd wgpu::BindGroup> for BindGroupWithOffsets<'bgd>{
    fn from(src: &'bgd wgpu::BindGroup) -> Self {
        BindGroupWithOffsets{
            bind_group: src,
            offsets: &[],
        }
    }
}


#[derive(Default)]
pub struct ComputeData<'cd> {
    pub bind_groups: Vec<BindGroupWithOffsets<'cd>>,
    pub push_constants: &'cd [(u32, &'cd [u8])],
}

pub trait ComputePipeline: Deref<Target = wgpu::ComputePipeline> {
    fn set_data<'d, 'cp>(
        &'d self,
        cpass: &'cp mut wgpu::ComputePass<'d>,
        data: ComputeData<'d>,
    ) {
        cpass.set_pipeline(self);
        for (i, bind_group) in data.bind_groups.into_iter().enumerate() {

            cpass.set_bind_group(i as u32, bind_group.bind_group, bind_group.offsets);
        }
        for (i, push_constants) in data.push_constants.iter().enumerate() {
            cpass.set_push_constants(push_constants.0, push_constants.1);
        }
    }
    fn dispatch<'d, 'cp>(
        &'d self,
        cpass: &'cp mut wgpu::ComputePass<'d>,
        data: ComputeData<'d>,
        num: [u32; 3],
    ) {
        self.set_data(cpass, data);
        cpass.dispatch(num[0], num[1], num[2]);
    }
    fn dispatch_indirect<'d, 'cp>(
        &'d self,
        cpass: &'cp mut wgpu::ComputePass<'d>,
        data: ComputeData<'d>,
        indirect_buffer: &'d Buffer<DispatchIndirect>,
        indirect_offset: wgpu::BufferAddress,
    ) {
        self.set_data(cpass, data);
        cpass.dispatch_indirect(indirect_buffer, indirect_offset);
    }
}

impl<C: Deref<Target = wgpu::ComputePipeline> + PipelineLayout> ComputePipeline for C {}

pub struct Viewport {
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
    pub depth_range: Range<f32>,
}

pub struct ScissorRect {
    pub x_range: Range<u32>,
    pub y_range: Range<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirect {
    pub vertex_count: u32,   // The number of vertices to draw.
    pub instance_count: u32, // The number of instances to draw.
    pub first_vertex: u32,   // The Index of the first vertex to draw.
    pub first_instance: u32, // The instance ID of the first instance to draw.
                             // has to be 0, unless [`Features::INDIRECT_FIRST_INSTANCE`] is enabled.
}

#[derive(Default)]
pub struct RenderData<'rd> {
    pub bind_groups: Vec<BindGroupWithOffsets<'rd>>,
    pub push_constants: Vec<(wgpu::ShaderStages, u32, &'rd [u8])>,
    pub index_buffer: Option<(wgpu::BufferSlice<'rd>, wgpu::IndexFormat)>,
    pub vertex_buffers: Vec<wgpu::BufferSlice<'rd>>,
    pub viewport: Option<Viewport>,
    pub scissor_rect: Option<ScissorRect>,
    pub stencil_reference: Option<u32>,
    pub blend_constant: Option<wgpu::Color>,
}

pub trait RenderPipeline: Deref<Target = wgpu::RenderPipeline> {
    fn set_data<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
    ) {
        rpass.set_pipeline(self);

        for (i, bind_group) in data.bind_groups.iter().enumerate() {
            rpass.set_bind_group(i as u32, bind_group.bind_group, bind_group.offsets);
        }

        for (_, push_constants) in data.push_constants.iter().enumerate() {
            rpass.set_push_constants(push_constants.0, push_constants.1, push_constants.2);
        }

        if let Some(index_buffer) = data.index_buffer {
            rpass.set_index_buffer(index_buffer.0, index_buffer.1);
        }

        for (i, vertex_buffer) in data.vertex_buffers.iter().enumerate() {
            rpass.set_vertex_buffer(i as u32, *vertex_buffer);
        }

        if let Some(viewport) = data.viewport {
            rpass.set_viewport(
                viewport.x_range.start,
                viewport.y_range.start,
                viewport.x_range.end - viewport.x_range.start,
                viewport.y_range.end - viewport.y_range.start,
                viewport.depth_range.start,
                viewport.depth_range.end,
            );
        }

        if let Some(scissor_rect) = data.scissor_rect {
            rpass.set_scissor_rect(
                scissor_rect.x_range.start,
                scissor_rect.y_range.start,
                scissor_rect.x_range.end - scissor_rect.x_range.start,
                scissor_rect.y_range.end - scissor_rect.y_range.start,
            );
        }

        if let Some(stencil_reference) = data.stencil_reference {
            rpass.set_stencil_reference(stencil_reference);
        }

        if let Some(blend_constant) = data.blend_constant {
            rpass.set_blend_constant(blend_constant);
        }
    }

    fn draw<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        self.set_data(rpass,  data);
        rpass.draw(vertices, instances);
    }
    fn draw_indexed<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
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
    ) {
        self.set_data(rpass, data);
        rpass.draw_indirect(indirect_buffer, indirect_offset);
    }
    fn draw_indexed_indirect<'d, 'rp>(
        &'d self,
        rpass: &'rp mut wgpu::RenderPass<'d>,
        data: RenderData<'d>,
        indirect_buffer: &'d Buffer<DrawIndirect>,
        indirect_offset: wgpu::BufferAddress,
    ) {
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
    ) {
        self.set_data(rpass, data);
        rpass.multi_draw_indirect(
            indirect_buffer,
            indirect_offset,
            count.min(indirect_buffer.len() as u32),
        );
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
    ) {
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
    ) {
        self.set_data(rpass, data);
        rpass.multi_draw_indexed_indirect(indirect_buffer, indirect_offset, count);
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
    ) {
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

impl<R: Deref<Target = wgpu::RenderPipeline> + PipelineLayout> RenderPipeline for R {}

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
                .into_bound_with(device, &Self::bind_group_layout(device, 0));
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
