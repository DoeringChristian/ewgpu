//!
//! A wrapper for the [wgpu](https://docs.rs/wgpu/latest/wgpu/) crate.
//!
//! This library provides wrappers for the [wgpu](https://docs.rs/wgpu/latest/wgpu/) crate 
//! that are typesafe and easy to use. For this Builders are provided for most of 
//! [wgpu](https://docs.rs/wgpu/latest/wgpu/)'s structs as well as wrappers with generics for structs of [wgpu](https://docs.rs/wgpu/latest/wgpu/) 
//! that hold data to prevent unintended casting.
//! This crate also provides macros for creating vertices and a Framework that enables easy
//! initialisation. 
//!
//! An example of how to render to a image:
//!```rust
//! use ewgpu::*;
//!
//! #[repr(C)]
//! #[make_vert]
//! struct Vert2{
//!     #[location = 0]
//!     pub pos: [f32; 2],
//!     #[location = 1]
//!     pub uv: [f32; 2],
//! }
//!
//! pub const QUAD_IDXS: [u32; 6] = [0, 1, 2, 2, 3, 0];
//!
//! pub const QUAD_VERTS: [Vert2; 4] = [
//!     Vert2 {
//!         pos: [-1.0, -1.0],
//!         uv: [0.0, 1.0],
//!     },
//!     Vert2 {
//!         pos: [1.0, -1.0],
//!         uv: [1.0, 1.0],
//!     },
//!     Vert2 {
//!         pos: [1.0, 1.0],
//!         uv: [1.0, 0.0],
//!     },
//!     Vert2 {
//!         pos: [-1.0, 1.0],
//!         uv: [0.0, 0.0],
//!     },
//! ];
//!
//! #[repr(C)]
//! #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
//! struct Consts{
//!      pub color: [f32; 4],
//! }
//! use winit::{window::WindowBuilder, event_loop::EventLoop};
//! 
//! env_logger::init();
//! 
//! let event_loop = EventLoop::new();
//! 
//! let mut gpu = GPUContextBuilder::new()
//!     .set_features_util()
//!     .set_limits(wgpu::Limits{
//!         max_push_constant_size: 128,
//!         ..Default::default()
//!     })
//!     .build();
//!
//! let indices = BufferBuilder::new()
//!     .index()
//!     .build(&gpu.device, &QUAD_IDXS);
//!
//! let vertices = BufferBuilder::new()
//!     .vertex()
//!     .build(&gpu.device, &QUAD_VERTS);
//!
//!
//! let vshader = VertexShader::from_src(&gpu.device, "
//! #version 460
//! #if VERTEX_SHADER
//!         
//! layout(location = 0) in vec2 i_pos;
//! layout(location = 1) in vec2 i_uv;
//!         
//! layout(location = 0) out vec2 f_pos;
//! layout(location = 1) out vec2 f_uv;
//!         
//! void main(){
//!     f_pos = i_pos;
//!     f_uv = i_uv;
//!         
//!     gl_Position = vec4(i_pos, 0.0, 1.0);
//! }
//!         
//! #endif
//! ", None).unwrap();
//!
//! let fshader = FragmentShader::from_src(&gpu.device, "
//! #version 460
//! #if FRAGMENT_SHADER
//!         
//! layout(location = 0) in vec2 f_pos;
//! layout(location = 1) in vec2 f_uv;
//!         
//! layout(location = 0) out vec4 o_color;
//!         
//! layout(push_constant) uniform PushConstants{
//!     vec4 color;
//! } constants;
//!         
//! void main(){
//!     o_color = vec4(1.0, 0., 0., 1.);
//! }
//!         
//! #endif
//! ", None).unwrap();
//!
//! let layout = pipeline_layout!(&gpu.device,
//!     bind_groups: {},
//!     push_constants: {
//!         Consts,
//!     }
//! );
//!
//! let pipeline = RenderPipelineBuilder::new(&vshader, &fshader)
//!     .push_vert_layout(Vert2::buffer_layout())
//!     .push_target_replace(wgpu::TextureFormat::Rgba8Unorm)
//!     .set_layout(&layout)
//!     .build(&gpu.device);
//! 
//! gpu.encode_img([1920, 1080], |gpu, dst, encoder|{
//!     let mut rpass = RenderPassBuilder::new()
//!         .push_color_attachment(dst.color_attachment_clear())
//!         .begin(encoder, None);
//!
//!     let mut rpass_ppl = rpass.set_pipeline(&pipeline);
//!
//!     let consts = Consts{
//!         color: [1.0, 0.0, 0.0, 1.0]
//!     };
//!
//!     rpass_ppl.set_push_const(0, &consts);
//!     rpass_ppl.set_vertex_buffer(0, vertices.slice(..));
//!     rpass_ppl.set_index_buffer(indices.slice(..));
//!     rpass_ppl.draw_indexed(0..(indices.len() as u32), 0, 0..1);
//! });
//!
//!```
//!

#[macro_use]
extern crate more_asserts;

#[macro_use]
pub extern crate ewgpu_macros;

//extern crate nalgebra_glm as glm;

pub mod binding;
pub mod buffer;
pub mod pipeline;
pub mod render_target;
pub mod texture;
pub mod uniform;
pub mod vert;
pub mod push_constants;
pub mod shader;
pub mod context;
pub mod utils;


pub use self::binding::*;
pub use self::buffer::*;
//pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::render_target::*;
pub use self::texture::*;
pub use self::uniform::*;
pub use self::vert::*;
pub use self::push_constants::*;
pub use self::shader::*;
pub use crate::ewgpu_macros::*;
pub use context::*;

