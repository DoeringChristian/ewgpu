use std::str;
use crate::*;

use core::ops::Range;
use core::num::NonZeroU32;

#[allow(unused)]
const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
pub const DEFAULT_ENTRY_POINT: &str = "main";

///
/// A struct representing a FragmentState.
///
pub struct FragmentState<'fs>{
    pub targets: Vec<wgpu::ColorTargetState>,
    pub entry_point: &'fs str,
    pub shader: &'fs wgpu::ShaderModule,
}

impl<'fs> FragmentState<'fs>{
    pub fn new(shader: &'fs wgpu::ShaderModule) -> Self{
        Self{
            targets: Vec::new(),
            shader,
            entry_point: DEFAULT_ENTRY_POINT,
        }
    }

    pub fn set_entry_point(mut self, entry_point: &'fs str) -> Self{
        self.entry_point = entry_point;
        self
    }
    
    pub fn push_target(mut self, color_target_state: wgpu::ColorTargetState) -> Self{
        self.targets.push(color_target_state);
        self
    }

    pub fn push_target_replace(mut self, format: wgpu::TextureFormat) -> Self{
        self.targets.push(wgpu::ColorTargetState{
            format,
            blend: Some(wgpu::BlendState{
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
pub struct VertexState<'vs>{
    pub vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>,
    pub entry_point: &'vs str,
    pub shader: &'vs wgpu::ShaderModule,
}

impl<'vs> VertexState<'vs>{
    pub fn new(shader: &'vs wgpu::ShaderModule) -> Self{
        Self{
            vertex_buffer_layouts: Vec::new(),
            entry_point: DEFAULT_ENTRY_POINT,
            shader,
        }
    }
    pub fn set_entry_point(mut self, entry_point: &'vs str) -> Self{
        self.entry_point = entry_point;
        self
    }
    pub fn push_vert_layout(mut self, vertex_buffer_layout: wgpu::VertexBufferLayout<'vs>) -> Self{
        self.vertex_buffer_layouts.push(vertex_buffer_layout);
        self
    }
    pub fn push_vert_layouts(mut self, mut vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vs>>) -> Self{
        self.vertex_buffer_layouts.append(&mut vertex_buffer_layouts);
        self
    }
}

/// 
/// A wrapper for wgpu::RenderPipeline with PushConstantRanges.
///
pub struct RenderPipeline{
    pub pipeline: wgpu::RenderPipeline,
    pub push_const_ranges: Vec<wgpu::PushConstantRange>
}

pub struct PipelineLayout{
    pub layout: wgpu::PipelineLayout,
    pub push_const_ranges: Vec<wgpu::PushConstantRange>,
}

impl PipelineLayout{
    ///
    /// Create a new pipeline layout from push_const_layouts and bind_group_layouts.
    ///
    /// Mostly for the pipeline_layout macro.
    ///
    pub fn new(device: &wgpu::Device, bind_group_layouts: &[&wgpu::BindGroupLayout], push_const_layouts: &[PushConstantLayout], label: wgpu::Label) -> Self{

        let mut offset = 0;
        let push_const_ranges: Vec<wgpu::PushConstantRange> = push_const_layouts.iter()
            .map(|x| {
                // align to 4 bytes.
                // TODO: write tests and use Align trait.
                let size_aligned = (((x.size as i32 - 4)/4 + 1)*4) as u32;
                let range = Range::<u32>{
                    start: offset,
                    end: offset + size_aligned,
                };
                offset = range.end;
                wgpu::PushConstantRange{
                    stages: x.stages,
                    range,
                }
            }).collect();

        Self{
            layout: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label,
                push_constant_ranges: &push_const_ranges,
                bind_group_layouts,
            }),
            push_const_ranges,
        }
    }
}

// TODO: put bind_group_names in Arc
#[derive(Default)]
pub struct PipelineLayoutBuilder<'l>{
    bind_group_layouts: Vec<&'l binding::BindGroupLayoutWithDesc>,
    push_const_layouts: Vec<PushConstantLayout>,
}

impl<'l> PipelineLayoutBuilder<'l>{
    pub fn new() -> Self{
        Self{
            bind_group_layouts: Vec::new(),
            push_const_layouts: Vec::new(),
        }
    }

    // TODO: simplify
    pub fn push_bind_group(mut self, bind_group_layout: &'l binding::BindGroupLayoutWithDesc) -> Self{
        self.bind_group_layouts.push(bind_group_layout);
        self
    }

    pub fn push_const_layout(mut self, push_const_layout: PushConstantLayout) -> Self{
        self.push_const_layouts.push(push_const_layout);
        self
    }

    pub fn build(self, device: &wgpu::Device, label: Option<&str>) -> PipelineLayout{

        let mut bind_group_layouts = Vec::with_capacity(self.bind_group_layouts.len());
        for bind_group_layout_desc in self.bind_group_layouts{
            bind_group_layouts.push(&bind_group_layout_desc.layout);
        }

        // Convert the push_const_layouts to push_const_ranges using alignment
        let mut offset = 0;
        let push_const_ranges: Vec<wgpu::PushConstantRange> = self.push_const_layouts.iter()
            .map(|x| {
                // align to 4 bytes.
                let size_aligned = (((x.size as i32 - 4)/4 + 1)*4) as u32;
                let range = Range::<u32>{
                    start: offset,
                    end: offset + size_aligned,
                };
                offset = range.end;
                wgpu::PushConstantRange{
                    stages: x.stages,
                    range,
                }
            }).collect();

        PipelineLayout{
            layout: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label,
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &push_const_ranges,
            }),
            push_const_ranges,
        }
    }
}

