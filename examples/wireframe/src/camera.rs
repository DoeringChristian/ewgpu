
use wgpu_utils::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera{
    pub mvp: [[f32; 4]; 4],
}
