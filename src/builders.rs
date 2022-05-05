
// TODO: Implement Builders for all wgpu types.

pub trait Builder{
    type Builder;
    fn builder() -> Self::Builder;
}

#[derive(DerefMut)]
pub struct BindGroupLayoutDescriptorBuilder<'a>{
    desc: wgpu::BindGroupLayoutDescriptor<'a>,
}

impl<'a> BindGroupLayoutDescriptorBuilder<'a>{
}

/*
impl<'a> Builder for wgpu::BindGroupLayout{
    type Builder = BindGroupLayoutDescriptorBuilder<'a>;

    fn builder() -> Self::Builder {
        BindGroupLayoutDescriptorBuilder{
            desc: wgpu::BindGroupLayoutDescriptor{
                label: None,
                entries: &[],
            }
        }
    }
}
*/