pub struct RenderPassPipeline<'rp, 'rpr>{
    pub render_pass: &'rpr mut RenderPass<'rp>,
    pub pipeline: &'rp RenderPipeline,
}

impl<'rp, 'rpr> RenderPassPipeline<'rp, 'rpr>{
    pub fn set_bind_group<B: binding::GetBindGroup>(&mut self, index: u32, bind_group: &'rp B, offsets: &'rp [wgpu::DynamicOffset]){
        self.render_pass.render_pass.set_bind_group(
            index,
            bind_group.get_bind_group(),
            offsets
        );
    }

    pub fn set_bind_groups(&mut self, bind_groups: &[&'rp wgpu::BindGroup]){
        for (i, bind_group) in bind_groups.iter().enumerate(){
            self.render_pass.render_pass.set_bind_group(
                i as u32,
                &bind_group,
                &[]
            );
        }
    }

    pub fn set_push_const<C: PushConstant>(&mut self, index: usize, constant: &C){
        self.render_pass.render_pass.set_push_constants(
            self.pipeline.push_const_ranges[index].stages, 
            self.pipeline.push_const_ranges[index].range.start,
            bytemuck::bytes_of(constant));
    }

    pub fn set_vertex_buffer<T: VertLayout>(&mut self, index: u32, buffer_slice: BufferSlice<'rp, T>){
        self.render_pass.render_pass.set_vertex_buffer(
            index,
            buffer_slice.slice
        );
    }

    pub fn set_index_buffer(&mut self, buffer_slice: BufferSlice<'rp, u32>){
        self.render_pass.render_pass.set_index_buffer(buffer_slice.slice, wgpu::IndexFormat::Uint32);
    }

    pub fn set_index_buffer16(&mut self, buffer_slice: BufferSlice<'rp, u16>){
        self.render_pass.render_pass.set_index_buffer(buffer_slice.slice, wgpu::IndexFormat::Uint16);
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>){
        self.render_pass.render_pass.draw(
            vertices.start..vertices.end,
            instances.start..instances.end
        );
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>){
        self.render_pass.render_pass.draw_indexed(
            indices.start..indices.end, 
            base_vertex, 
            instances.start..instances.end
        );
    }

    pub fn set_pipeline(&'rpr mut self, pipeline: &'rp RenderPipeline) -> Self{
        self.render_pass.render_pass.set_pipeline(&pipeline.pipeline);
        Self{
            render_pass: self.render_pass,
            pipeline,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DispatchIndirect{
    x: u32,
    y: u32,
    z: u32,
}

///
/// Wrapper for wgpu::ComputePass
///
pub struct ComputePass<'cp>{
    pub cpass: wgpu::ComputePass<'cp>,
}

impl<'cp> ComputePass<'cp>{
    pub fn new(encoder: &'cp mut wgpu::CommandEncoder, label: Option<&str>) -> Self{
        let cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
            label,
        });
        Self{
            cpass,
        }
    }

    pub fn set_pipeline(&mut self, pipeline: &'cp ComputePipeline) -> ComputePassPipeline<'cp, '_>{
        self.cpass.set_pipeline(&pipeline.pipeline);
        ComputePassPipeline{
            cpass: self,
            pipeline,
        }
    }

}

///
/// A ComputePass with pipeline needed for push_const offsets.
///
pub struct ComputePassPipeline<'cp, 'cpr>{
    pub cpass: &'cpr mut ComputePass<'cp>,
    pub pipeline: &'cp ComputePipeline,
}

impl<'cp, 'cpr> ComputePassPipeline<'cp, 'cpr>{
    pub fn set_bind_group<B: binding::GetBindGroup>(&mut self, index: u32, bind_group: &'cp B, offsets: &'cp [wgpu::DynamicOffset]){
        self.cpass.cpass.set_bind_group(index, bind_group.get_bind_group(), offsets);
    }

