pub trait VertLayout: bytemuck::Pod + bytemuck::Zeroable + Copy + Clone {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}

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
