
#[macro_use]
extern crate more_asserts;

#[macro_use]
pub extern crate wgpu_utils_macros;

extern crate nalgebra_glm as glm;

pub mod binding;
pub mod buffer;
pub mod framework;
pub mod mesh;
pub mod pipeline;
pub mod render_target;
pub mod texture;
pub mod uniform;
pub mod vert;

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

#[cfg(test)]
mod tests {

    #[test]
    fn test_inst_buffer_layout(){
    }
}