    pub fn set_push_const<C: PushConstant>(&mut self, index: u32, constant: &C){
        self.cpass.cpass.set_push_constants(
            self.pipeline.push_const_ranges[index as usize].range.start,
            bytemuck::bytes_of(constant));
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32){
        self.cpass.cpass.dispatch(x, y, z);
    }

    pub fn dispatch_indirect(&mut self, indirect_buffer: &'cp Buffer<DispatchIndirect>, indirect_offset: wgpu::BufferAddress){
        self.cpass.cpass.dispatch_indirect(&indirect_buffer.buffer, indirect_offset);
    }

    pub fn set_pipeline(&'cpr mut self, pipeline: &'cp ComputePipeline) -> Self{
        self.cpass.cpass.set_pipeline(&pipeline.pipeline);
        Self{
            cpass: self.cpass,
            pipeline,
        }
    }
}

///
/// A wrapper for wgpu::ComputePipeline with PushConstantRanges
///
pub struct ComputePipeline{
    pub pipeline: wgpu::ComputePipeline,
    pub push_const_ranges: Vec<wgpu::PushConstantRange>,
}

///
/// A builder for a ComputePipeline
///
///
pub struct ComputePipelineBuilder<'cpb>{
    label: wgpu::Label<'cpb>,
    layout: Option<&'cpb PipelineLayout>,
    module: &'cpb wgpu::ShaderModule,
    entry_point: &'cpb str,
}

impl<'cpb> ComputePipelineBuilder<'cpb>{

    pub fn new(module: &'cpb ComputeShader) -> Self{
        Self{
            label: None,
            layout: None,
            module,
            entry_point: "main",
        }
    }

    pub fn set_entry_point(mut self, entry_point: &'cpb str) -> Self{
        self.entry_point = entry_point;
        self
    }

    pub fn set_label(mut self, label: wgpu::Label<'cpb>) -> Self{
        self.label = label;
        self
    }

    pub fn set_layout(mut self, layout: &'cpb PipelineLayout) -> Self{
        self.layout = Some(layout);
        self
    }

    pub fn build(&mut self, device: &wgpu::Device) -> ComputePipeline{
        let layout = self.layout.expect("no layout provided");
        ComputePipeline{
            pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
                label: self.label,
                layout: Some(&layout.layout),
                module: self.module,
                entry_point: self.entry_point,
            }),
            push_const_ranges: layout.push_const_ranges.clone(),
        }
    }
}


