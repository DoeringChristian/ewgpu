use crate::*;
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

#[derive(DerefMut)]
pub struct BindGroupLayout<'bgl> {
    pub desc: BindGroupLayoutDescriptor<'bgl>,
    #[target]
    pub wgpu: wgpu::BindGroupLayout,
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
#[derive(DerefMut)]
pub struct PipelineLayout<'pl> {
    pub desc: PipelineLayoutDescriptor<'pl>,
    #[target]
    pub wgpu: wgpu::PipelineLayout,
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

const TEST_PPLD: PipelineLayoutDescriptor = PipelineLayoutDescriptor {
    label: None,
    bind_group_layouts: &[BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    }],
    push_constant_ranges: &[],
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DispatchIndirect {
    x: u32,
    y: u32,
    z: u32,
}

pub trait RenderPipeline {
    const Layout: PipelineLayoutDescriptor<'static>;
}
