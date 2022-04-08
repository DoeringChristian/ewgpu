
use crate::*;
use std::ops::Range;

/*
///
/// Drawables can be drawn using a RenderPassPipeline.
///
/// The Function vert_buffer_layout can be used to extract the vertex buffer layout when creating a
/// RenderPipeline.
///
pub trait Drawable<'rd, RD: RenderData<'rd>>{
    fn draw<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_, RD>);
    fn draw_indexed<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_, RD>, range: Range<u32>);
    fn get_vert_buffer_layouts(&self) -> Vec<wgpu::VertexBufferLayout<'static>>;
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>;
}
*/
