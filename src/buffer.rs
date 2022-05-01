use ewgpu_macros::DerefMut;
use wgpu::util::DeviceExt;
use std::{marker::PhantomData, ops::{Deref, DerefMut, RangeBounds, Range}};
use std::mem::ManuallyDrop;
use crate::utils::*;

use super::binding;

// TODO: find a way to implement diffrent types of buffers.

/// 
/// A wrapper for the wgpu::BufferSlice but with its data exposed.
/// This can be used to either copy to another buffer or read/write from/to it.
///
pub struct BufferSlice<'bs, C: bytemuck::Pod>{
    buffer: &'bs Buffer<C>,
    range: Range<usize>,
}

impl<'bs, C: bytemuck::Pod> From<BufferSlice<'bs, C>> for wgpu::BufferSlice<'bs>{
    fn from(src: BufferSlice<'bs, C>) -> Self {
        src.buffer.buffer.slice(src.range_addr())
    }
}

impl<'bs, C: bytemuck::Pod> BufferSlice<'bs, C>{
    ///
    /// Convert the range of elements (BufferSlice::range) into a range of bytes.
    ///
    #[inline]
    fn range_addr(&self) -> Range<wgpu::BufferAddress>{
        ((self.range.start * std::mem::size_of::<C>()) as u64)..((self.range.end * std::mem::size_of::<C>()) as u64)
    }

    ///
    /// Map the slice whilst polling the device.
    ///
    pub async fn map_async_poll(&self, device: &wgpu::Device) -> BufferView<'bs, C>{
        let slice = self.buffer.buffer.slice(self.range_addr());
        let mapping = slice.map_async(wgpu::MapMode::Read);

        device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();

        BufferView{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(slice.get_mapped_range()),
        }
    }

    ///
    /// Map the slice asynchronously.
    /// wgpu::Device::poll has to be called before this Future will complete.
    ///
    pub async fn map_async(&self) -> BufferView<'bs, C>{
        let slice = self.buffer.buffer.slice(self.range_addr());
        let mapping = slice.map_async(wgpu::MapMode::Read);

        mapping.await.unwrap();

        BufferView{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(slice.get_mapped_range()),
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
    pub fn map_blocking(&self, device: &wgpu::Device) -> BufferView<'bs, C>{
        pollster::block_on(self.map_async_poll(device))
    }

    ///
    /// Copy the buffer slice to another buffer.
    /// Only the bytes that fit into the destination buffer are copied.
    ///
    /// ```rust
    /// use ewgpu::*;
    ///
    /// let instance = wgpu::Instance::new(wgpu::Backends::all());
    ///
    /// let mut gpu = GPUContextBuilder::new()
    ///     .enable_feature(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
    ///     .set_limits(wgpu::Limits{
    ///         max_push_constant_size: 128,
    ///         ..Default::default()
    ///     }).build();
    ///
    /// let buffer1 = BufferBuilder::<u64>::new()
    ///                 .read().write().copy_src()
    ///                 .set_label(Some("buffer1"))
    ///                 .build(&gpu.device, &[0, 1, 2, 3]);
    /// let mut buffer2 = BufferBuilder::<u64>::new()
    ///                 .read().write().copy_dst()
    ///                 .set_label(Some("buffer2"))
    ///                 .build(&gpu.device, &[0, 0, 0, 0]);
    ///
    /// gpu.encode(|gpu, encoder|{
    ///     buffer1.slice(..).copy_to_buffer(&mut buffer2, 1, encoder);
    /// });
    ///
    /// gpu.encode(|gpu, encoder|{
    ///     assert_eq!(buffer2.slice(..).map_blocking(&gpu.device).as_ref(), [0, 0, 1, 2]);
    /// })
    /// ```
    ///
    pub fn copy_to_buffer(&self, dst: &mut Buffer<C>, offset: wgpu::BufferAddress, encoder: &mut wgpu::CommandEncoder){
        //let slice = self.buffer.buffer.slice(self.range_bytes);
        let range_addr = self.range_addr();
        let src_offset_bytes = range_addr.start as u64;

        let dst_offset_bytes = offset * std::mem::size_of::<C>() as u64;

        let size_bytes = (range_addr.end - range_addr.start) as u64;
        let size_bytes = size_bytes.min(dst.size() as u64 - dst_offset_bytes);

        encoder.copy_buffer_to_buffer(
            &self.buffer.buffer,
            src_offset_bytes,
            &dst.buffer,
            dst_offset_bytes,
            size_bytes
        );
    }
}