///
/// A wrapper for wgpu::RenderPass
///
pub struct RenderPass<'rp>{
    pub render_pass: wgpu::RenderPass<'rp>,
}

impl<'rp> RenderPass<'rp>{

    pub fn set_pipeline(&mut self, pipeline: &'rp RenderPipeline) -> RenderPassPipeline<'rp, '_>{
        self.render_pass.set_pipeline(&pipeline.pipeline);
        RenderPassPipeline{
            render_pass: self,
            pipeline,
        }
    }

    /* TODO: maybe remove RenderPassPipeline
       #[inline]
       pub fn set_bind_group(&mut self, index: u32, bind_group: &'rp wgpu::BindGroup, offsets: &'rp [wgpu::DynamicOffset]){
       self.render_pass.set_bind_group(index, bind_group, offsets);
       }
       */
}

///
/// A builder for the RenderPass.
///
#[derive(Default)]
pub struct RenderPassBuilder<'rp>{
    color_attachments: Vec<wgpu::RenderPassColorAttachment<'rp>>,
}

impl<'rp> RenderPassBuilder<'rp>{
    pub fn new() -> Self{
        Self{
            color_attachments: Vec::new(),
        }
    }

    pub fn push_color_attachment(mut self, color_attachment: wgpu::RenderPassColorAttachment<'rp>) -> Self{
        self.color_attachments.push(color_attachment);
        self
    }

    // TODO: add depth_stencil_attachment
    pub fn begin(self, encoder: &'rp mut wgpu::CommandEncoder, label: Option<&'rp str>) -> RenderPass<'rp>{
        RenderPass{
            render_pass: encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label,
                color_attachments: &self.color_attachments,
                depth_stencil_attachment: None,
            }),
        }
    }
}

///
/// A Builder for a RenderPipeline.
///
/// Pipeline layout has to be set.
///
pub struct RenderPipelineBuilder<'rpb>{
    label: Option<&'rpb str>,
    layout: Option<&'rpb PipelineLayout>,
    vertex: VertexState<'rpb>,
    fragment: FragmentState<'rpb>,
    primitive: wgpu::PrimitiveState,
    depth_stencil: Option<wgpu::DepthStencilState>,
    multisample: wgpu::MultisampleState,
    multiview: Option<NonZeroU32>,
}

impl<'rpb> RenderPipelineBuilder<'rpb>{

    pub fn new(vertex_shader: &'rpb VertexShader, fragment_shader: &'rpb FragmentShader) -> Self{
        let label = None;
        let layout = None;
        let primitive = wgpu::PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };
        let depth_stencil = None;
        let multisample = wgpu::MultisampleState{
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        let multiview = None;

        let vertex = VertexState{
            vertex_buffer_layouts: Vec::new(),
            entry_point: DEFAULT_ENTRY_POINT,
            shader: vertex_shader,
        };
        let fragment = FragmentState{
            targets: Vec::new(),
            entry_point: DEFAULT_ENTRY_POINT,
            shader: fragment_shader,
        };

        Self{
            label,
            layout,
            vertex,
            fragment,
            primitive,
            depth_stencil,
            multisample,
            multiview,
        }
    }

    #[inline]
    pub fn set_primitive(mut self, primitive: wgpu::PrimitiveState) -> Self{
        self.primitive = primitive;
        self
    }

    #[inline]
    pub fn set_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self{
        self.primitive.topology = topology;
        self
    }

    #[inline]
    pub fn set_strip_index_format(mut self, format: Option<wgpu::IndexFormat>) -> Self{
        self.primitive.strip_index_format = format;
        self
    }

    #[inline]
    pub fn set_front_face(mut self, front_face: wgpu::FrontFace) -> Self{
        self.primitive.front_face = front_face;
        self
    }

