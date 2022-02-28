
// TODO: better way to implement push const range
pub struct PushConstantLayout{
    pub stages: wgpu::ShaderStages,
    pub size: u32,
}

pub trait PushConstant: bytemuck::Pod{
    fn push_const_layout(stages: wgpu::ShaderStages) -> PushConstantLayout;
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

#[cfg(test)]
mod tests{
    #[allow(unused)]
    use super::*;

    #[test]
    fn push_consts(){
    }
}
