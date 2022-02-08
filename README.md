
# Wgpu Utils:

This is a simple wrapper for [wgpu](https://github.com/gfx-rs/wgpu) that
provides builders. It also re-implements some of the types from wgpu to use
generics and make it safer.  
## Examples: 
Wgpu Buffers can be unsafe when types do not match:
```rust
let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
    label: None,
    contents: bytemuck::cast_slice(data),
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
});
```
When reading the buffer back from the GPU it is not guaranteed that the buffer
to which one reads it has the same format. 

In this library:
```rust
let buffer = Buffer::<i32>::new_storage(device, None, data);
```

Now when reading the buffer back the mapped slice is of the same type as the
provided data and therefore no "undefined behaviour" will occur.

This library also provides builders for some of its types since they are easier
to use than wgpu's descriptors since they can provide sane defaults whilst
still allowing detailed configuration.

## Motivation:
I have noticed that nearly all projects using wgpu that are listed on their website [https://wgpu.rs/](https://wgpu.rs/) ([bevy](https://github.com/bevyengine/bevy), [Veloren](https://gitlab.com/veloren/veloren), [blub](https://github.com/wumpf/blub)) implement somewhat common wrappers around wgpu to make it easier and safer to program with it.

## Goals:
Hide all operations in wgpu that could result in panics and/or undefined
behaviour behind rust's safety infrastructure if possible.
