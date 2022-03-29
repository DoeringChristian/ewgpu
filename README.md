# Easy WGPU:



This crate provides a wrapper on [wgpu](https://github.com/gfx-rs/wgpu).
It is still in an early state and changes can happen that break existing code.
The main focus is to make the writing of graphics application,
for which engines are too high level easy.
This crate therefore re-implements some of the types from wgpu using
generics to make it safer and easier.
Though the main goal is comfort, runtime performance should not be impacted.

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
has the same format. 

In this library:
```rust
let buffer = BufferBuilder::new()
    .storage().read().write()
    .append_slice(&[0, 1, 2, 3])
    .build(device);
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
And make it as easy as possible to write simple graphics programs.

### Immediate Goals:
 - [ ] Write unit tests for all testable modules.
 - [ ] Rename all generic default names from C to T.
