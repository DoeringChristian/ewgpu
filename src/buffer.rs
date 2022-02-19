use anyhow::*;
use wgpu::util::DeviceExt;
use std::{marker::PhantomData, ops::{Deref, DerefMut, RangeBounds}, borrow::{Borrow, BorrowMut}, future, cell::Cell};
use binding::CreateBindGroupLayout;
use std::mem::ManuallyDrop;
use std::ops::Bound;

use super::binding;


// TODO: find a way to implement diffrent types of buffers.

pub struct BufferSlice<'bs, C: bytemuck::Pod>{
    buffer: &'bs Buffer<C>,
    slice: wgpu::BufferSlice<'bs>,
    offset: wgpu::BufferAddress,
    len: wgpu::BufferAddress,
}

impl<'bs, C: bytemuck::Pod> BufferSlice<'bs, C>{
    ///
    /// Map the slice whilst polling the device.
    ///
    pub async fn map_async_poll(self, device: &wgpu::Device) -> BufferView<'bs, C>{
        let mapping = self.slice.map_async(wgpu::MapMode::Read);

        device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();

        BufferView{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(self.slice.get_mapped_range()),
        }
    }

    ///
    /// Map the slice asynchronously.
    /// wgpu::Device::poll has to be called before this Future will complete.
    ///
    pub async fn map_async(self) -> BufferView<'bs, C>{
        let mapping = self.slice.map_async(wgpu::MapMode::Read);

        mapping.await.unwrap();

        BufferView{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(self.slice.get_mapped_range()),
        }
    }

    ///
    /// Map the slice and block this thread untill maping is complete.
    ///
    /// Example:
    /// 
    /// let array = [0, 1, 2, 3, 4];
    /// let mapped_buffer = MappedBuffer::new_storage(device, None, array);
    ///
    /// mapped_buffer.slice_blocking(..)[0] = 1;
    ///
    /// let i = mapped_buffer.slice(..)[0];
    /// 
    pub fn map_blocking(self, device: &wgpu::Device) -> BufferView<'bs, C>{
        pollster::block_on(self.map_async_poll(device))
    }

    ///
    /// Map the slice mutably whilst polling the device.
    ///
    ///
    pub async fn map_async_poll_mut(self, device: &wgpu::Device) -> BufferViewMut<'bs, C>{
        let mapping = self.slice.map_async(wgpu::MapMode::Write);

        device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();

        BufferViewMut{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(self.slice.get_mapped_range_mut()),
        }
    }

    ///
    /// Map the slice asynchronously for writing to the buffer.
    /// wgpu::Device::poll has to be called before this Future will complete.
    ///
    pub async fn map_async_mut(self) -> BufferViewMut<'bs, C>{
        let mapping = self.slice.map_async(wgpu::MapMode::Write);

        mapping.await.unwrap();

        BufferViewMut{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(self.slice.get_mapped_range_mut()),
        }
    }

    ///
    /// Map the slice mutably and block this thread untill maping is complete.
    ///
    /// slice.map_blocking_mut(device)[0] = 1;
    ///
    pub fn map_blocking_mut(self, device: &wgpu::Device) -> BufferViewMut<'bs, C>{
        pollster::block_on(self.map_async_poll_mut(device))
    }

    pub fn copy_to_buffer(&self, dst: &mut Buffer<C>, offset: &wgpu::BufferAddress, encoder: &mut wgpu::CommandEncoder){
        let src_offset_bytes = self.offset * std::mem::size_of::<C>() as u64;
        let size_bytes = self.len * std::mem::size_of::<C>() as u64;

        let offset_bytes = offset * std::mem::size_of::<C>() as u64;

        encoder.copy_buffer_to_buffer(
            &self.buffer.buffer,
            src_offset_bytes,
            &dst.buffer,
            offset_bytes,
            size_bytes
        );
    }
}

///
/// TODO: Implement some way of differentiating buffer usages at compile time.
///
pub struct Buffer<C: bytemuck::Pod>{
    pub buffer: wgpu::Buffer,
    len: usize,
    _pd: PhantomData<C>,
}

