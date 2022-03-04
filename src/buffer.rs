use wgpu::util::DeviceExt;
use std::{marker::PhantomData, ops::{Deref, DerefMut, RangeBounds, Range}};
use std::mem::ManuallyDrop;
use std::ops::Bound;
use crate::utils::Align;

use super::binding;


// TODO: find a way to implement diffrent types of buffers.

/// 
/// A wrapper for the wgpu::BufferSlice but with its data exposed.
///
/// TODO: maybe imlement a BufferSliceMut
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
/// A builder for a Buffer.
///
/// Example: 
/// ```ignore 
/// let buffer = BufferBuilder::new()
///     .uniform().copy_dst()
///     .append_slice([0, 1, 2, 3])
///     .selt_label(Some("buffer"))
///     .build(device);
///
/// ```
///
pub struct BufferBuilder<'bb, C: bytemuck::Pod>{
    data: Vec<C>,
    usages: wgpu::BufferUsages,
    label: wgpu::Label<'bb>,
}

impl<'bb, C: bytemuck::Pod> BufferBuilder<'bb, C>{
    pub fn new() -> Self{
        Self{
            data: Vec::new(),
            usages: wgpu::BufferUsages::empty(),
            label: None,
        }
    }

    pub fn append_data(mut self, mut data: Vec<C>) -> Self{
        self.data.append(&mut data);
        self
    }

    pub fn append_slice(mut self, data: &[C]) -> Self{
        self.data.extend_from_slice(data);
        self
    }

    #[inline]
    pub fn vertex(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::VERTEX;
        self
    }
    #[inline]
    pub fn storage(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::STORAGE;
        self
    }
    #[inline]
    pub fn uniform(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::UNIFORM;
        self
    }
    #[inline]
    pub fn copy_dst(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::COPY_DST;
        self
    }
    #[inline]
    pub fn copy_src(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::COPY_SRC;
        self
    }
    #[inline]
    pub fn read(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::MAP_READ;
        self
    }
    #[inline]
    pub fn write(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::MAP_WRITE;
        self
    }

    #[inline]
    pub fn set_label(mut self, label: wgpu::Label<'bb>) -> Self{
        self.label = label;
        self
    }

    pub fn build(&self, device: &wgpu::Device) -> Buffer<C>{
        Buffer::<C>::new(device, self.usages, self.label, &self.data)
    }

    pub fn build_empty(&self, device: &wgpu::Device, len: usize) -> Buffer<C>{
        Buffer::<C>::new_empty(device, self.usages, self.label, len)
    }
}

///
/// TODO: Implement some way of differentiating buffer usages at compile time.
///
/// A typesafe wrapper for wgpu::Buffer.
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

    #[inline]
    pub fn new_vert(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::VERTEX, label, data)
    }

    #[inline]
    pub fn new_storage(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, label, data)
    }

    #[inline]
    pub fn new_index(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, wgpu::BufferUsages::INDEX, label, data)
    }

    #[inline]
    pub fn new_uniform(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, usage | wgpu::BufferUsages::UNIFORM, label, data)
    }
    #[inline]
    pub fn new_dst_uniform(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_uniform(device, wgpu::BufferUsages::COPY_DST, label, data)
    }
    #[inline]
    pub fn new_mapped(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        Self::new(device, usage | wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE, label, data)
    }
    #[inline]
    pub fn new_mapped_storage(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::STORAGE, label, data)
    }
    #[inline]
    pub fn new_mapped_index(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::INDEX, label, data)
    }
    #[inline]
    pub fn new_mapped_vert(device: &wgpu::Device, label: wgpu::Label, data: &[C]) -> Self{
        Self::new_mapped(device, wgpu::BufferUsages::VERTEX, label, data)
    }
    #[inline]
    pub fn len(&self) -> usize{
        self.len
    }

    // TODO: Export bound start and end to own functions.
    ///
    /// Slices have to be aligned with MAP_ALIGNMENT (8 bytes).
    /// TODO: for some reason the start_bound is multiplied by two.
    ///
    /// example: 
    /// ```rust
    ///# use wgpu_utils::*;
    ///# Framework::new([1920, 1080], |gpu|{
    ///     let buffer = BufferBuilder::<u64>::new()
    ///         .read().write()
    ///         .append_slice(&[0, 1, 2, 3])
    ///         .set_label(Some("buffer"))
    ///         .build(&gpu.device);
    ///    buffer.slice(0..).map_blocking_mut(&gpu.device)[0] = 8;
    ///    assert_eq!(buffer.slice(..).map_blocking(&gpu.device).as_ref(), [8, 1, 2, 3]);
    ///# });
    /// ```
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

        let len = end_bound - start_bound;

        // Get a range in bytes for wgpu::Buffer::slice(range)
        let range = (start_bound * std::mem::size_of::<C>() as u64)..(end_bound * std::mem::size_of::<C>() as u64);
        // TODO: Evaluate weather it is better to align the values or to let wgpu panic if not
        // aligned.
        //let range = range.start.align_floor(wgpu::MAP_ALIGNMENT)..range.end.align_ceil(wgpu::MAP_ALIGNMENT);
        //println!("{:?}", range);

        BufferSlice{
            buffer: self,
            slice: self.buffer.slice(range),
            offset: start_bound,
            len,
        }
    }

    // TODO: maybe move to slice.
    pub fn write_buffer(&mut self, queue: &wgpu::Queue, offset: usize, data: &[C]){
        queue.write_buffer(&self.buffer, (offset * std::mem::size_of::<C>()) as u64, bytemuck::cast_slice(data));
    }
}

impl<C: bytemuck::Pod> binding::BindGroupContent for Buffer<C>{
    fn entries(visibility: wgpu::ShaderStages) -> Vec<binding::BindGroupLayoutEntry>{
        vec!{
            binding::BindGroupLayoutEntry::new(visibility, binding::wgsl::buffer(false))
        }
    }

    fn resources<'br>(&'br self) -> Vec<wgpu::BindingResource<'br>> {
        vec!{
            self.as_entire_binding(),
        }
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

// TODO: Actually make buffer mutable
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

