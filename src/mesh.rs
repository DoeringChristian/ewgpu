use std::ops::Range;
use crate::*;
#[allow(unused)]
use wgpu::util::DeviceExt;
use anyhow::*;

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
    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>) {
        self.nmesh.draw_indexed(render_pass, range)
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
        self.draw_indexed(render_pass, 0..1)
    }
    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>) {
        self.buffers.push_vertex_buffers_to(render_pass);
        render_pass.set_index_buffer(self.idx_buffer.slice(..));

        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, range);
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
        self.draw_indexed(render_pass, self.inst_buffers.get_min_range())
    }

    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>) {
        self.vert_buffers.push_vertex_buffers_to(render_pass);
        self.inst_buffers.push_vertex_buffers_to(render_pass);

        render_pass.set_index_buffer(self.idx_buffer.slice(..));

        render_pass.draw_indexed(0..self.idx_buffer.len() as u32, 0, range);
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

    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>) {
        self.nimesh.draw_indexed(render_pass, range)
    }

    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>> {
        Self::create_vert_buffer_layouts()
    }
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        NIMesh::<(Buffer<V>, ), (Buffer<I>, )>::create_vert_buffer_layouts()
    }
}

