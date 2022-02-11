use super::buffer::*;
use bytemuck::*;
use memoffset;
use wgpu::util::DeviceExt;
use wgpu_utils_macros::Vert;

pub trait Vert: bytemuck::Pod + bytemuck::Zeroable + Copy + Clone {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
#[derive(Vert)]
pub struct Vert2 {
    #[location = 0]
    pub pos: [f32; 2],
    #[location = 1]
    pub uv: [f32; 2],
}

impl Vert2 {
    pub const QUAD_VERTS: [Vert2; 4] = [
        Vert2 {
            pos: [-1.0, -1.0],
            uv: [0.0, 1.0],
        },
        Vert2 {
            pos: [1.0, -1.0],
            uv: [1.0, 1.0],
        },
        Vert2 {
            pos: [1.0, 1.0],
            uv: [1.0, 0.0],
        },
        Vert2 {
            pos: [-1.0, 1.0],
            uv: [0.0, 0.0],
        },
    ];

    pub const QUAD_IDXS: [u32; 6] = [0, 1, 2, 2, 3, 0];
}

/*

impl Vert for Vert2{
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        let o = memoffset::offset_of!(Vert2, pos);
        const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2
        ];
        let offset = 0;
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
*/

pub trait Instance: bytemuck::Pod + bytemuck::Zeroable + Copy + Clone {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}
