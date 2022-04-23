use crate::*;

pub struct PushConstantLayout{
    pub stages: wgpu::ShaderStages,
    pub size: u32,
}

pub type PushConstantNone<C>           = PushConstant<{ShaderStages::NONE}, C>;
pub type PushConstantAll<C>            = PushConstant<{ShaderStages::ALL}, C>;
pub type PushConstantVertex<C>         = PushConstant<{ShaderStages::VERTEX}, C>;
pub type PushConstantFragment<C>       = PushConstant<{ShaderStages::FRAGMENT}, C>;
pub type PushConstantCompute<C>        = PushConstant<{ShaderStages::COMPUTE}, C>;
pub type PushConstantVertexFragment<C> = PushConstant<{ShaderStages::VERTEX_FRAGMENT}, C>;

pub struct PushConstant<const S: u32, T: bytemuck::Pod>{
    content: T,
}

impl<const S: u32, T: bytemuck::Pod> PushConstant<S, T>{

    pub fn new(content: T) -> Self{
        Self{
            content,
        }
    }
    pub fn push_const_layout() -> PushConstantLayout{
        PushConstantLayout{
            stages: wgpu::ShaderStages::from_bits_truncate(S),
            size: std::mem::size_of::<T>() as u32,
        }
    }
    pub fn push_const_slice<'pcr>(&'pcr self) -> (&'pcr[u8], wgpu::ShaderStages){
        (bytemuck::bytes_of(&self.content), wgpu::ShaderStages::from_bits_truncate(S))
    }
}

pub type PushConstantRefNone<'pcr, C>           = PushConstantRef<'pcr, {ShaderStages::NONE}, C>;
pub type PushConstantRefAll<'pcr, C>            = PushConstantRef<'pcr, {ShaderStages::ALL}, C>;
pub type PushConstantRefVertex<'pcr, C>         = PushConstantRef<'pcr, {ShaderStages::VERTEX}, C>;
pub type PushConstantRefFragment<'pcr, C>       = PushConstantRef<'pcr, {ShaderStages::FRAGMENT}, C>;
pub type PushConstantRefCompute<'pcr, C>        = PushConstantRef<'pcr, {ShaderStages::COMPUTE}, C>;
pub type PushConstantRefVertexFragment<'pcr, C> = PushConstantRef<'pcr, {ShaderStages::VERTEX_FRAGMENT}, C>;

pub struct PushConstantRef<'pcr, const S: u32, T: bytemuck::Pod>{
    content: &'pcr T,
}

impl<'pcr, const S: u32, T: bytemuck::Pod> PushConstantRef<'pcr, S, T>{
    pub fn new(content: &'pcr T) -> Self{
        Self{
            content,
        }
    }
    pub fn push_const_layout() -> PushConstantLayout{
        PushConstantLayout{
            stages: wgpu::ShaderStages::from_bits_truncate(S),
            size: std::mem::size_of::<T>() as u32,
        }
    }
    pub fn push_const_slice(&'pcr self) -> (&'pcr[u8], wgpu::ShaderStages){
        (bytemuck::bytes_of(self.content), wgpu::ShaderStages::from_bits_truncate(S))
    }
}
