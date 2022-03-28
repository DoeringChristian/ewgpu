
use crate::*;
use std::ops::Range;

///
/// Drawables can be drawn using a RenderPassPipeline.
///
/// The Function vert_buffer_layout can be used to extract the vertex buffer layout when creating a
/// RenderPipeline.
///
pub trait Drawable{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>);
    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>, range: Range<u32>);
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
