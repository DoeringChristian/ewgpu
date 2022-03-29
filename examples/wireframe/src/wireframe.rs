
use super::camera::*;
use wgpu_utils::*;

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

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Vert)]
pub struct WireframeMeshVert{
    #[location = 0]
    pos: [f32; 4],
    #[location = 1]
    color: [f32; 4],
}

pub struct GPUWireframe{
    // Unused
    width: f32,

    //line_indices: BindGroup<Buffer<u32>>,
    //line_vertices: BindGroup<Buffer<WireframeVert>>,

    line: BindGroup<(Buffer<u32>, Buffer<WireframeVert>)>,
    mesh: BindGroup<(Buffer<u32>, Buffer<WireframeMeshVert>)>,

    //mesh_indices: BindGroup<Buffer<u32>>,
    //mesh_vertices: BindGroup<Buffer<WireframeMeshVert>>,
}

impl GPUWireframe{
    pub fn new(device: &wgpu::Device, vertices: &[WireframeVert], indices: &[u32], width: f32) -> Self{

        let line = BindGroup::new((
            BufferBuilder::new()
            .storage()
            .write()
            .append_slice(indices)
            .build(device),
            BufferBuilder::new()
            .storage()
            .write()
            .append_slice(vertices)
            .build(device)),
            device,
        );

        let mesh = BindGroup::new((
            BufferBuilder::new()
            .index()
            .storage()
            .build_empty(device, line.0.len()/2 * 6),
            BufferBuilder::new()
            .storage()
            .vertex()
            .build_empty(device, line.0.len()/2 * 4)),
            device
        );

        //let width = UniformBindGroup::new(device, width);

        Self{
            line,
            mesh,
            width,
        }
    }
}

pub struct WireframeRenderer{
    line_cppl: ComputePipeline,
    mesh_rppl: RenderPipeline,

    width: UniformBindGroup<WidthUniform>,
}

impl WireframeRenderer{
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self{

        let line_layout = pipeline_layout!(device,
            bind_groups: {
                line: BindGroup::<(Buffer<u32>, Buffer<WireframeVert>)> => wgpu::ShaderStages::all(),
                mesh: BindGroup::<(Buffer<u32>, Buffer<WireframeMeshVert>)> => wgpu::ShaderStages::all(),
                width: UniformBindGroup::<WidthUniform> => wgpu::ShaderStages::all(),
            },
            push_constants: {
                Camera => wgpu::ShaderStages::COMPUTE,
            }
        );

        let line_shader = ComputeShader::from_src(device, include_str!("shaders/line_ppl.glsl"), None).unwrap();

        let line_cppl = ComputePipelineBuilder::new(&line_shader)
            .set_layout(&line_layout)
            .build(device);

        let mesh_layout = pipeline_layout!(device,
            bind_groups:{
            },
            push_constants:{
            }
        );
        
        let vshader = VertexShader::from_src(device, include_str!("shaders/wf_mesh_rppl.glsl"), None).unwrap();
        let fshader = FragmentShader::from_src(device, include_str!("shaders/wf_mesh_rppl.glsl"), None).unwrap();

        let mesh_rppl = RenderPipelineBuilder::new(&vshader, &fshader)
            .set_layout(&mesh_layout)
            .push_vert_layout(WireframeMeshVert::buffer_layout())
            .push_target_replace(format)
            .build(device);

        let width = UniformBindGroup::new(device, WidthUniform::default());

        Self{
            line_cppl,
            mesh_rppl,
            width,
        }
    }

    pub fn update(&mut self, wireframe: &GPUWireframe, camera: &Camera, screen_size: [f32; 2], queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder){
        {
            self.width.borrow_ref(queue).width = [screen_size[0], screen_size[1], 0., wireframe.width];
        }
        {
            let mut cpass = ComputePass::new(encoder, None);

            let mut cpass_ppl = cpass.set_pipeline(&self.line_cppl);

            cpass_ppl.set_push_const(0, camera);
            cpass_ppl.set_bind_group(0, &wireframe.line, &[]);
            cpass_ppl.set_bind_group(1, &wireframe.mesh, &[]);
            cpass_ppl.set_bind_group(2, &self.width, &[]);
            cpass_ppl.dispatch((wireframe.line.0.len() / 2) as u32, 1, 1);
        }
    }

    pub fn render(&self, wireframe: &GPUWireframe, encoder: &mut wgpu::CommandEncoder, dst: &wgpu::TextureView){
        {
            let mut rpass = RenderPassBuilder::new()
                .push_color_attachment(dst.color_attachment_load())
                .begin(encoder, Some("Wireframe RenderPass"));

            let mut rpass_ppl = rpass.set_pipeline(&self.mesh_rppl);
            rpass_ppl.set_vertex_buffer(0, wireframe.mesh.1.buffer.slice(..));
            rpass_ppl.set_index_buffer(wireframe.mesh.0.buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass_ppl.draw_indexed(0..wireframe.mesh.0.len() as u32, 0, 0..1);
        }
    }

}

#[cfg(test)]
mod test{
    use wgpu_utils::*;
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

