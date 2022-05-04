
use crate::*;

#[derive(DerefMut)]
pub struct MipMapRenderPipeline{
    rppl: wgpu::RenderPipeline,
}

impl MipMapRenderPipeline{
    pub fn new(device: &wgpu::Device, formats: &[wgpu::TextureFormat]) -> Self{
        let shader = ShaderModule::from_src_wgls(device, include_str!("blit.wgsl"), "main", None).unwrap();
        let rppl = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("MipMapRenderPipeline"),
            layout: None,
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState{
                module: &&shader,
                entry_point: "fs_main",
                targets: &[formats[0].into()],
            }),
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        Self{
            rppl,
        }
    }
}

impl PipelineLayout for MipMapRenderPipeline{
}
