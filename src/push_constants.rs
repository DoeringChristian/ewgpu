
///
/// A trait implemented by all types that can be used as push constants.
///
/// By default all types that derive bytemuck::Pod can be push constants.
///
pub struct PushConstantLayout{
    pub stages: wgpu::ShaderStages,
    pub size: u32,
}

pub trait PushConstant: bytemuck::Pod{
    fn push_const_layout(stages: wgpu::ShaderStages) -> PushConstantLayout;
    fn as_slice8(&self) -> &[u8]{
        bytemuck::bytes_of(self)
    }
}

impl<T: bytemuck::Pod> PushConstant for T{
    fn push_const_layout(stages: wgpu::ShaderStages) -> PushConstantLayout {
        PushConstantLayout{
            stages,
            size: std::mem::size_of::<T>() as u32,
        }
    }
}

pub struct PushConstantVec<T: bytemuck::Pod>{
    pub layout: PushConstantLayout,
    pub content: T,
}