    #[inline]
    pub fn set_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self{
        self.primitive.cull_mode = cull_mode;
        self
    }

    #[inline]
    pub fn set_polygon_mode(mut self, mode: wgpu::PolygonMode) -> Self{
        self.primitive.polygon_mode = mode;
        self
    }

    #[inline]
    pub fn set_unclipped_depth(mut self, unclipped_depth: bool) -> Self{
        self.primitive.unclipped_depth = unclipped_depth;
        self
    }

    #[inline]
    pub fn set_conservative(mut self, conservative: bool) -> Self{
        self.primitive.conservative = conservative;
        self
    }

    #[inline]
    pub fn set_depth_stencil_less32(mut self) -> Self{
        self.depth_stencil = Some(wgpu::DepthStencilState{
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        self
    }

    #[inline]
    pub fn set_depth_stencil(mut self, depth_stencil: Option<wgpu::DepthStencilState>) -> Self{
        self.depth_stencil = depth_stencil;
        self
    }

    #[inline]
    pub fn set_multisample(mut self, multisample: wgpu::MultisampleState) -> Self{
        self.multisample = multisample;
        self
    }

    #[inline]
    pub fn set_multiview(mut self, multiview: Option<NonZeroU32>) -> Self{
        self.multiview = multiview;
        self
    }

    #[inline]
    pub fn push_vert_layout(mut self, vertex_buffer_layout: wgpu::VertexBufferLayout<'rpb>) -> Self{
        self.vertex = self.vertex.push_vert_layout(vertex_buffer_layout);
        self
    }

    #[inline]
    pub fn push_vert_layouts(mut self, vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'rpb>>) -> Self{
        self.vertex = self.vertex.push_vert_layouts(vertex_buffer_layouts);
        self
    }

    #[inline]
    pub fn push_drawable_layouts<D: Drawable>(self) -> Self{
        self.push_vert_layouts(D::create_vert_buffer_layouts())
    }

    ///
    /// Pushes a RenderTarget to the fragment state.
    ///
    /// Has to be pushed in the same order as their corresponding color attachements.
    ///
    #[inline]
    pub fn push_target_replace(mut self, format: wgpu::TextureFormat) -> Self{
        self.fragment = self.fragment.push_target_replace(format);
        self
    }

    ///
    /// Pushes a RenderTarget to the fragment state.
    ///
    /// Has to be pushed in the same order as their corresponding color attachements.
    ///
    #[inline]
    pub fn push_target(mut self, color_target_state: wgpu::ColorTargetState) -> Self{
        self.fragment.targets.push(color_target_state);
        self
    }

    #[inline]
    pub fn set_layout(mut self, layout: &'rpb PipelineLayout) -> Self{
        self.layout = Some(layout);
        self
    }

    #[inline]
    pub fn set_label(mut self, label: wgpu::Label<'rpb>) -> Self{
        self.label = label;
        self
    }

    #[inline]
    pub fn vert_entry_point(mut self, entry_point: &'rpb str) -> Self{
        self.vertex.entry_point = entry_point;
        self
    }

    #[inline]
    pub fn frag_entry_point(mut self, entry_point: &'rpb str) -> Self{
        self.fragment.entry_point = entry_point;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> RenderPipeline{

        let layout = self.layout.expect("no layout provided");

        let fragment = wgpu::FragmentState{
            module: self.fragment.shader,
            entry_point: self.fragment.entry_point,
            targets: &self.fragment.targets,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: self.label,
            layout: Some(&layout.layout),
            vertex: wgpu::VertexState{
                module: self.vertex.shader,
                entry_point: self.vertex.entry_point,
                buffers: &self.vertex.vertex_buffer_layouts,
            },
            fragment: Some(fragment),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            multiview: self.multiview,
        });

        RenderPipeline{
            pipeline: render_pipeline,
            push_const_ranges: layout.push_const_ranges.clone(),
        }
    }
}




// TODO:
// Counting RenderPass
