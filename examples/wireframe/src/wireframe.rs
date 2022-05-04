
use super::camera::*;
use ewgpu::*;
use super::wireframe_ppl::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WireframeVert{
    pub pos: [f32; 4],
    pub color: [f32; 4],
    // Has to be as [f32; 4] for alignment
    pub width: [f32; 4],
}

impl WireframeVert{
    pub fn new(pos: [f32; 3], color: [f32; 4], width: f32) -> Self{
        Self{
            pos: [pos[0], pos[1], pos[2], 1.],
            color,
            width: [width, 0., 0., 0.],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WidthUniform{
    pub width: [f32; 4],
}

#[derive(BindGroupContent)]
pub struct WidthBindGroupContent{
    uniform: Uniform<WidthUniform>,
}

impl BindGroupContentLayout for WidthBindGroupContent{
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("WidthBindGroupContent"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    ..wgsl::uniform_entry()
                }
            ]
        })
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Vert)]
pub struct WireframeMeshVert{
    #[location = 0]
    pos: [f32; 4],
    #[location = 1]
    color: [f32; 4],
}

#[derive(BindGroupContent)]
pub struct LineBindGroupContent{
    pub indices: Buffer<u32>,
    pub verts: Buffer<WireframeVert>,
}

impl BindGroupContentLayout for LineBindGroupContent{
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("LineBindGroupContent"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    ..wgsl::buffer_entry(false)
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    ..wgsl::buffer_entry(false)
                }
            ]
        })
    }
}

#[derive(BindGroupContent)]
pub struct MeshBindGroupContent{
    pub indices: Buffer<u32>,
    pub verts: Buffer<WireframeMeshVert>,
}

impl BindGroupContentLayout for MeshBindGroupContent{
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("LineBindGroupContent"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    ..wgsl::buffer_entry(false)
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    ..wgsl::buffer_entry(false)
                }
            ]
        })
    }
}

pub struct GPUWireframe{
    // Unused
    width: f32,

    //line_indices: BindGroup<Buffer<u32>>,
    //line_vertices: BindGroup<Buffer<WireframeVert>>,

    line: Bound<LineBindGroupContent>,
    mesh: Bound<MeshBindGroupContent>,

    //mesh_indices: BindGroup<Buffer<u32>>,
    //mesh_vertices: BindGroup<Buffer<WireframeMeshVert>>,
}

impl GPUWireframe{
    pub fn new(device: &wgpu::Device, vertices: &[WireframeVert], indices: &[u32], width: f32) -> Self{

        let line = LineBindGroupContent{
            indices: BufferBuilder::new()
                .storage().write().build(device, indices),
            verts: BufferBuilder::new()
                .storage().write()
                .build(device, vertices),
        }.into_bound(device);

        let mesh = MeshBindGroupContent{
            indices: BufferBuilder::new()
                .index().storage()
                .build_empty(device, line.indices.len() / 2 * 6),
            verts: BufferBuilder::new()
                .storage().vertex()
                .build_empty(device, line.indices.len() / 2 * 4)
        }.into_bound(device);

        //let width = UniformBindGroup::new(device, width);

        Self{
            line,
            mesh,
            width,
        }
    }
}

pub struct WireframeRenderer{
    line_cppl: WireframeMeshPipeline,
    mesh_rppl: WireframeRenderPipeline,

    width: Bound<Uniform<WidthUniform>>,
}

impl WireframeRenderer{
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self{

        /*
        let line_layout = pipeline_layout!(device,
            bind_groups: {
                line: BindGroup::<(Buffer<u32>, Buffer<WireframeVert>)> => wgpu::ShaderStages::all(),
                mesh: BindGroup::<(Buffer<u32>, Buffer<WireframeMeshVert>)> => wgpu::ShaderStages::all(),
                width: BoundUniform::<WidthUniform> => wgpu::ShaderStages::all(),
            },
            push_constants: {
                Camera => wgpu::ShaderStages::COMPUTE,
            }
        );
        */

        let line_cppl = WireframeMeshPipeline::new(device);

        let mesh_rppl = WireframeRenderPipeline::new(device, format);

        let width = Uniform::new(WidthUniform::default(), device).into_bound_with(device, &line_cppl.get_bind_group_layout(2));

        Self{
            line_cppl,
            mesh_rppl,
            width,
        }
    }

    pub fn update(&mut self, wireframe: &GPUWireframe, camera: &Camera, screen_size: [f32; 2], queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder){
        {
            self.width.borrow_mut(queue).width = [screen_size[0], screen_size[1], 0., wireframe.width];
        }
        {
            let data = ComputeData{
                bind_groups: vec![
                    (&wireframe.line).into(),
                    (&wireframe.mesh).into(),
                    (&self.width).into(),
                ],
                push_constants: &[
                    (0, bytemuck::bytes_of(camera)),
                ]
            };
            
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: None,
            });

            self.line_cppl.dispatch(&mut cpass, data, [(wireframe.line.indices.len() / 2) as u32, 1, 1]);
        }
    }

    pub fn render(&self, wireframe: &GPUWireframe, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView){
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label: None,
                color_attachments: &[
                    dst.color_attachment_load(),
                ],
                depth_stencil_attachment: None,
            });

            let data = RenderData{
                vertex_buffers: vec![
                    wireframe.mesh.verts.slice(..).into(),
                ],
                index_buffer: Some(
                    wireframe.mesh.indices.slice(..).into(),
                ),
                ..Default::default()
            };
            self.mesh_rppl.draw_indexed(&mut rpass, data, 0..wireframe.mesh.indices.len() as u32, 0, 0..1);
        }
    }

}

#[cfg(test)]
mod test{
    use ewgpu::*;
    use crate::wireframe::*;
    #[test]
    fn test(){
        let mut gpu = GPUContextBuilder::new()
            .set_features_util()
            .set_limits(wgpu::Limits{
                max_push_constant_size: 128,
                ..Default::default()})
            .build();

        let vertices = [
            WireframeVert{
                pos: [0., 1., 0., 0.],
                color: [1., 0., 0., 1.],
                width: [0.1, 0., 0., 0.],
            },
            WireframeVert{
                pos: [1., 0., 0., 0.],
                color: [0., 1., 0., 1.],
                width: [0.2, 0., 0., 0.],
            }
        ];

        let indices = [0, 1];
        let wireframe = GPUWireframe::new(&gpu.device, &vertices, &indices, 1.);

        let wireframe_renderer = WireframeRenderer::new(&gpu.device, wgpu::TextureFormat::Rgba8Unorm);

        let res = gpu.encode_img([800, 600], |gpu, view, encoder|{

            let norm_size = glm::vec2(800., 600.).normalize();
            let mvp = glm::perspective(800./600., std::f32::consts::PI / 2., 0.01, 100.) * glm::identity() * glm::identity();
            //let mvp = glm::identity();
            let camera = Camera{
                mvp: mvp.into(),
            };

            wireframe_renderer.render(&wireframe, encoder, view);

        }).save("output/test.png").unwrap();
    }
}