pub struct BufferSliceMut<'bs, C: bytemuck::Pod>{
    buffer: &'bs mut Buffer<C>,
    range: Range<usize>,
}

impl<'bs, C: bytemuck::Pod> BufferSliceMut<'bs, C>{
    ///
    /// Convert the range of elements (BufferSlice::range) into a range of bytes.
    ///
    #[inline]
    fn range_addr(&self) -> Range<wgpu::BufferAddress>{
        ((self.range.start * std::mem::size_of::<C>()) as u64)..((self.range.end * std::mem::size_of::<C>()) as u64)
    }

    ///
    /// Map the slice mutably whilst polling the device.
    ///
    ///
    pub async fn map_async_poll_mut(&'bs self, device: &wgpu::Device) -> BufferViewMut<'bs, C>{
        let range_addr = self.range_addr();

        let slice = self.buffer.buffer.slice(range_addr);
        let mapping = slice.map_async(wgpu::MapMode::Write);

        device.poll(wgpu::Maintain::Wait);

        mapping.await.unwrap();

        BufferViewMut{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(slice.get_mapped_range_mut()),
        }
    }

    ///
    /// Map the slice asynchronously for writing to the buffer.
    /// wgpu::Device::poll has to be called before this Future will complete.
    ///
    pub async fn map_async_mut(&'bs self) -> BufferViewMut<'bs, C>{
        let range_addr = self.range_addr();

        let slice = self.buffer.buffer.slice(range_addr);
        let mapping = slice.map_async(wgpu::MapMode::Write);

        mapping.await.unwrap();

        BufferViewMut{
            buffer: self.buffer,
            buffer_view: ManuallyDrop::new(slice.get_mapped_range_mut()),
        }
    }

    ///
    /// Map the slice mutably and block this thread untill maping is complete.
    ///
    /// slice.map_blocking_mut(device)[0] = 1;
    ///
    pub fn map_blocking_mut(&'bs self, device: &wgpu::Device) -> BufferViewMut<'bs, C>{
        pollster::block_on(self.map_async_poll_mut(device))
    }
}

///
/// A builder for a Buffer.
/// Be aware of the padding sometimes added in glsl for example:
/// A struct of with a vec3 type in glsl might be padded to 32 bytes.
/// Writing an array of [f32; 3] to that buffer might result
/// in a misalignment.
///
/// Example: 
/// ```
/// # use ewgpu::*;
/// # let gpu = GPUContextBuilder::new()
/// #   .set_features_util()
/// #   .build();
///
/// let buffer = BufferBuilder::new()
///     .uniform().copy_dst()
///     .set_label(Some("buffer"))
///     .build(&gpu.device, &[0, 1, 2, 3]);
///
/// ```
///
pub struct BufferBuilder<'bb, C: bytemuck::Pod>{
    usages: wgpu::BufferUsages,
    label: wgpu::Label<'bb>,
    _ty: PhantomData<C>,
}

impl<'bb, C: bytemuck::Pod> Default for BufferBuilder<'bb, C>{
    fn default() -> Self {
        Self{
            usages: wgpu::BufferUsages::empty(),
            label: None,
            _ty: PhantomData,
        }
    }
}

impl<'bb, C: bytemuck::Pod> BufferBuilder<'bb, C>{
    ///
    /// Create a new BufferBuilder
    /// wit emtpy buffer Usages and None as label.
    ///
    pub fn new() -> Self{
        Self{
            usages: wgpu::BufferUsages::empty(),
            label: None,
            _ty: PhantomData,
        }
    }

    ///
    /// Set the VERTEX usage.
    ///
    #[inline]
    pub fn vertex(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::VERTEX;
        self
    }
    ///
    /// Set the INDEX usage.
    ///
    #[inline]
    pub fn index(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::INDEX;
        self
    }
    ///
    /// Set the STORAGE usage.
    ///
    #[inline]
    pub fn storage(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::STORAGE;
        self
    }
    ///
    /// Set the UNIFORM usage.
    ///
    #[inline]
    pub fn uniform(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::UNIFORM;
        self
    }
    ///
    /// Set the COPY_DST usage.
    ///
    #[inline]
    pub fn copy_dst(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::COPY_DST;
        self
    }
    ///
    /// Set the COPY_SRC usage.
    ///
    #[inline]
    pub fn copy_src(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::COPY_SRC;
        self
    }
    ///
    /// Set the MAP_READ usage.
    ///
    #[inline]
    pub fn read(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::MAP_READ;
        self
    }
    ///
    /// Set the MAP_WRITE usage.
    ///
    #[inline]
    pub fn write(mut self) -> Self{
        self.usages |= wgpu::BufferUsages::MAP_WRITE;
        self
    }

    ///
    /// Set buffer usages for the buffer.
    ///
    #[inline]
    pub fn set_usage(mut self, usage: wgpu::BufferUsages) -> Self{
        self.usages = usage;
        self
    }

    ///
    /// Set the label of the buffer.
    ///
    #[inline]
    pub fn set_label(mut self, label: wgpu::Label<'bb>) -> Self{
        self.label = label;
        self
    }

    ///
    /// Build a buffer with data.
    ///
    pub fn build(&self, device: &wgpu::Device, data: &[C]) -> Buffer<C>{
        Buffer::<C>::new(
            device, 
            self.usages, 
            self.label, 
            data,
        )
    }

    ///
    /// Build a buffer with length. Data in the buffer is undefined.
    ///
    pub fn build_empty(&self, device: &wgpu::Device, len: usize) -> Buffer<C>{
        Buffer::<C>::new_empty(device, self.usages, self.label, len)
    }
}

// TODO: std140 and std430

///
/// A typesafe wrapper for wgpu::Buffer.
///
#[allow(unused)]
#[derive(DerefMut)]
pub struct Buffer<C: bytemuck::Pod>{
    #[target]
    pub buffer: wgpu::Buffer,
    len: usize,
    usage: wgpu::BufferUsages,
    label: Option<String>,
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

        let label = label.map(|x|{x.to_string()});

        Self{
            buffer,
            len,
            usage,
            label,
            _pd: PhantomData,
        }
    }

    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: wgpu::Label, data: &[C]) -> Self{
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label,
            contents: bytemuck::cast_slice(data),
            usage,
        });

        let label = label.map(|x|{x.to_string()});

        Self{
            buffer,
            len: data.len(),
            usage,
            label,
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

    ///
    /// Returns the number of elements in the buffer.
    ///
    #[inline]
    pub fn len(&self) -> usize{
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool{
        self.len == 0
    }

    ///
    /// Returns the size of the buffer in bytes.
    ///
    #[inline]
    pub fn size(&self) -> usize{
        self.len * std::mem::size_of::<C>()
    }

    // TODO: Export bound start and end to own functions.
    ///
    /// Slices have to be aligned with MAP_ALIGNMENT (8 bytes).
    /// TODO: for some reason the start_bound is multiplied by two.
    ///
    /// example: 
    /// ```rust
    /// # use ewgpu::*;
    /// # let gpu = GPUContextBuilder::new()
    /// #   .set_features_util()
    /// #   .build();
    ///
    /// let mut buffer = BufferBuilder::<u64>::new()
    ///     .read().write()
    ///     .set_label(Some("buffer"))
    ///     .build(&gpu.device, &[0, 1, 2, 3]);
    /// assert_eq!(buffer.slice(..).map_blocking(&gpu.device).as_ref(), [0, 1, 2, 3]);
    /// ```
    pub fn slice<S: RangeBounds<usize>>(&self, bounds: S) -> BufferSlice<C>{
        let range = bounds.clamp(0..self.len());

        BufferSlice{
            buffer: self,
            range,
        }
    }

    ///
    /// Get a mutable slice to the Buffer.
    /// The bound is clamped by the size of the Buffer.
    ///
    ///
    /// ```rust
    /// # use ewgpu::*;
    /// # let gpu = GPUContextBuilder::new()
    /// #   .set_features_util()
    /// #   .build();
    ///
    /// let mut buffer = BufferBuilder::<u64>::new()
    ///     .read().write()
    ///     .set_label(Some("buffer"))
    ///     .build(&gpu.device, &[0, 1, 2, 3]);
    /// buffer.slice_mut(0..).map_blocking_mut(&gpu.device)[0] = 8;
    /// assert_eq!(buffer.slice(..).map_blocking(&gpu.device).as_ref(), [8, 1, 2, 3]);
    /// ```
    ///
    pub fn slice_mut<S: RangeBounds<usize>>(&mut self, bounds: S) -> BufferSliceMut<C>{
        let range = bounds.clamp(0..self.len());

        BufferSliceMut{
            buffer: self,
            range,
        }
    }

    ///
    /// Expands a Buffer to a given size.
    /// This copies the content of the buffer to the new one.
    /// If the buffer is in a BindGroup this has to be updated.
    ///
    pub fn expand_to(&mut self, len: usize, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device){
        if len > self.len(){
            self.resize(len, encoder, device);
        }
    }

    ///
    /// Resizes a Buffer to a given size.
    /// This copies the content of the buffer cutting the excess.
    /// If the buffer is in a BindGroup this has to be updated.
    ///
    pub fn resize(&mut self, len: usize, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device){
        // Need to allow manual_map because we cannot use map as it would return a ref to a value
        // in the closure.
        #[allow(clippy::manual_map)]
        let label = match &self.label{
            Some(string) => Some(string.as_str()),
            None => None
        };
        let mut tmp_buf = Buffer::<C>::new_empty(device, self.usage, label, len);
        self.slice(..).copy_to_buffer(&mut tmp_buf, 0, encoder);
        *self = tmp_buf;
    }

    ///
    /// Expands a Buffer to a given size and clears it.
    /// This does not copy the content of the buffer.
    /// If the buffer is in a BindGroup this has to be updated.
    ///
    pub fn expand_to_clear(&mut self, len: usize, device: &wgpu::Device){
        if len > self.len(){
            self.resize_clear(len, device);
        }
    }

    ///
    /// Resizes a Buffer to a given size and clears it.
    /// This does not copy the content of the buffer.
    /// If the buffer is in a BindGroup this has to be updated.
    ///
    pub fn resize_clear(&mut self, len: usize, device: &wgpu::Device){
        // Need to allow manual_map because we cannot use map as it would return a ref to a value
        // in the closure.
        #[allow(clippy::manual_map)]
        let label = match &self.label{
            Some(string) => Some(string.as_str()),
            None => None
        };
        *self = Buffer::<C>::new_empty(device, self.usage, label, len);
    }

    // TODO: maybe move to slice.
    pub fn write_buffer(&mut self, queue: &wgpu::Queue, offset: usize, data: &[C]){
        queue.write_buffer(&self.buffer, (offset * std::mem::size_of::<C>()) as u64, bytemuck::cast_slice(data));
    }
}

impl<C: bytemuck::Pod> binding::BindGroupContent for Buffer<C>{
    fn entries(visibility: Option<wgpu::ShaderStages>) -> Vec<binding::BindGroupLayoutEntry>{
        vec!{
            binding::BindGroupLayoutEntry::new(visibility.unwrap_or(wgpu::ShaderStages::all()), binding::wgsl::buffer(false))
        }
    }

    fn resources(&self) -> Vec<wgpu::BindingResource> {
        vec!{
            self.as_entire_binding(),
        }
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

///
/// A mutable view into the Buffer.
/// Since this can only be derived from a mut_slice and all access to the view require mutable
/// references to it there can only ever be one BufferViewMut that is currently in use.
///
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

pub trait IndexFormat{
    fn index_format() -> wgpu::IndexFormat;
    fn get_index_format(&self) -> wgpu::IndexFormat{
        Self::index_format()
    }
}

impl IndexFormat for Buffer<u32>{
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}

impl IndexFormat for Buffer<u16>{
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
