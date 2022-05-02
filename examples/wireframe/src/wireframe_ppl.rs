use ewgpu::*;

use crate::wireframe::{WireframeVert, WireframeMeshVert};

use super::camera::Camera;

#[derive(DerefMut)]
pub struct WireframeMeshPipeline{
    cppl: wgpu::ComputePipeline,
}

impl WireframeMeshPipeline{
    pub fn new(device: &wgpu::Device) -> Self{

        let shader = ComputeShader::from_src(device, include_str!("shaders/line_ppl.glsl"), None).unwrap();

        let cppl = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
            label: None,
            layout: Some(&Self::LAYOUT.pipeline_layout(device)),
            module: &shader,
            entry_point: "main",
        });

        Self{
            cppl,
        }
    }
}

impl ComputePipeline for WireframeMeshPipeline{
    const LAYOUT: PipelineLayoutDescriptor<'static> = PipelineLayoutDescriptor{
        label: None,
        bind_group_layouts: &[
            BindGroupLayoutDescriptor{
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgsl::buffer(false),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgsl::buffer(false),
                        count: None,
                    }
                ],
            },
            BindGroupLayoutDescriptor{
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgsl::buffer(false),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry{
                        binding: 1,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgsl::buffer(false),
                        count: None,
                    }
                ]
            },
            BindGroupLayoutDescriptor{
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry{
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgsl::uniform(),
                        count: None,
                    }
                ]
            }

        ],
        push_constant_ranges: &[
            wgpu::PushConstantRange{
                stages: wgpu::ShaderStages::all(),
                range: 0..(std::mem::size_of::<Camera>() as u32),
            }
        ],
    };
}
