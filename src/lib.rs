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
//! use wgpu_utils::*;
//!
//!#[repr(C)]
//!#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
//!struct Consts{
//!     pub color: [f32; 4],
//!}
//!
//!struct ImgState{
//!     pipeline: RenderPipeline,
//!     mesh: Mesh<Vert2>,
//!}
//!
//!impl State for ImgState{
//!     fn new(gpu: &mut GPUContext) -> Self{
//!         let mesh = Mesh::<Vert2>::new(&gpu.device, &Vert2::QUAD_VERTS,
//!         &Vert2::QUAD_IDXS).unwrap();
//!
//!         let vshader = VertexShader::from_src(&gpu.device, "
//!         #version 460
//!         #if VERTEX_SHADER
//!         
//!         layout(location = 0) in vec2 i_pos;
//!         layout(location = 1) in vec2 i_uv;
//!         
//!         layout(location = 0) out vec2 f_pos;
//!         layout(location = 1) out vec2 f_uv;
//!         
//!         void main(){
//!             f_pos = i_pos;
//!             f_uv = i_uv;
//!         
//!             gl_Position = vec4(i_pos, 0.0, 1.0);
//!         }
//!         
//!         #endif
//!         ", None).unwrap();
//!
//!         let fshader = FragmentShader::from_src(&gpu.device, "
//!         #version 460
//!         #if FRAGMENT_SHADER
//!         
//!         layout(location = 0) in vec2 f_pos;
//!         layout(location = 1) in vec2 f_uv;
//!         
//!         layout(location = 0) out vec4 o_color;
//!         
//!         layout(push_constant) uniform PushConstants{
//!             vec4 color;
//!         } constants;
//!         
//!         void main(){
//!             o_color = vec4(1.0, 0., 0., 1.);
//!         }
//!         
//!         #endif
//!         ", None).unwrap();
//!
//!         let layout = pipeline_layout!(&gpu.device,
//!             bind_groups: {},
//!             push_constants: {
//!                 Consts,
//!             }
//!         );
//!
//!         let pipeline = RenderPipelineBuilder::new(&vshader, &fshader)
//!             .push_drawable_layouts::<Mesh<Vert2>>()
//!             .push_target_replace(wgpu::TextureFormat::Rgba8Unorm)
//!             .set_layout(&layout)
//!             .build(&gpu.device);
//!
//!         Self{
//!             mesh,
//!             pipeline,
//!         }
//!     }
//!
//!     fn render(&mut self, gpu: &mut GPUContext){
//!         {
//!             gpu.encode_img([1920, 1080], |gpu, dst, encoder|{
//!                 let mut rpass = RenderPassBuilder::new()
//!                     .push_color_attachment(dst.color_attachment_clear())
//!                     .begin(encoder, None);
//!
//!                 let mut rpass_ppl = rpass.set_pipeline(&self.pipeline);
//!
//!                 let consts = Consts{
//!                     color: [1.0, 0.0, 0.0, 1.0]
//!                 };
//!
//!                 rpass_ppl.set_push_const(0, &consts);
//!
//!                 self.mesh.draw(&mut rpass_ppl);
//!             });
//!         }
//!     }
//!}
//!
//!```
//!

#[macro_use]
extern crate more_asserts;

pub extern crate wgpu_utils_macros;

//extern crate nalgebra_glm as glm;

pub mod binding;
pub mod buffer;
pub mod framework;
pub mod mesh;
pub mod pipeline;
pub mod render_target;
pub mod texture;
pub mod uniform;
pub mod vert;
pub mod push_constants;
pub mod shader;
pub mod gpu_context;

mod utils;


pub use self::binding::*;
pub use self::buffer::*;
pub use self::framework::*;
pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::render_target::*;
pub use self::texture::*;
pub use self::uniform::*;
pub use self::vert::*;
pub use self::push_constants::*;
pub use self::shader::*;
pub use crate::wgpu_utils_macros::*;
pub use crate::gpu_context::*;

