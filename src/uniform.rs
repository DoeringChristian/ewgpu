use super::binding::BindGroup;
use super::binding::CreateBindGroupLayout;
use super::buffer::*;
use super::binding;
use std::ops::{Deref, DerefMut};

///
/// A struct mutably referencing a Uniform to edit its content and update it when UniformRef is
/// droped.
///
pub struct UniformRefMut<'ur, C: bytemuck::Pod>{
    queue: &'ur wgpu::Queue,
    uniform: &'ur mut Uniform<C>,
}


impl<C: bytemuck::Pod> Deref for UniformRefMut<'_, C>{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.uniform.uniform_vec.content[0]
    }
}

impl<C: bytemuck::Pod> DerefMut for UniformRefMut<'_, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform.uniform_vec.content[0]
    }
}

impl<C: bytemuck::Pod> Drop for UniformRefMut<'_, C>{
    fn drop(&mut self) {
        self.uniform.uniform_vec.update_int(self.queue);
    }
}

///
/// A struct mutably referencing a UniformVec to edit its content and update it when UniformVecRef is
/// droped.
///
pub struct UniformVecRefMut<'ur, C: bytemuck::Pod>{
    queue: &'ur mut wgpu::Queue,
    uniform_vec: &'ur mut UniformVec<C>,
}

impl<C: bytemuck::Pod> Deref for UniformVecRefMut<'_, C>{
    type Target = [C];

    fn deref(&self) -> &Self::Target{
        &self.uniform_vec.content
    }
}

impl<C: bytemuck::Pod> DerefMut for UniformVecRefMut<'_, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform_vec.content
    }
}

impl<C: bytemuck::Pod> Drop for UniformVecRefMut<'_, C>{
    fn drop(&mut self){
        self.uniform_vec.update_int(self.queue);
    }
}

///
/// A Buffer with a vector that can be used as a Unform directly.
/// It keeps a copy of the content in memory for easier updates.
/// TODO: Remove content.
///
pub struct UniformVec<C: bytemuck::Pod>{
    buffer: Buffer<C>,

    content: Vec<C>,
}

impl<C: bytemuck::Pod> UniformVec<C>{
    fn name() -> &'static str{
        let type_name = std::any::type_name::<C>();
        let pos = type_name.rfind(':').unwrap_or(0);
        &type_name[(pos + 1)..]
    }

    pub fn new(src: &[C], device: &wgpu::Device) -> Self{
        let buffer = BufferBuilder::new()
            .uniform().copy_dst()
            .set_label(Some(&format!("UniformBuffer: {}", Self::name())))
            .build(device, src);

        Self{
            buffer,
            content: Vec::from(src),
        }
    }

    pub fn update_int(&mut self, queue: &wgpu::Queue){
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.content));
    }

    pub fn borrow_mut<'ur>(&'ur mut self, queue: &'ur mut wgpu::Queue) -> UniformVecRefMut<'ur, C>{
        UniformVecRefMut{
            queue,
            uniform_vec: self,
        }
    }

    #[inline]
    pub fn len(&self) -> usize{
        self.content.len()
    }
}

impl<C: bytemuck::Pod> binding::BindGroupContent for UniformVec<C>{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<binding::BindGroupLayoutEntry>{
        vec!{
            binding::BindGroupLayoutEntry::new(visibility, binding::wgsl::uniform()),
        }
    }

    fn resources(& self) -> Vec<wgpu::BindingResource> {
        vec!{
            self.buffer.as_entire_binding(),
        }
    }
}

///
/// A UniformVec with a single element usefull for cameras etc.
///
pub struct Uniform<C: bytemuck::Pod>{
    uniform_vec: UniformVec<C>,
}

impl<C: bytemuck::Pod> Uniform<C>{
    pub fn new(src: C, device: &wgpu::Device) -> Self{
        Self{
            uniform_vec: UniformVec::new(&[src], device)
        }
    }

    pub fn borrow_mut<'ur>(&'ur mut self, queue: &'ur wgpu::Queue) -> UniformRefMut<'ur, C>{
        UniformRefMut{
            queue,
            uniform: self,
        }
    }
}

impl<C: bytemuck::Pod> binding::BindGroupContent for Uniform<C>{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<binding::BindGroupLayoutEntry>{
        vec!{
            binding::BindGroupLayoutEntry::new(visibility, binding::wgsl::uniform())
        }
    }

    fn resources(&self) -> Vec<wgpu::BindingResource> {
        vec!{
            self.uniform_vec.buffer.as_entire_binding(),
        }
    }
}

///
/// A uniform inside a BindGroup
///
pub struct UniformBindGroup<C: bytemuck::Pod>{
    bind_group: binding::BindGroup<Uniform<C>>,
}

impl <C: bytemuck::Pod> UniformBindGroup<C>{
    pub fn new(device: &wgpu::Device, src: C) -> Self{
        Self{
            bind_group: binding::BindGroup::new(Uniform::new(src, device), device)
        }
    }
}

impl<C: bytemuck::Pod> binding::GetBindGroup for UniformBindGroup<C>{
    fn get_bind_group(&self) -> &wgpu::BindGroup {
        self.bind_group.get_bind_group()
    }
}

impl<C: bytemuck::Pod> Deref for UniformBindGroup<C>{
    type Target = binding::BindGroup<Uniform<C>>;

    fn deref(&self) -> &Self::Target {
        &self.bind_group
    }
}

impl<C: bytemuck::Pod> DerefMut for UniformBindGroup<C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bind_group
    }
}

impl<C: bytemuck::Pod> CreateBindGroupLayout for UniformBindGroup<C>{
    fn create_bind_group_layout(device: &wgpu::Device, label: Option<&str>) -> binding::BindGroupLayoutWithDesc {
        BindGroup::<Uniform<C>>::create_bind_group_layout(device, label)
    }
    fn create_bind_group_layout_vis(device: &wgpu::Device, label: Option<&str>, visibility: wgpu::ShaderStages) -> crate::BindGroupLayoutWithDesc {
        BindGroup::<Uniform<C>>::create_bind_group_layout_vis(device, label, visibility)
    }
}
