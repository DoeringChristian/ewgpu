use super::binding;
use super::pipeline;
use super::vert::*;
use super::buffer::*;
use super::uniform::UniformBindGroup;
use super::binding::GetBindGroup;
use bytemuck;
use cgmath::*;
#[allow(unused)]
use wgpu::util::DeviceExt;
use anyhow::*;
use std::marker::PhantomData;
use std::ops::Range;

///
/// Drawables can be drawn using a RenderPassPipeline.
///
/// The Function vert_buffer_layout can be used to extract the vertex buffer layout when creating a
/// RenderPipeline.
///
pub trait Drawable{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>);
    //fn draw_instanced<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>);
    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>>;
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>;
}

pub trait UpdatedDrawable<D>: Drawable{
    fn update(&mut self, queue: &mut wgpu::Queue, data: &D);
    fn update_draw<'rp>(&'rp mut self, queue: &mut wgpu::Queue, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, data: &D){
        self.update(queue, data);
        self.draw(render_pass)
    }
}

///
/// A DataDrawable is Anything that can be drawn without knowing the pipeline to use own pipeline.
///
/// It has to get data from somewhere for example textures to draw to.
///
pub trait DataDrawable<'pd, D>{
    fn draw_data(&'pd self, render_pass: &'_ mut pipeline::RenderPass<'pd>, data: D);
}

pub struct Mesh<V: VertLayout>{
    nmesh: NMesh<(Buffer<V>,)>,
}

impl<V: VertLayout> Mesh<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{
        let vert_buffer = Buffer::new_vert(device, None, verts);

        let nmesh = NMesh::new(device, (vert_buffer, ), idxs)?;

        Ok(Self{
            nmesh,
        })
    }
}

impl<V: VertLayout> Drawable for Mesh<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        self.nmesh.draw(render_pass);
    }
    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        NMesh::<(Buffer<V>, )>::create_vert_buffer_layouts()
    }
}

pub struct NMesh<V: VertBuffers>{
    pub buffers: V,
    idx_buffer: Buffer<u32>,
}

impl<V: VertBuffers> NMesh<V>{
    pub fn new(device: &wgpu::Device, buffers: V, idxs: &[u32]) -> Result<Self>{
        let idx_buffer = Buffer::new_index(device, None, idxs);
        //let buffers = VertBuffers::<D>::new(device, verts);

        Ok(Self{
            buffers,
            idx_buffer,
        })
    }
}

impl<V: VertBuffers> Drawable for NMesh<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        self.buffers.push_vertex_buffers_to(render_pass);
        render_pass.set_index_buffer(self.idx_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, 0..1);
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        V::create_vert_buffer_layouts()
    }

    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        V::create_vert_buffer_layouts()
    }
}

pub struct NIMesh<VB: VertBuffers, IB: VertBuffers>{
    pub vert_buffers: VB,
    pub inst_buffers: IB,
    idx_buffer: Buffer<u32>,
}

impl<VB: VertBuffers, IB: VertBuffers> NIMesh<VB, IB>{
    pub fn new(device: &wgpu::Device, vert_buffers: VB, inst_buffers: IB, idxs: &[u32]) -> Result<Self>{
        let idx_buffer = Buffer::<u32>::new_index(device, None, idxs);

        Ok(Self{
            idx_buffer,
            vert_buffers,
            inst_buffers,
        })
    }
}

impl<VB: VertBuffers, IB: VertBuffers> Drawable for NIMesh<VB, IB>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        self.vert_buffers.push_vertex_buffers_to(render_pass);
        self.inst_buffers.push_vertex_buffers_to(render_pass);

        render_pass.set_index_buffer(self.idx_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, self.inst_buffers.get_min_range());
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }

    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        let mut vb_layout = VB::create_vert_buffer_layouts();
        let mut ib_layout = IB::create_vert_buffer_layouts();
        vb_layout.append(&mut ib_layout);
        vb_layout
    }
}

pub struct IMesh<V: VertLayout, I: VertLayout>{
    nimesh: NIMesh<(Buffer<V>, ), (Buffer<I>, )>,
}

impl<V: VertLayout, I: VertLayout> IMesh<V, I>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32], insts: &[I]) -> Result<Self>{
        let vert_buffer = Buffer::new_vert(device, None, verts);
        let inst_buffer = Buffer::new_vert(device, None, insts);

        let nimesh = NIMesh::new(device, (vert_buffer, ), (inst_buffer, ), idxs)?;

        Ok(Self{
            nimesh,
        })
    }
}

// TODO: Change get_vert_buffer_layout to be able to set all layouts.
impl<V: VertLayout, I: VertLayout> Drawable for IMesh<V, I>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        self.nimesh.draw(render_pass);
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        NIMesh::<(Buffer<V>, ), (Buffer<I>, )>::create_vert_buffer_layouts()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelTransforms{
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

pub struct Model<V: VertLayout>{
    mesh: Mesh<V>,
    uniform_buffer: UniformBindGroup<ModelTransforms>,
}

impl<V: VertLayout> Model<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{

        let mesh = Mesh::<V>::new(device, verts, idxs)?;

        let model = glm::Mat4::identity();
        let view = glm::Mat4::identity();
        let proj = glm::Mat4::identity();
        let model_transforms = ModelTransforms{
            model: model.into(),
            view: view.into(),
            proj: proj.into()
        };
        let uniform_buffer = UniformBindGroup::new(device, model_transforms);

        Ok(Self{
            mesh,
            uniform_buffer,
        })
    }
}

impl<V: VertLayout> Drawable for Model<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        render_pass.set_bind_group(0, &self.uniform_buffer, &[]);

        self.mesh.draw(render_pass);
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }

    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec!(V::buffer_layout())
    }
}

impl<V: VertLayout> UpdatedDrawable<ModelTransforms> for Model<V>{
    fn update(&mut self, queue: &mut wgpu::Queue, data: &ModelTransforms) {
        *self.uniform_buffer.borrow_ref(queue) = *data;
    }
}