impl<C: bytemuck::Pod> Buffer<C>{
    pub fn new_empty(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, len: usize) -> Self{
        let buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label,
            size: (len * std::mem::size_of::<C>()) as u64,
            usage,
            mapped_at_creation: false,
        });

        Self{
            buffer,
            len,
            _pd: PhantomData,
        }
    }

    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label,
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self{
            buffer,
            len: data.len(),
            _pd: PhantomData,
        }
    }

    pub fn new_vert(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::VERTEX, label, data)
    }

    pub fn new_storage(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, label, data)
    }

    pub fn new_index(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::INDEX, label, data)
    }

    pub fn new_uniform(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, usage | wgpu::BufferUsages::UNIFORM, label, data)
    }

    pub fn new_dst_uniform(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_uniform(device, wgpu::BufferUsages::COPY_DST, label, data)
    }

    pub fn new_mapped(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, usage | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, label, data)
    }

    pub fn new_mapped_storage(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::STORAGE, label, data)
    }

    pub fn new_mapped_index(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::INDEX, label, data)
    }

    pub fn new_mapped_vert(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::VERTEX, label, data)
    }

    pub fn len(&self) -> usize{
        self.len
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> BufferSlice<C>{
        let start_bound = bounds.start_bound();
        let end_bound = bounds.end_bound();

        let start_bound = match start_bound{
            Bound::Unbounded => 0 as wgpu::BufferAddress,
            Bound::Included(offset) => {(offset + 0).max(0)},
            Bound::Excluded(offset) => {(offset + 1).max(0)},
        };

        let end_bound = match end_bound{
            Bound::Unbounded => {(self.len()) as wgpu::BufferAddress},
            Bound::Included(offset) => {(offset + 1).min(self.len() as u64)},
            Bound::Excluded(offset) => {(offset + 0).min(self.len() as u64)},
        };

        let start_bound = start_bound;
        let end_bound = end_bound;

        let size = end_bound - start_bound;

        BufferSlice{
            buffer: self,
            slice: self.buffer.slice(bounds),
            offset: start_bound,
            len: size,
        }
    }

    pub fn write_buffer(&mut self, queue: &wgpu::Queue, offset: usize, data: &[C]){
        queue.write_buffer(&self.buffer, (offset * std::mem::size_of::<C>()) as u64, bytemuck::cast_slice(data));
    }
}

impl<C: bytemuck::Pod> binding::BindGroupContent for Buffer<C>{
    fn push_entries_to(bind_group_layout_builder: &mut binding::BindGroupLayoutBuilder, visibility: wgpu::ShaderStages) {
        bind_group_layout_builder.push_entry_ref(visibility, binding::wgsl::buffer(false))
    }

    fn push_resources_to<'bgb>(&'bgb self, bind_group_builder: &mut binding::BindGroupBuilder<'bgb>) {
        bind_group_builder.resource_ref(self.as_entire_binding())
    }
}

impl<C: bytemuck::Pod> Deref for Buffer<C>{
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<C: bytemuck::Pod> DerefMut for Buffer<C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub struct BufferView<'mbr, C: bytemuck::Pod>{
    buffer: &'mbr Buffer<C>,
    buffer_view: ManuallyDrop<wgpu::BufferView<'mbr>>,
}

impl<'mbr, C: bytemuck::Pod> AsRef<[C]> for BufferView<'mbr, C>{
    fn as_ref(&self) -> &[C] {
        bytemuck::cast_slice(self.buffer_view.as_ref())
    }
}

impl<'mbr, C: bytemuck::Pod> Deref for BufferView<'mbr, C>{
    type Target = [C];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(self.buffer_view.as_ref())
    }
}

impl<'mbr, C: bytemuck::Pod> Drop for BufferView<'mbr, C>{
    fn drop(&mut self) {
        // SAFETY: Dropping buffer view before unmap is required.
        // self.buffer_view is also not used afterwards.
        unsafe{
            ManuallyDrop::drop(&mut self.buffer_view);
        }
        self.buffer.unmap();
    }
}

pub struct BufferViewMut<'mbr, C: bytemuck::Pod>{
    buffer: &'mbr Buffer<C>,
    buffer_view: ManuallyDrop<wgpu::BufferViewMut<'mbr>>,
}

impl<'mbr, C: bytemuck::Pod> AsMut<[C]> for BufferViewMut<'mbr, C>{
    fn as_mut(&mut self) -> &mut [C] {
        bytemuck::cast_slice_mut(self.buffer_view.as_mut())
    }
}

impl<'mbr, C: bytemuck::Pod> Deref for BufferViewMut<'mbr, C>{
    type Target = [C];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(self.buffer_view.as_ref())
    }
}

impl<'mbr, C: bytemuck::Pod> DerefMut for BufferViewMut<'mbr, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        bytemuck::cast_slice_mut(self.buffer_view.as_mut())
    }
}

impl<'mbr, C: bytemuck::Pod> Drop for BufferViewMut<'mbr, C>{
    fn drop(&mut self) {
        // SAFETY: Dropping buffer view before unmap is required.
        // self.buffer_view is also not used afterwards.
        unsafe{
            ManuallyDrop::drop(&mut self.buffer_view);
        }
        self.buffer.buffer.unmap();
    }
}
