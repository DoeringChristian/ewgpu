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

## Features:
 - [x] Buffers with type generics to prevent wrong casts.
 - [x] BindGroup generics that allow grouping of bind groups.
 - [x] BindGroup(Layout) builders.
 - [x] A framework for initializing a window and handling states.
 - [x] Generic mesh and model structs.
 - [x] Fragment/Vertex State builders.
 - [x] Pipeline layout builder.
 - [x] A seperate RenderPassPipeline type that could be used for predefined pipeline layouts.
 - [x] ComputePipeline builder.
 - [x] RenderPass builder.
 - [x] RenderPipeline builder.
 - [x] Render Target functions simplifying use of multiple color attachments.
 - [x] Texture with load and new functions.
 - [x] Uniforms with generic types.
 - [x] Vert2 default vertex struct.
 - [ ] Vert3 default vertex struct.

## Goals:
Hide all operations in wgpu that could result in panics and/or undefined
behaviour behind rust's safety infrastructure if possible.

### Immediate Goals:
 - [ ] Implement some way to use the Rust type system to prevent Buffers that are initialized without the COPY_DST usage to be target of a copy_to_buffer operation. 
 - [ ] Implement some way to prevent Buffers/Slices of buffers that are initialized without the COPY_SRC usage to be source of a copy_to_buffer operation.
 - [ ] Write unit tests for all testable modules.
 - [ ] Rename all generic default names from C to T.
 - [ ] Add functions to Framework that allow configuration.
 - [ ] Make ModelTransforms a generic in Model struct.
