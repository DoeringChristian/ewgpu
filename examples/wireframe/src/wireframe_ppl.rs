use ewgpu::*;

use crate::wireframe::{MeshVert, WireframeVert};

use super::camera::Camera;

#[derive(DerefMut)]
pub struct WireframeRenderPipeline {
    rppl: wgpu::RenderPipeline,
}

impl PipelineLayout for WireframeRenderPipeline {}

impl WireframeRenderPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        /*
        let vshader =
            VertexShader::from_src_glsl(device, include_str!("shaders/wf_mesh_rppl.glsl"), None)
                .unwrap();
        let fshader =
            FragmentShader::from_src_glsl(device, include_str!("shaders/wf_mesh_rppl.glsl"), None)
                .unwrap();
        */

        let shader = Shader::load(device, std::path::Path::new("src/shaders/wf_mesh_rppl.glsl"), wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, None).unwrap();

        let rppl = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                buffers: &[MeshVert::buffer_layout()],
                ..shader.vertex_state()
                //..vshader.vertex_state()
            },
            fragment: Some(wgpu::FragmentState {
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::all(),
                }],
                ..shader.fragment_state()
                //..fshader.fragment_state()
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multiview: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self { rppl }
    }
}

#[derive(DerefMut)]
pub struct WireframeMeshPipeline {
    cppl: wgpu::ComputePipeline,
}

impl From<wgpu::ComputePipeline> for WireframeMeshPipeline{
    fn from(src: wgpu::ComputePipeline) -> Self {
        Self{cppl: src}
    }
}

impl WireframeMeshPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader =
            ComputeShader::from_src_glsl(device, include_str!("shaders/line_ppl.glsl"), None)
                .unwrap();

        let cppl = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Self::layout(device).as_ref(),
            module: &shader,
            entry_point: "main",
        });

        Self { cppl }
    }
}

impl PipelineLayout for WireframeMeshPipeline {
    fn layout(device: &wgpu::Device) -> Option<wgpu::PipelineLayout> {
        Some(
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::all(),
                                ty: wgsl::buffer(false),
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::all(),
                                ty: wgsl::buffer(false),
                                count: None,
                            },
                        ],
                    }),
                    &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::all(),
                                ty: wgsl::buffer(false),
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::all(),
                                ty: wgsl::buffer(false),
                                count: None,
                            },
                        ],
                    }),
                    &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::all(),
                            ty: wgsl::uniform(),
                            count: None,
                        }],
                    }),
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: 0..(std::mem::size_of::<Camera>() as u32),
                }],
            }),
        )
    }
}
