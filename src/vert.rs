use super::buffer::*;
use wgpu_utils_macros::Vert;
use super::pipeline;
use std::ops::Range;

// Possibly remove generic D
pub trait VertBuffers{
    fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>;
    fn push_vertex_buffers_to<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>);
    fn get_min_range(&self) -> Range<u32>;
}

macro_rules! vert_buffer_for_tuple{
    ($($name:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($name: VertLayout),+> VertBuffers for ($(Buffer<$name>, )+){
            /*
               fn new<($(&[$name], )+)>(device: &wgpu::Device, data: ($(&[$name], )+)) -> Self{
               let ($($name, )+) = data;
               let ($($name, )+) = ($(Buffer::new_vert(device, None, $name), )+);
               ($($name,)+)
               }
            */

            fn create_vert_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>{
                vec!($($name::buffer_layout(),)+)
            }

            fn push_vertex_buffers_to<'rp>(&'rp self, render_pass: &'_ mut pipeline::RenderPassPipeline<'rp, '_>){
                let ($($name, )+) = self;
                ($(render_pass.push_vertex_buffer($name.buffer.slice(..)),)+);
            }


            fn get_min_range(&self) -> Range<u32>{
                let start_idx = 0;
                let mut end_idx = std::u32::MAX;
                let ($($name, )+) = self;

                $(
                    if ($name.len() as u32) < end_idx{
                        end_idx = $name.len() as u32;
                    }
                )+;
                start_idx..end_idx
            }

        }
    }
}

// TODO: Add a derive macro for stucts.
vert_buffer_for_tuple!{ A }
vert_buffer_for_tuple!{ A B }
vert_buffer_for_tuple!{ A B C }
vert_buffer_for_tuple!{ A B C D }
vert_buffer_for_tuple!{ A B C D E }
vert_buffer_for_tuple!{ A B C D E F }
vert_buffer_for_tuple!{ A B C D E F G }
vert_buffer_for_tuple!{ A B C D E F G H }
vert_buffer_for_tuple!{ A B C D E F G H I }
vert_buffer_for_tuple!{ A B C D E F G H I J }
vert_buffer_for_tuple!{ A B C D E F G H I J K }
vert_buffer_for_tuple!{ A B C D E F G H I J K L }


pub trait VertLayout: bytemuck::Pod + bytemuck::Zeroable + Copy + Clone {
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
   wgpu::VertexAttribute
   }
   }
   */


#[cfg(test)]
mod tests {
    use crate::*;

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
    #[derive(Vert)]
    struct VertTest8{
        // uint8 not working
        #[location = 0]
        uint8x2: [u8; 2],
        #[location = 1]
        uint8x4: [u8; 4],
        #[location = 2]
        #[norm]
        unorm8x2: [u8; 2],
        #[location = 3]
        #[norm]
        unorm8x4: [u8; 4],
        #[location = 4]
        sint8x2: [i8; 2],
        #[location = 5]
        sint8x4: [i8; 4],
        #[location = 6]
        #[norm]
        snorm8x2: [i8; 2],
        #[location = 7]
        #[norm]
        snorm8x4: [i8; 4],
   }

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
    #[derive(Vert)]
    struct VertTest16{
        #[location = 8]
        uint16x2: [u16; 2],
        #[location = 9]
        uint16x4: [u16; 4],
        #[location = 10]
        #[norm]
        unorm16x2: [u16; 2],
        #[location = 11]
        #[norm]
        unorm16x4: [u16; 2],
        #[location = 12]
        sint16x2: [i16; 2],
        #[location = 13]
        sint16x4: [i16; 4],
        #[location = 14]
        #[norm]
        snorm16x2: [i16; 2],
        #[location = 15]
        #[norm]
        snorm16x4: [i16; 4],
   }

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
    #[derive(Vert)]
    struct VertTest32{
        #[location = 16]
        uint32: u32,
        #[location = 17]
        uint32x2: [u32; 2],
        #[location = 18]
        uint32x3: [u32; 3],
        #[location = 19]
        uint32x4: [u32; 4],
        #[location = 20]
        sint32: i32,
        #[location = 21]
        sint32x2: [i32; 2],
        #[location = 22]
        sint32x3: [i32; 3],
        #[location = 23]
        sint32x4: [i32; 4],
        #[location = 24]
        float32: f32,
        #[location = 25]
        float32x2: [f32; 2],
        #[location = 26]
        float32x3: [f32; 3],
        #[location = 27]
        float32x4: [f32; 4],
        #[location = 28]
        float64x2: [f64; 2],
        #[location = 29]
        float64x3: [f64; 3],
        #[location = 30]
        float64x4: [f64; 4],
   }

    // TODO: write the tests.
    #[test]
    fn test_vert8_buffer_layout(){
        let layout = VertTest8::buffer_layout();

        let layout_cmp = wgpu::VertexBufferLayout{
            array_stride: 24,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
            ]
        };
    }

    #[test]
    fn test_vert16_buffer_layout(){
        let layout = VertTest16::buffer_layout();
    }

    #[test]
    fn test_vert32_buffer_layout(){
        let layout = VertTest32::buffer_layout();
    }
}
