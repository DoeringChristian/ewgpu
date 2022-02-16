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
    vert_buffer: Buffer<V>,
    idx_buffer: Buffer<u32>,
}

impl<V: VertLayout> Mesh<V>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32]) -> Result<Self>{
        let vertex_buffer = Buffer::new_vert(device, None, verts);

        let idx_buffer = Buffer::new_index(device, None, idxs);

        Ok(Self{
            idx_buffer,
            vert_buffer: vertex_buffer,
        })
    }
}

impl<V: VertLayout> Drawable for Mesh<V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        render_pass.set_vertex_buffer(0, self.vert_buffer.buffer.slice(..));
        render_pass.set_index_buffer(self.idx_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, 0..1);
    }
    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec!(V::buffer_layout())
    }
}

pub struct NMesh<D, V: VertBuffers<D>>{
    pub buffers: V,
    idx_buffer: Buffer<u32>,
    _ty: PhantomData<D>,
}

impl<D, V: VertBuffers<D>> NMesh<D, V>{
    pub fn new(device: &wgpu::Device, verts: D, idxs: &[u32]) -> Result<Self>{
        let idx_buffer = Buffer::new_index(device, None, idxs);
        let buffers = VertBuffers::<D>::new(device, verts);

        Ok(Self{
            buffers,
            idx_buffer,
            _ty: PhantomData,
        })
    }
}

impl<D, V: VertBuffers<D>> Drawable for NMesh<D, V>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        self.buffers.set_vertex_buffers(render_pass);
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

pub struct NIMesh<V, I, VB: VertBuffers<V>, IB: VertBuffers<I>>{
    pub vert_buffers: VB,
    pub inst_buffers: IB,

    _ty_v: PhantomData<V>,
    _ty_i: PhantomData<I>,
}

impl<V, I, VB: VertBuffers<V>, IB: VertBuffers<I>> NIMesh<V, I, VB, IB>{
    
}

impl<V, I, VB: VertBuffers<V>, IB: VertBuffers<I>> Drawable for NIMesh<V, I, VB, IB>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        
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
    //#[slot = 0]
    vertex_buffer: Buffer<V>,
    //#[slot = 1]
    instance_buffer: Buffer<I>,
    //#[index]
    idx_buffer: Buffer<u32>,
}

impl<V: VertLayout, I: VertLayout> IMesh<V, I>{
    pub fn new(device: &wgpu::Device, verts: &[V], idxs: &[u32], insts: &[I]) -> Result<Self>{
        let vertex_buffer = Buffer::new_vert(device, None, verts);
        let instance_buffer = Buffer::new_vert(device, None, insts);
        let idx_buffer = Buffer::new_index(device, None, idxs);

        Ok(Self{
            vertex_buffer,
            idx_buffer,
            instance_buffer,
        })
    }
}

// TODO: Change get_vert_buffer_layout to be able to set all layouts.
impl<V: VertLayout, I: VertLayout> Drawable for IMesh<V, I>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..));
        render_pass.set_index_buffer(self.idx_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, 0..self.instance_buffer.len() as u32);
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec!(V::buffer_layout(), I::buffer_layout())
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
        render_pass.set_bind_group(0, &self.uniform_buffer.get_bind_group(), &[]);

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


